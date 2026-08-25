//! 工作工具栏命令的 IPC 冒烟：mock runtime 真协议调用（与 terminal_ipc.rs
//! 同 harness 模式）。命令层不需要 State（user_root 自解析），测试用
//! ELWRIGHT_USER_ROOT 指向临时目录隔离真实用户层。
//!
//! 集成测试二进制与 lib 单测分属不同进程，env 互不干扰；本文件内多测试
//! 并行共享进程，用文件级锁串行化 env 写入。

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use elwright_core::core::commands;
use serde_json::json;
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;

type Wv = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn build_webview() -> Wv {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            commands::todo_list,
            commands::todo_add,
            commands::todo_toggle,
            commands::todo_remove,
            commands::note_get,
            commands::note_save,
            commands::note_list,
        ])
        .build(mock_context(noop_assets()))
        .expect("build mock app");
    tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("创建 main webview")
}

fn request(cmd: &str, body: serde_json::Value) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
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

/// 每个测试独立 user root + 锁，互不串扰。
fn setup(tag: &str) -> (PathBuf, impl Drop) {
    let guard = env_lock();
    let dir = std::env::temp_dir().join(format!("elwright-wbipc-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::env::set_var("ELWRIGHT_USER_ROOT", &dir);
    struct Guard(std::sync::MutexGuard<'static, ()>, PathBuf);
    impl Drop for Guard {
        fn drop(&mut self) {
            std::env::remove_var("ELWRIGHT_USER_ROOT");
            let _ = std::fs::remove_dir_all(&self.1);
        }
    }
    (dir.clone(), Guard(guard, dir))
}

#[test]
fn todo_add_list_toggle_remove_roundtrip_via_ipc() {
    let (_root, _guard) = setup("todo");
    let wv = build_webview();

    let added = call_ok(&wv, "todo_add", json!({ "text": "写周报" }));
    assert_eq!(added["text"], "写周报");
    assert_eq!(added["done"], false);
    assert!(added["id"].is_number(), "camelCase 字段可被前端读取");
    assert!(added.get("createdAt").is_some(), "createdAt 非 snake_case");

    let id = added["id"].as_u64().unwrap();
    let toggled = call_ok(&wv, "todo_toggle", json!({ "id": id }));
    assert_eq!(toggled["done"], true);
    assert!(toggled["completedAt"].is_string());

    let list = call_ok(&wv, "todo_list", json!({}));
    assert_eq!(list.as_array().unwrap().len(), 1);

    call_ok(&wv, "todo_remove", json!({ "id": id }));
    assert_eq!(
        call_ok(&wv, "todo_list", json!({}))
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn todo_unknown_id_and_blank_text_error_in_chinese() {
    let (_root, _guard) = setup("todo-err");
    let wv = build_webview();

    let err = call(&wv, "todo_toggle", json!({ "id": 9999 })).expect_err("未知 id 应报错");
    assert!(err.get("__TAURI_ERROR__").is_some() || err.is_string());

    let blank = call(&wv, "todo_add", json!({ "text": "   " })).expect_err("空文本应报错");
    let msg = blank
        .is_string()
        .then(|| blank.as_str().unwrap().to_string());
    if let Some(m) = msg {
        assert!(m.contains("不能为空"), "中文错误: {m}");
    }
}

#[test]
fn note_save_get_list_roundtrip_via_ipc() {
    let (root, _guard) = setup("note");
    let wv = build_webview();

    // 无记录 → null（前端显示空编辑器）
    assert_eq!(
        call_ok(&wv, "note_get", json!({ "date": "2026-08-25" })),
        serde_json::Value::Null
    );

    call_ok(
        &wv,
        "note_save",
        json!({ "date": "2026-08-25", "content": "# 今日" }),
    );
    let got = call_ok(&wv, "note_get", json!({ "date": "2026-08-25" }));
    assert_eq!(got, "# 今日");

    call_ok(
        &wv,
        "note_save",
        json!({ "date": "2026-08-24", "content": "昨天" }),
    );
    let dates = call_ok(&wv, "note_list", json!({}));
    assert_eq!(dates, json!(["2026-08-25", "2026-08-24"]));

    assert!(root.join("notes/2026-08-25.md").exists(), "落盘 notes/");
}

#[test]
fn note_invalid_date_rejected_via_ipc() {
    let (_root, _guard) = setup("note-err");
    let wv = build_webview();

    for bad in ["../evil", "2026-8-5", "2026/08/25"] {
        let err = call(&wv, "note_save", json!({ "date": bad, "content": "x" }))
            .expect_err("非法日期应报错");
        assert!(err.is_string(), "错误应为字符串: {err:?}");
    }
}
