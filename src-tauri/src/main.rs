use elwright_core::core::{executor, invoke, registry, version};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;
use tauri::Manager;

// setup 期解析一次（含 bundle 资源目录探测），IPC 命令复用
static ROOT: OnceLock<PathBuf> = OnceLock::new();

#[derive(Serialize)]
struct ViewDocResult {
    ok: bool,
    content: String,
    path: Option<String>,
}

#[derive(Serialize)]
struct RunScriptResult {
    ok: bool,
    output: String,
}

#[tauri::command]
fn list_capabilities() -> Result<Vec<registry::Capability>, String> {
    let registry = load_registry()?;
    Ok(registry.list().to_vec())
}

#[tauri::command]
fn view_doc(id: String) -> Result<ViewDocResult, String> {
    let registry = load_registry()?;
    let capability = registry
        .get(&id)
        .ok_or_else(|| format!("未找到能力: {}", id))?;
    let relative = capability
        .doc
        .as_ref()
        .or(capability.entry.as_ref())
        .ok_or_else(|| format!("能力 {} 无可查看文档", id))?;
    let path = registry.root.join(relative);
    let content = std::fs::read_to_string(&path)
        .map_err(|error| format!("读取文档 {} 失败: {}", path.display(), error))?;

    Ok(ViewDocResult {
        ok: true,
        content,
        path: Some(relative.clone()),
    })
}

#[tauri::command]
async fn run_script(id: String, args: Vec<String>) -> Result<RunScriptResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let registry = load_registry()?;
        let capability = registry
            .get(&id)
            .ok_or_else(|| format!("未找到能力: {}", id))?;
        if capability.kind != "script" {
            return Err(format!(
                "{} 不是脚本型能力（type={}），无法 run",
                id, capability.kind
            ));
        }
        let entry = registry
            .resolve_entry(capability)
            .ok_or_else(|| format!("能力 {} 缺少 entry 字段", id))?;
        if !entry.exists() {
            return Err(format!("脚本不存在: {}", entry.display()));
        }

        let result = executor::run_script_capture(&entry, &args)?;
        let mut output = result.output;
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&format!("退出码: {}", result.code));
        Ok(RunScriptResult {
            ok: result.code == 0,
            output,
        })
    })
    .await
    .map_err(|error| format!("脚本执行任务异常: {}", error))?
}

#[tauri::command]
async fn invoke_skill(id: String, prompt: String) -> Result<invoke::InvokeOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let registry = load_registry()?;
        let capability = registry
            .get(&id)
            .ok_or_else(|| format!("未找到能力: {}", id))?;
        if capability.kind != "skill" {
            return Err(format!(
                "{} 不是技能型能力（type={}），无法 invoke",
                id, capability.kind
            ));
        }
        Ok(invoke::invoke_skill(&registry, capability, &prompt))
    })
    .await
    .map_err(|error| format!("技能调用任务异常: {}", error))?
}

#[derive(Serialize)]
struct UpdateInfo {
    current: String,
    latest: String,
    update_available: bool,
    release_url: String,
}

/// 查询 GitHub 最新 Release，与本应用版本比较。
/// 只在用户点击「检查更新」时调用（不轮询），API 限额无压力。
#[tauri::command]
async fn check_update() -> Result<UpdateInfo, String> {
    tauri::async_runtime::spawn_blocking(|| {
        #[derive(Deserialize)]
        struct GhRelease {
            tag_name: String,
            html_url: String,
        }
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
        let release: GhRelease = client
            .get("https://api.github.com/repos/X-85/Elwright/releases/latest")
            .header("User-Agent", "Elwright-update-check")
            .send()
            .map_err(|e| format!("检查更新失败：无法访问 GitHub（离网或网络受限）: {}", e))?
            .error_for_status()
            .map_err(|e| format!("检查更新失败：GitHub 返回错误: {}", e))?
            .json()
            .map_err(|e| format!("解析版本信息失败: {}", e))?;
        let current = env!("CARGO_PKG_VERSION").to_string();
        Ok(UpdateInfo {
            update_available: version::is_newer(&release.tag_name, &current),
            latest: version::normalize(&release.tag_name).to_string(),
            current,
            release_url: release.html_url,
        })
    })
    .await
    .map_err(|e| format!("更新检查任务异常: {}", e))?
}

fn load_registry() -> Result<registry::Registry, String> {
    let root = ROOT
        .get()
        .cloned()
        .unwrap_or_else(|| registry::resolve_root(&[]));
    registry::Registry::load(&root)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let resource_dir = app
                .path()
                .resource_dir()
                .map_err(|e| format!("定位资源目录失败: {}", e))?;
            let root = registry::resolve_root(&[resource_dir]);
            let _ = ROOT.set(root);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_capabilities,
            view_doc,
            run_script,
            invoke_skill,
            check_update
        ])
        .run(tauri::generate_context!())
        .expect("启动 Elwright 桌面应用失败");
}
