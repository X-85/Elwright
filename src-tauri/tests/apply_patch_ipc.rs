//! 代码浏览器阶段④（ADR-001）：受控补丁编辑 IPC 冒烟。
//!
//! 三例：
//! 1. preview：合法 diff → 解析 + 预览，不写文件
//! 2. apply + revert：写入项目根内文件，返回 snapshotId；再调 revert 还原
//! 3. sensitive：敏感路径（.env / .ssh / node_modules）preview 阶段就被拒

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde_json::json;
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;

type Wv = tauri::WebviewWindow<tauri::test::MockRuntime>;

/// 本测试用 ELWRIGHT_USER_ROOT 隔离用户层快照；进程级环境变量需互斥。
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// 每个测试独立 user root + 锁（持锁覆盖整个测试函数，避免其他测试中途改 env）。
/// 返回 (project_root, user_root, drop guard)。
fn setup(tag: &str) -> (PathBuf, PathBuf, impl Drop) {
    let guard = env_lock();
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let proj = std::env::temp_dir().join(format!(
        "elwright-patch-ipc-proj-{}-{}-{}",
        std::process::id(),
        tag,
        seq
    ));
    let user = std::env::temp_dir().join(format!(
        "elwright-patch-ipc-user-{}-{}-{}",
        std::process::id(),
        tag,
        seq
    ));
    let _ = fs::remove_dir_all(&proj);
    let _ = fs::remove_dir_all(&user);
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&user).unwrap();
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(
        proj.join("src/foo.rs"),
        "alpha\nbeta\ngamma\ndelta\nepsilon\n",
    )
    .unwrap();
    fs::create_dir_all(proj.join("node_modules/pkg")).unwrap();
    fs::write(
        proj.join("node_modules/pkg/index.js"),
        "module.exports = {};\n",
    )
    .unwrap();
    fs::write(proj.join(".env"), "SECRET=x\n").unwrap();
    std::env::set_var("ELWRIGHT_USER_ROOT", &user);
    struct Guard(
        #[allow(dead_code)] std::sync::MutexGuard<'static, ()>,
        PathBuf,
        PathBuf,
    );
    impl Drop for Guard {
        fn drop(&mut self) {
            std::env::remove_var("ELWRIGHT_USER_ROOT");
            let _ = fs::remove_dir_all(&self.1);
            let _ = fs::remove_dir_all(&self.2);
        }
    }
    (proj.clone(), user.clone(), Guard(guard, proj, user))
}

fn build_wv() -> Wv {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            elwright_core::core::commands::apply_patch_preview,
            elwright_core::core::commands::apply_patch_apply,
            elwright_core::core::commands::apply_patch_revert,
            elwright_core::core::commands::apply_patch_snapshots,
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
fn preview_does_not_write() {
    let wv = build_wv();
    let (root, _user, _guard) = setup("preview");
    let root_str = root.to_string_lossy().to_string();
    let original = fs::read_to_string(root.join("src/foo.rs")).unwrap();

    let patch_text =
        "--- a/src/foo.rs\n+++ b/src/foo.rs\n@@ -1,3 +1,3 @@\n alpha\n-beta\n+BETA\n gamma\n";

    let preview = call_ok(
        &wv,
        "apply_patch_preview",
        json!({"projectRoot": root_str, "patchText": patch_text}),
    );
    let files = preview["files"].as_array().unwrap();
    assert_eq!(files.len(), 1, "preview 只解析一个文件");
    assert_eq!(files[0]["file"], "src/foo.rs");
    assert!(files[0]["new_content"].as_str().unwrap().contains("BETA"));
    assert!(preview["warnings"].as_array().unwrap().is_empty());

    // 预览不应写文件
    let after = fs::read_to_string(root.join("src/foo.rs")).unwrap();
    assert_eq!(after, original, "preview 不应触碰文件");
}

#[test]
fn apply_then_revert_round_trip() {
    let wv = build_wv();
    let (root, _user, _guard) = setup("roundtrip");
    let root_str = root.to_string_lossy().to_string();
    let original = fs::read_to_string(root.join("src/foo.rs")).unwrap();

    let patch_text =
        "--- a/src/foo.rs\n+++ b/src/foo.rs\n@@ -1,3 +1,3 @@\n alpha\n-beta\n+BETA\n gamma\n";

    let preview = call_ok(
        &wv,
        "apply_patch_preview",
        json!({"projectRoot": root_str, "patchText": patch_text}),
    );
    let files = preview["files"].as_array().unwrap().clone();

    let apply_result = call_ok(
        &wv,
        "apply_patch_apply",
        json!({"projectRoot": root_str, "previews": files}),
    );
    let snapshot_id = apply_result["snapshot_id"].as_str().unwrap().to_string();
    assert_eq!(apply_result["applied"].as_array().unwrap().len(), 1);
    assert_eq!(apply_result["skipped"].as_array().unwrap().len(), 0);

    let after_apply = fs::read_to_string(root.join("src/foo.rs")).unwrap();
    assert!(after_apply.contains("BETA"), "应用后文件应含 BETA");

    let revert_result = call_ok(
        &wv,
        "apply_patch_revert",
        json!({"projectRoot": root_str, "snapshotId": snapshot_id}),
    );
    assert_eq!(revert_result["restored"].as_array().unwrap().len(), 1);

    let after_revert = fs::read_to_string(root.join("src/foo.rs")).unwrap();
    assert_eq!(after_revert, original, "撤销后文件应回到原状");

    let snaps = call_ok(
        &wv,
        "apply_patch_snapshots",
        json!({"projectRoot": root_str}),
    );
    assert_eq!(snaps.as_array().unwrap().len(), 0);
}

#[test]
fn sensitive_path_rejected() {
    let wv = build_wv();
    let (root, _user, _guard) = setup("sensitive");
    let root_str = root.to_string_lossy().to_string();

    let patch_env = "--- a/.env\n+++ b/.env\n@@ -1,1 +1,1 @@\n-SECRET=x\n+SECRET=y\n";
    let r = call(
        &wv,
        "apply_patch_preview",
        json!({"projectRoot": root_str, "patchText": patch_env}),
    );
    assert!(r.is_ok(), "preview 阶段不抛错，仅在 warnings 标注");
    let v = r.unwrap();
    let files = v["files"].as_array().unwrap();
    let warns = v["warnings"].as_array().unwrap();
    assert_eq!(files.len(), 0, "敏感文件不进 files");
    assert!(warns
        .iter()
        .any(|w| w.as_str().unwrap_or_default().contains(".env")));

    let patch_nm = "--- a/node_modules/pkg/index.js\n+++ b/node_modules/pkg/index.js\n@@ -1,1 +1,1 @@\n-module.exports = {};\n+module.exports = 1;\n";
    let r = call(
        &wv,
        "apply_patch_preview",
        json!({"projectRoot": root_str, "patchText": patch_nm}),
    );
    assert!(r.is_ok());
    let v = r.unwrap();
    let warns = v["warnings"].as_array().unwrap();
    assert!(warns
        .iter()
        .any(|w| w.as_str().unwrap_or_default().contains("node_modules")));
}
