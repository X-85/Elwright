//! 资源工作区 IPC 回归：前端新建资源不带 id，服务端必须能反序列化
//! （v0.1.6 冒烟发现：Resource.id 缺 serde(default) 导致 create_resource
//! 恒报 missing field `id`，浏览器 e2e 走 localStorage 不经此接缝）。
//!
//! 注意：workspace_root 用 user_root()，本测试写真实 ~/.elwright，
//! 结束前删除自己创建的数据。

use serde_json::json;
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;

type Wv = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn build_wv() -> Wv {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            elwright_core::core::commands::workspace_load,
            elwright_core::core::commands::workspace_create_resource,
            elwright_core::core::commands::workspace_delete_resource,
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
        url: "tauri://localhost".parse().unwrap(),
        body: body.into(),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    }
}

fn call(wv: &Wv, cmd: &str, body: serde_json::Value) -> Result<serde_json::Value, String> {
    get_ipc_response(wv, request(cmd, body))
        .map_err(|e| format!("{e:?}"))
        .map(|b| match &b {
            tauri::ipc::InvokeResponseBody::Json(s) => {
                serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
            }
            tauri::ipc::InvokeResponseBody::Raw(bytes) => {
                serde_json::from_slice(bytes).unwrap_or(serde_json::Value::Null)
            }
        })
}

#[test]
fn create_resource_without_id_succeeds() {
    let wv = build_wv();

    // 前端真实负载形态：无 id 字段
    let created = call(
        &wv,
        "workspace_create_resource",
        json!({"resource":{"title":"ipc-regression","kind":"note","value":"临时回归数据","folderId":null,"note":"","launchArgs":[],"icon":""}}),
    )
    .expect("无 id 新建资源应成功");
    let id = created["id"].as_str().expect("服务端应生成 id").to_string();
    assert_eq!(created["title"], "ipc-regression");

    // 清理本测试写入真实数据目录的条目
    call(&wv, "workspace_delete_resource", json!({"id": id})).expect("删除回归数据应成功");

    let data = call(&wv, "workspace_load", json!({})).expect("load 应成功");
    let still_there = data["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == created["id"]);
    assert!(!still_there, "回归数据应已被删除");
}
