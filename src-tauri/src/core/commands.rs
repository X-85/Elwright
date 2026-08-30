//! 桌面壳全部 IPC 命令（Tauri command 层）。
//!
//! 从 main.rs 下沉（enhancement-2026-08-quality-tier2-e2e）：命令层是
//! 前端 bridge 与 shared core 之间的接缝，此前在 bin crate 里 tests/
//! 无法触达、零测试——终端 Bug #1（Channel 参数被自建 no-op 替换）正是
//! 出在这一层。下沉后可用 tauri mock runtime 以真实 IPC 协议测
//! （tests/terminal_ipc.rs）。
//!
//! 状态注入：setup 期构建 [`AppCtx`] 经 `.manage()` 放入 tauri State，
//! 命令从 `State<AppCtx>` 取——替代原 main.rs 的 static OnceLock，
//! mock 测试可注入自定义后端（如可观测的 MockBackend）。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::{Manager, State};

use super::chat_store;
use super::{executor, export, invoke, llm, registry, terminal, version};

/// 桌面壳全局状态：setup 期填充，经 `.manage()` 注入，命令层只读。
pub struct AppCtx {
    /// 资源根（setup 期 resolve_root 一次，含 bundle 资源目录探测）
    pub root: PathBuf,
    /// 终端 session 注册表（后端可注入：生产 LocalBackend，测试 MockBackend）
    pub terminal: Arc<terminal::SessionRegistry>,
}

fn load_registry(ctx: &AppCtx) -> Result<registry::Registry, String> {
    registry::Registry::load(&ctx.root)
}

#[derive(Serialize)]
pub struct ViewDocResult {
    ok: bool,
    content: String,
    path: Option<String>,
}

#[derive(Serialize)]
pub struct RunScriptResult {
    ok: bool,
    output: String,
}

/// list_capabilities 的下发结构：条目 + 来源标记（前端渲染「自定义」徽标）。
#[derive(Serialize)]
pub struct CapabilityWithOrigin {
    #[serde(flatten)]
    cap: registry::Capability,
    origin: registry::Origin,
}

#[tauri::command]
pub fn list_capabilities(ctx: State<AppCtx>) -> Result<Vec<CapabilityWithOrigin>, String> {
    let registry = load_registry(&ctx)?;
    Ok(registry
        .list()
        .iter()
        .map(|c| CapabilityWithOrigin {
            cap: c.clone(),
            origin: registry.origin_of(&c.id),
        })
        .collect())
}

#[tauri::command]
pub fn view_doc(ctx: State<AppCtx>, id: String) -> Result<ViewDocResult, String> {
    let registry = load_registry(&ctx)?;
    let capability = registry
        .get(&id)
        .ok_or_else(|| format!("未找到能力: {}", id))?;
    let relative = capability
        .doc
        .as_ref()
        .or(capability.entry.as_ref())
        .ok_or_else(|| format!("能力 {} 无可查看文档", id))?;
    let path = registry.resolve_resource(relative);
    let content = std::fs::read_to_string(&path)
        .map_err(|error| format!("读取文档 {} 失败: {}", path.display(), error))?;

    Ok(ViewDocResult {
        ok: true,
        content,
        path: Some(relative.clone()),
    })
}

#[tauri::command]
pub async fn run_script<R: tauri::Runtime>(
    ctx: tauri::AppHandle<R>,
    id: String,
    args: Vec<String>,
) -> Result<RunScriptResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let ctx = ctx.state::<AppCtx>();
        let registry = load_registry(&ctx)?;
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
pub async fn invoke_skill<R: tauri::Runtime>(
    ctx: tauri::AppHandle<R>,
    id: String,
    prompt: String,
) -> Result<invoke::InvokeOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let ctx = ctx.state::<AppCtx>();
        let registry = load_registry(&ctx)?;
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
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub update_available: bool,
    pub release_url: String,
}

/// 查询 GitHub 最新 Release，与本应用版本比较。
/// 只在用户点击「检查更新」时调用（不轮询），API 限额无压力。
#[tauri::command]
pub async fn check_update() -> Result<UpdateInfo, String> {
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

// ---- 导入/导出/删除（用户叠加层 ~/.elwright/）----

#[tauri::command]
pub fn import_capability(path: String, force: bool) -> Result<String, String> {
    let overlay = registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("读取 {} 失败: {}", path, e))?;
    export::import_capability(&overlay, &text, force)
}

#[tauri::command]
pub fn export_capability(ctx: State<AppCtx>, id: String, path: String) -> Result<String, String> {
    let registry = load_registry(&ctx)?;
    let bundle = export::export_capability(&registry, &id)?;
    std::fs::write(&path, &bundle).map_err(|e| format!("写入 {} 失败: {}", path, e))?;
    Ok(format!("已导出 {} -> {}", id, path))
}

#[tauri::command]
pub fn delete_capability(ctx: State<AppCtx>, id: String) -> Result<String, String> {
    let registry = load_registry(&ctx)?;
    let overlay = registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    export::delete_capability(&overlay, &registry, &id)
}

// ---- AI 对话（阶段①：多轮非流式）----

/// chat_completion 的入参消息：只接受 user/assistant（system 由 Rust 侧
/// 固定前置，前端无法注入——chat behavior 的安全要求）。
#[derive(Deserialize)]
pub struct ChatMessageArg {
    role: String,
    content: String,
}

/// 多轮对话：合并 LLM 配置链（与 invoke_skill 同链路），前置系统提示词，
/// 非流式返回 assistant 回复。未配置/失败返回中文 Err——对话无降级 SOP，
/// 会话保留与重试由前端负责。
#[tauri::command]
pub async fn chat_completion<R: tauri::Runtime>(
    ctx: tauri::AppHandle<R>,
    messages: Vec<ChatMessageArg>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if messages.is_empty() {
            return Err("消息列表为空".to_string());
        }
        for m in &messages {
            if m.role != "user" && m.role != "assistant" {
                return Err(format!(
                    "不支持的消息角色: {}（仅 user/assistant，system 由应用控制）",
                    m.role
                ));
            }
        }
        let ctx = ctx.state::<AppCtx>();
        let registry = load_registry(&ctx)?;
        let layers = llm::ConfigLayers::collect(&registry.root, registry.llm_default.clone());
        let (config, _) = layers.merged();
        if config.base_url.is_empty() {
            return Err("未配置 LLM：请在「⚙ 模型设置」填写 base_url 后使用 AI 对话".to_string());
        }
        let client = llm::LlmClient { config };
        let mut all = vec![llm::ChatMessage::new("system", llm::CHAT_SYSTEM_PROMPT)];
        all.extend(messages.into_iter().map(|m| llm::ChatMessage {
            role: m.role,
            content: m.content,
        }));
        client.chat_messages(&all)
    })
    .await
    .map_err(|e| format!("对话任务异常: {}", e))?
}

// ---- AI 对话会话存储（阶段②：本地会话管理）----

#[tauri::command]
pub fn chat_list_sessions() -> Result<Vec<chat_store::ChatSessionSummary>, String> {
    Ok(chat_store::list_sessions())
}

#[tauri::command]
pub fn chat_load_session(id: String) -> Result<Option<chat_store::ChatSession>, String> {
    Ok(chat_store::load_session(&id))
}

/// 保存（upsert）。messages 从前端来，role/content 经 chat_completion 已限定
/// 为 user/assistant；updated_at 服务端写。created_at 已存在则保留。
#[tauri::command]
pub fn chat_save_session(
    id: String,
    title: String,
    messages: Vec<ChatMessageArg>,
) -> Result<(), String> {
    let messages: Vec<llm::ChatMessage> = messages
        .into_iter()
        .map(|m| llm::ChatMessage {
            role: m.role,
            content: m.content,
        })
        .collect();
    chat_store::save_session(&id, &title, &messages)
}

#[tauri::command]
pub fn chat_delete_session(id: String) -> Result<(), String> {
    chat_store::delete_session(&id)
}

// ---- LLM 模型设置（读合并视图 / 写用户层 / 连接测试）----

#[tauri::command]
pub fn get_llm_config(ctx: State<AppCtx>) -> Result<llm::LlmConfigView, String> {
    let registry = load_registry(&ctx)?;
    let layers = llm::ConfigLayers::collect(&registry.root, registry.llm_default.clone());
    Ok(layers.view())
}

/// 保存到用户层 ~/.elwright/config.json。
/// apiKey 语义：Some(v) 空串=清除、Some(v) 非空=写入、None=保留现值不改。
/// 环境变量/项目层优先级更高，保存后视图可能仍显示被更高层覆盖的值。
// 参数名保持 camelCase：与前端 invoke 参数对齐（Tauri IPC 约定）
#[tauri::command]
#[allow(non_snake_case)]
pub fn set_llm_config(
    ctx: State<AppCtx>,
    baseUrl: String,
    apiKey: Option<String>,
    model: String,
) -> Result<llm::LlmConfigView, String> {
    // api_key 保留语义：None 时先读用户层现值回填
    let key = match apiKey {
        Some(k) => k,
        None => llm::read_user_api_key(),
    };
    llm::set_user_config(&baseUrl, &key, &model)?;
    let registry = load_registry(&ctx)?;
    let layers = llm::ConfigLayers::collect(&registry.root, registry.llm_default.clone());
    Ok(layers.view())
}

/// 连接测试：用表单当前值（未保存也可测）发最小请求。
// 参数名保持 camelCase：与前端 invoke 参数对齐（Tauri IPC 约定）
#[tauri::command]
#[allow(non_snake_case)]
pub async fn test_llm_connection(
    baseUrl: String,
    apiKey: String,
    model: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || llm::test_connection(&baseUrl, &apiKey, &model))
        .await
        .map_err(|e| format!("连接测试任务异常: {}", e))?
}

// ---- 集成终端 IPC ----

/// 打开新终端会话。channel 由前端创建并作为参数传入（Tauri CommandArg
/// 会把它解析回指向 JS onmessage 回调的 channel）；PTY 输出经 pump 线程
/// 从该 channel 推给前端。
#[tauri::command]
pub async fn terminal_open<R: tauri::Runtime>(
    ctx: tauri::AppHandle<R>,
    cols: u16,
    rows: u16,
    cwd: Option<String>,
    shell: Option<String>,
    env: Option<Vec<(String, String)>>,
    channel: Channel<Vec<u8>>,
) -> Result<u64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let ctx = ctx.state::<AppCtx>();
        let reg = ctx.terminal.clone();
        let backend_shells = reg.default_shells();
        let shell = shell
            .or_else(|| backend_shells.into_iter().next())
            .ok_or_else(|| "未配置默认 shell".to_string())?;
        let cwd_path = cwd
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let env = env.unwrap_or_default();
        let id = reg.open(&shell, &cwd_path, cols, rows, &env, channel)?;
        Ok(id.0)
    })
    .await
    .map_err(|e| format!("终端启动任务异常: {}", e))?
}

#[tauri::command]
pub fn terminal_write(ctx: State<AppCtx>, id: u64, data: Vec<u8>) -> Result<(), String> {
    ctx.terminal.write(terminal::SessionId(id), &data)
}

#[tauri::command]
pub fn terminal_resize(ctx: State<AppCtx>, id: u64, cols: u16, rows: u16) -> Result<(), String> {
    ctx.terminal.resize(terminal::SessionId(id), cols, rows)
}

#[tauri::command]
pub fn terminal_close(ctx: State<AppCtx>, id: u64) -> Result<(), String> {
    ctx.terminal.kill(terminal::SessionId(id));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 回归 v0.1.3 修复：Tauri IPC 返回值不会自动 snake→camel，
    /// 必须显式 #[serde(rename_all = "camelCase")]，否则前端读到 undefined，
    /// 导致「检查更新」按钮永远显示「已是最新版本」。
    /// 用 to_string + from_str 模拟 IPC 在 webview 里 JSON.parse 的路径。
    #[test]
    fn update_info_serializes_camel_case() {
        let info = UpdateInfo {
            current: "0.1.1".to_string(),
            latest: "0.1.2".to_string(),
            update_available: true,
            release_url: "https://github.com/X-85/Elwright/releases/tag/v0.1.2".to_string(),
        };
        let raw = serde_json::to_string(&info).unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(json["current"], "0.1.1");
        assert_eq!(json["latest"], "0.1.2");
        assert_eq!(json["updateAvailable"], true);
        assert!(
            json.get("update_available").is_none(),
            "UpdateInfo 序列化必须输出 updateAvailable，不能有 snake_case 字段（前端 bridge.ts 读 updateAvailable）",
        );
        assert_eq!(
            json["releaseUrl"],
            "https://github.com/X-85/Elwright/releases/tag/v0.1.2"
        );
    }
}
