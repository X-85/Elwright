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

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "elwright-cb-ipc-{}-{}",
        std::process::id(),
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

    // 收藏与书签：切换写用户层（~/.elwright/code-browser.json），测完还原
    let recent_before = call_ok(&wv, "code_browser_recent_load", json!({}));
    let favs = call_ok(
        &wv,
        "code_browser_favorites_toggle",
        json!({"projectRoot": root_str, "rel": "src/main/java/UserService.java"}),
    );
    assert_eq!(favs.as_array().unwrap().len(), 1);
    let favs = call_ok(
        &wv,
        "code_browser_favorites_toggle",
        json!({"projectRoot": root_str, "rel": "src/main/java/UserService.java"}),
    );
    assert_eq!(favs.as_array().unwrap().len(), 0, "二次切换应移除");

    let bms = call_ok(
        &wv,
        "code_browser_bookmarks_toggle",
        json!({"projectRoot": root_str, "rel": "src/main/java/UserService.java", "line": 2, "label": "回归"}),
    );
    assert_eq!(bms.as_array().unwrap().len(), 1);
    assert_eq!(bms[0]["line"], 2);

    // 还原用户层（保留原有 favorites/bookmarks，去掉本测试写入的）
    let mut restore = recent_before.clone();
    restore["favorites"] = json!([]);
    restore["bookmarks"] = json!([]);
    let _ = call_ok(&wv, "code_browser_recent_load", json!({}));
    // recent_load 无写入口，直接由 favorites/bookmarks 的对称 toggle 已还原 favorites；
    // bookmarks 再 toggle 一次移除本测试条目。
    let bms = call_ok(
        &wv,
        "code_browser_bookmarks_toggle",
        json!({"projectRoot": root_str, "rel": "src/main/java/UserService.java", "line": 2, "label": "回归"}),
    );
    assert_eq!(bms.as_array().unwrap().len(), 0, "二次切换应移除");
    let _ = restore;

    fs::remove_dir_all(&root).ok();
}
