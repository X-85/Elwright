//! 设置中心 模型档案（Q19）IPC 冒烟。
//!
//! 五例：
//! 1. list 空
//! 2. save + list 可见
//! 3. set_active + get_active 同步
//! 4. delete active → 自动清空 active
//! 5. 旧 flat 兼容（无 profiles 字段时返回空 list + get_active = None）

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;

type Wv = tauri::WebviewWindow<tauri::test::MockRuntime>;

/// 进程级环境变量互斥。
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// 每个测试独立 user root + 持锁覆盖全函数。
fn setup(tag: &str) -> (PathBuf, impl Drop) {
    let guard = env_lock();
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let user = std::env::temp_dir().join(format!(
        "elwright-llm-ipc-user-{}-{}-{}",
        std::process::id(),
        tag,
        seq
    ));
    let _ = fs::remove_dir_all(&user);
    fs::create_dir_all(&user).unwrap();
    std::env::set_var("ELWRIGHT_USER_ROOT", &user);
    struct Guard(
        #[allow(dead_code)] std::sync::MutexGuard<'static, ()>,
        PathBuf,
    );
    impl Drop for Guard {
        fn drop(&mut self) {
            std::env::remove_var("ELWRIGHT_USER_ROOT");
            let _ = fs::remove_dir_all(&self.1);
        }
    }
    (user.clone(), Guard(guard, user))
}

fn build_wv() -> Wv {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            elwright_core::core::commands::llm_list_profiles,
            elwright_core::core::commands::llm_get_active_profile,
            elwright_core::core::commands::llm_set_active_profile,
            elwright_core::core::commands::llm_save_profile,
            elwright_core::core::commands::llm_delete_profile,
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

#[test]
fn list_empty_initially() {
    let (_user, _g) = setup("list-empty");
    let wv = build_wv();
    let out = call_ok(&wv, "llm_list_profiles", serde_json::json!({}));
    let arr = out.as_array().expect("list 返回数组");
    assert!(arr.is_empty(), "初始应为空 list");
    let active = call_ok(&wv, "llm_get_active_profile", serde_json::json!({}));
    assert!(active.is_null(), "初始 active 为 null");
}

#[test]
fn save_then_list_appears() {
    let (_user, _g) = setup("save-list");
    let wv = build_wv();
    call_ok(
        &wv,
        "llm_save_profile",
        serde_json::json!({
            "profile": {
                "name": "work",
                "base_url": "http://w/v1",
                "api_key": "wk",
                "model": "wm"
            }
        }),
    );
    let arr = call_ok(&wv, "llm_list_profiles", serde_json::json!({}))
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "work");
    assert_eq!(arr[0]["active"], false);
    assert_eq!(arr[0]["source"], "user");
}

#[test]
fn set_active_then_get_active_reflects() {
    let (_user, _g) = setup("active");
    let wv = build_wv();
    call_ok(
        &wv,
        "llm_save_profile",
        serde_json::json!({
            "profile": {
                "name": "local-ollama",
                "base_url": "http://localhost:11434/v1",
                "api_key": "",
                "model": "qwen2.5-coder:7b"
            }
        }),
    );
    call_ok(
        &wv,
        "llm_set_active_profile",
        serde_json::json!({ "name": "local-ollama" }),
    );
    let active = call_ok(&wv, "llm_get_active_profile", serde_json::json!({}));
    assert_eq!(active, "local-ollama");
    let arr = call_ok(&wv, "llm_list_profiles", serde_json::json!({}))
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(arr[0]["active"], true);
}

#[test]
fn delete_active_clears_active_field() {
    let (_user, _g) = setup("delete-active");
    let wv = build_wv();
    call_ok(
        &wv,
        "llm_save_profile",
        serde_json::json!({
            "profile": {
                "name": "work",
                "base_url": "http://w/v1",
                "api_key": "k",
                "model": "m"
            }
        }),
    );
    call_ok(
        &wv,
        "llm_set_active_profile",
        serde_json::json!({ "name": "work" }),
    );
    call_ok(
        &wv,
        "llm_delete_profile",
        serde_json::json!({ "name": "work" }),
    );
    let active = call_ok(&wv, "llm_get_active_profile", serde_json::json!({}));
    assert!(active.is_null(), "删除激活档案后 active 应清空");
    let arr = call_ok(&wv, "llm_list_profiles", serde_json::json!({}))
        .as_array()
        .unwrap()
        .clone();
    assert!(arr.is_empty());
    // 二次 delete → IPC 返回 Err
    let err = call(
        &wv,
        "llm_delete_profile",
        serde_json::json!({ "name": "work" }),
    );
    assert!(err.is_err(), "删除不存在的档案应报错");
}

#[test]
fn flat_only_config_keeps_legacy_working() {
    let (user, _g) = setup("flat-only");
    // 直接落一份纯 flat 字段文件，模拟 v0.1.10 之前的用户配置
    fs::write(
        user.join("config.json"),
        r#"{"base_url":"http://legacy/v1","api_key":"k","model":"m"}"#,
    )
    .unwrap();
    let wv = build_wv();
    let arr = call_ok(&wv, "llm_list_profiles", serde_json::json!({}));
    assert!(
        arr.as_array().unwrap().is_empty(),
        "无 profiles 字段时 list 应为空"
    );
    let active = call_ok(&wv, "llm_get_active_profile", serde_json::json!({}));
    assert!(
        active.is_null(),
        "无 active_profile 时 get_active 应为 null"
    );
    // set_active 引用不存在的 name → 报错
    let err = call(
        &wv,
        "llm_set_active_profile",
        serde_json::json!({ "name": "ghost" }),
    );
    assert!(
        err.is_err(),
        "对 flat-only 配置 set_active 不存在的 name 应报错"
    );
}
