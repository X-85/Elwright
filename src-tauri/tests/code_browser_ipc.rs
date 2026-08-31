//! 代码浏览器 IPC 冒烟：mock runtime 走真实 IPC 协议调命令
//! （项目根用临时目录，不碰真实用户数据）。

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;

type Wv = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn build_wv() -> Wv {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            elwright_core::core::commands::code_browser_tree,
            elwright_core::core::commands::code_browser_read,
            elwright_core::core::commands::code_browser_search,
            elwright_core::core::commands::code_browser_scan_symbols,
            elwright_core::core::commands::code_browser_recent_load,
            elwright_core::core::commands::code_browser_recent_open,
            elwright_core::core::commands::code_browser_recent_remove_project,
            elwright_core::core::commands::code_browser_favorites_toggle,
            elwright_core::core::commands::code_browser_bookmarks_toggle,
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

/// 最近/收藏/书签命令读写真实用户层 ~/.elwright/code-browser.json（load→改→save），
/// 本文件多个测试并行执行会互相丢更新——用互斥锁把碰用户层的测试串行化。
static USER_STORE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn temp_project() -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "elwright-cb-ipc-{}-{}-{}",
        std::process::id(),
        seq,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(dir.join("src/main/java")).unwrap();
    fs::write(
        dir.join("src/main/java/UserService.java"),
        "public interface UserService {\n    User getById(Long id);\n}\n",
    )
    .unwrap();
    fs::write(
        dir.join("src/main/java/UserServiceImpl.java"),
        "public class UserServiceImpl implements UserService {\n    public User getById(Long id) { return null; }\n}\n",
    )
    .unwrap();
    fs::write(dir.join("README.md"), "# demo\nfind-me-here\n").unwrap();
    dir
}

#[test]
fn tree_read_search_symbols_over_ipc() {
    let _guard = USER_STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let wv = build_wv();
    let root = temp_project();
    let root_str = root.to_string_lossy().to_string();

    // 目录树：src 目录在前
    let tree = call_ok(
        &wv,
        "code_browser_tree",
        json!({"projectRoot": root_str, "rel": ""}),
    );
    assert_eq!(tree[0]["name"], "src", "{tree}");
    assert_eq!(tree[0]["kind"], "dir");

    // 文件读取：语言识别 + 内容
    let doc = call_ok(
        &wv,
        "code_browser_read",
        json!({"projectRoot": root_str, "rel": "src/main/java/UserService.java"}),
    );
    assert_eq!(doc["language"], "java");
    assert!(doc["content"]
        .as_str()
        .unwrap()
        .contains("interface UserService"));

    // 穿越：IPC 层报错而非成功
    let err = call(
        &wv,
        "code_browser_read",
        json!({"projectRoot": root_str, "rel": "../../../etc/passwd"}),
    );
    assert!(err.is_err(), "路径穿越应被拒绝");

    // 内容搜索
    let hits = call_ok(
        &wv,
        "code_browser_search",
        json!({"projectRoot": root_str, "query": "FIND-ME", "mode": "content"}),
    );
    assert_eq!(hits.as_array().unwrap().len(), 1);
    assert_eq!(hits[0]["line"], 2);

    // 轻量符号：interface + class + method
    let symbols = call_ok(
        &wv,
        "code_browser_scan_symbols",
        json!({"projectRoot": root_str}),
    );
    let arr = symbols.as_array().unwrap();
    assert!(
        arr.iter()
            .any(|s| s["name"] == "UserService" && s["kind"] == "interface"),
        "{symbols}"
    );
    assert!(arr
        .iter()
        .any(|s| s["name"] == "UserServiceImpl" && s["kind"] == "class"));
    assert!(arr
        .iter()
        .any(|s| s["name"] == "getById" && s["kind"] == "method"));

    // 收藏与书签：切换写用户层（~/.elwright/code-browser.json）。
    // 用户层可能已有真实数据（非 CI 全新 HOME），断言一律用增量而非绝对长度。
    let recent_before = call_ok(&wv, "code_browser_recent_load", json!({}));
    let favs_before = recent_before["favorites"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let bms_before = recent_before["bookmarks"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);

    let favs = call_ok(
        &wv,
        "code_browser_favorites_toggle",
        json!({"projectRoot": root_str, "rel": "src/main/java/UserService.java"}),
    );
    assert_eq!(
        favs.as_array().unwrap().len(),
        favs_before + 1,
        "收藏应恰好新增一条"
    );
    let favs = call_ok(
        &wv,
        "code_browser_favorites_toggle",
        json!({"projectRoot": root_str, "rel": "src/main/java/UserService.java"}),
    );
    assert_eq!(
        favs.as_array().unwrap().len(),
        favs_before,
        "二次切换应移除"
    );

    let bms = call_ok(
        &wv,
        "code_browser_bookmarks_toggle",
        json!({"projectRoot": root_str, "rel": "src/main/java/UserService.java", "line": 2, "label": "回归"}),
    );
    assert_eq!(
        bms.as_array().unwrap().len(),
        bms_before + 1,
        "书签应恰好新增一条"
    );
    assert!(bms
        .as_array()
        .unwrap()
        .iter()
        .any(|b| b["projectRoot"] == root_str.as_str() && b["line"] == 2));

    // 还原用户层：bookmarks 对称 toggle 一次移除本测试条目，存量数据不动
    let bms = call_ok(
        &wv,
        "code_browser_bookmarks_toggle",
        json!({"projectRoot": root_str, "rel": "src/main/java/UserService.java", "line": 2, "label": "回归"}),
    );
    assert_eq!(bms.as_array().unwrap().len(), bms_before, "二次切换应移除");

    fs::remove_dir_all(&root).ok();
}

#[test]
fn recent_project_remove_over_ipc() {
    let _guard = USER_STORE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let wv = build_wv();
    let root = temp_project();
    let root_str = root.to_string_lossy().to_string();

    // 打开临时项目 → 写入最近列表（rel 为空不产生最近文件）
    let opened = call_ok(
        &wv,
        "code_browser_recent_open",
        json!({"projectRoot": root_str, "rel": ""}),
    );
    assert!(
        opened["projects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["rootPath"] == root_str.as_str()),
        "打开后应出现在最近项目: {opened}"
    );

    // 删除 → 消失
    let after = call_ok(
        &wv,
        "code_browser_recent_remove_project",
        json!({"projectRoot": root_str}),
    );
    assert!(
        !after["projects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["rootPath"] == root_str.as_str()),
        "删除后不应再出现: {after}"
    );

    // 幂等：再删一次仍成功
    let again = call_ok(
        &wv,
        "code_browser_recent_remove_project",
        json!({"projectRoot": root_str}),
    );
    assert!(again["projects"].is_array());

    fs::remove_dir_all(&root).ok();
}
