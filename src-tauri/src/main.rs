use elwright_core::core::{executor, invoke, registry};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::OnceLock;
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

fn load_registry() -> Result<registry::Registry, String> {
    let root = ROOT
        .get()
        .cloned()
        .unwrap_or_else(|| registry::resolve_root(&[]));
    registry::Registry::load(&root)
}

fn main() {
    tauri::Builder::default()
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
            invoke_skill
        ])
        .run(tauri::generate_context!())
        .expect("启动 Elwright 桌面应用失败");
}
