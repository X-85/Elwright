//! AI 对话 IPC 冒烟：本地 mock LLM 服务验证 ADR-004 上下文裁剪真实生效。
//!
//! 环境变量经 test_env::env_serialization_guard 串行化；ELWRIGHT_LLM_* 指向
//! 本测试自起的 127.0.0.1 mock 服务（优先级最高，隔离真实用户配置）。

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use serde_json::json;
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::Manager;

type Wv = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn build_wv(root: std::path::PathBuf) -> Wv {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            elwright_core::core::commands::chat_completion,
        ])
        .build(mock_context(noop_assets()))
        .expect("build mock app");
    app.manage(elwright_core::core::commands::AppCtx {
        root,
        terminal: elwright_core::core::terminal::SessionRegistry::new(std::sync::Arc::new(
            elwright_core::core::terminal::LocalBackend::new(),
        )),
    });
    tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("创建 main webview")
}

fn request(cmd: &str, body: serde_json::Value) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        // Windows 下 Tauri IPC origin 是 http://tauri.localhost（与 terminal_ipc.rs 同源处理）
        url: if cfg!(any(windows, target_os = "android")) {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        }
        .parse()
        .unwrap(),
        body: body.into(),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    }
}

fn call(
    wv: &Wv,
    cmd: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    get_ipc_response(wv, request(cmd, body)).map(|b| match &b {
        tauri::ipc::InvokeResponseBody::Json(s) => {
            serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
        }
        tauri::ipc::InvokeResponseBody::Raw(bytes) => {
            serde_json::from_slice(bytes).unwrap_or(serde_json::Value::Null)
        }
    })
}

fn call_ok(wv: &Wv, cmd: &str, body: serde_json::Value) -> serde_json::Value {
    call(wv, cmd, body).expect("命令应成功")
}

/// 单请求 mock LLM：记录收到的请求体原文，返回固定 OpenAI 兼容响应。
fn spawn_mock_llm() -> (String, Arc<Mutex<String>>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind mock llm");
    let addr = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(String::new()));
    let cap = captured.clone();
    std::thread::spawn(move || {
        let Ok((mut sock, _)) = listener.accept() else {
            return;
        };
        let mut buf: Vec<u8> = Vec::new();
        let mut tmp = [0u8; 8192];
        let header_end = loop {
            if let Some(pos) = buf.windows(4).rposition(|w| w == b"\r\n\r\n") {
                break pos + 4;
            }
            let n = match sock.read(&mut tmp) {
                Ok(0) | Err(_) => break buf.len(),
                Ok(n) => n,
            };
            buf.extend_from_slice(&tmp[..n]);
        };
        let head = String::from_utf8_lossy(&buf[..header_end]).to_ascii_lowercase();
        let content_length: usize = head
            .split("\r\n")
            .find_map(|l| {
                l.strip_prefix("content-length:")
                    .map(|v| v.trim().parse().unwrap())
            })
            .unwrap_or(0);
        while buf.len() < header_end + content_length {
            let n = match sock.read(&mut tmp) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            buf.extend_from_slice(&tmp[..n]);
        }
        *cap.lock().unwrap() = String::from_utf8_lossy(&buf[header_end..]).to_string();
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"mock-ok"}}]}"#;
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = sock.write_all(resp.as_bytes());
        let _ = sock.flush();
    });
    (format!("http://{addr}"), captured)
}

#[test]
fn chat_completion_trims_long_history_over_ipc() {
    // 环境变量为进程级：本测试是所在二进制里唯一用例，天然与其他测试二进制隔离
    // （lib 侧测试在另一进程；core::test_env 锁为 #[cfg(test)] 门控，集成测试不可见）

    // 隔离用户层（双保险：ELWRIGHT_LLM_* 已是最高优先级）
    let user_root = std::env::temp_dir().join(format!(
        "elwright-chat-ipc-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&user_root).unwrap();
    std::env::set_var("ELWRIGHT_USER_ROOT", &user_root);

    let budget: usize = 3_000;
    let (base_url, captured) = spawn_mock_llm();
    std::env::set_var("ELWRIGHT_LLM_BASE_URL", &base_url);
    std::env::set_var("ELWRIGHT_LLM_API_KEY", "sk-mock");
    std::env::set_var("ELWRIGHT_LLM_MODEL", "mock-model");
    std::env::set_var("ELWRIGHT_LLM_CONTEXT_BUDGET_CHARS", budget.to_string());

    let wv = build_wv(std::path::PathBuf::from(".."));

    // 6 轮历史（每条 ~800 字符，总 ~9.6k）+ 最新问题，远超 3000 预算
    let mut messages: Vec<serde_json::Value> = Vec::new();
    for i in 0..6 {
        messages.push(json!({"role": "user", "content": format!("u{i}-{}", "x".repeat(800))}));
        messages.push(json!({"role": "assistant", "content": format!("a{i}-{}", "y".repeat(800))}));
    }
    messages.push(json!({"role": "user", "content": "最终问题"}));

    let reply = call_ok(&wv, "chat_completion", json!({ "messages": messages }));
    assert_eq!(reply, "mock-ok", "mock LLM 应正常返回");

    let raw = captured.lock().unwrap().clone();
    assert!(!raw.is_empty(), "mock LLM 应收到请求");
    let body: serde_json::Value = serde_json::from_str(&raw).expect("请求体应为 JSON");
    let sent = body["messages"].as_array().expect("messages 数组");

    assert_eq!(sent[0]["role"], "system", "system 由 core 前置");
    assert_eq!(
        sent.last().unwrap()["content"],
        "最终问题",
        "最新 user 必留且不被改动"
    );

    // 历史（不含 system）总量收敛到预算内
    let history_chars: usize = sent[1..]
        .iter()
        .map(|m| m["content"].as_str().unwrap().chars().count())
        .sum();
    assert!(
        history_chars <= budget,
        "裁剪后历史应 ≤ 预算：{history_chars} > {budget}"
    );

    // 最旧一轮应被整条丢弃，较新内容保留
    let raw_all = raw.clone();
    assert!(!raw_all.contains("u0-"), "最旧轮次应被裁掉");
    assert!(
        sent.iter()
            .any(|m| m["content"].as_str().unwrap_or("").starts_with("u5-")),
        "较新轮次应保留: {sent:?}"
    );

    std::env::remove_var("ELWRIGHT_LLM_BASE_URL");
    std::env::remove_var("ELWRIGHT_LLM_API_KEY");
    std::env::remove_var("ELWRIGHT_LLM_MODEL");
    std::env::remove_var("ELWRIGHT_LLM_CONTEXT_BUDGET_CHARS");
    std::env::remove_var("ELWRIGHT_USER_ROOT");
    std::fs::remove_dir_all(&user_root).ok();
}
