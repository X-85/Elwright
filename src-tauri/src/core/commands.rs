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

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::{Manager, State};

use super::chat_store;
use super::code_browser;
use super::workbench;
use super::{executor, export, invoke, llm, patch, registry, terminal, version, workspace};

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
        let mut all = vec![llm::ChatMessage::new(
            "system",
            chat_system_prompt(registry.list()),
        )];
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

// ---- 工作工具栏（Workbench 第一阶段：Todo + 今日记录）----

#[tauri::command]
pub fn todo_list() -> Result<Vec<workbench::TodoItem>, String> {
    Ok(workbench::todo_list())
}

#[tauri::command]
pub fn todo_add(text: String) -> Result<workbench::TodoItem, String> {
    workbench::todo_add(&text)
}

#[tauri::command]
pub fn todo_toggle(id: u64) -> Result<workbench::TodoItem, String> {
    workbench::todo_toggle(id)
}

#[tauri::command]
pub fn todo_remove(id: u64) -> Result<(), String> {
    workbench::todo_remove(id)
}

/// 读某日记录；无记录返回 null（前端显示空编辑器）。
#[tauri::command]
pub fn note_get(date: String) -> Result<Option<String>, String> {
    workbench::note_get(&date)
}

#[tauri::command]
pub fn note_save(date: String, content: String) -> Result<(), String> {
    workbench::note_save(&date, &content)
}

/// 已有记录的日期列表（倒序，最近在前）。
#[tauri::command]
pub fn note_list() -> Result<Vec<String>, String> {
    Ok(workbench::note_list_dates())
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

// ---- 资源管理与课题工作区（feature-2026-08-progressive-capabilities /
// enhancement-2026-08-software-shortcuts）。数据层在 core::workspace，
// 根目录用用户主目录（~/.elwright/workspace），与资源根 ctx.root 无关。

fn workspace_root() -> Result<PathBuf, String> {
    registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())
}

#[derive(Serialize)]
pub struct TopicReportResult {
    source: String,
    content: String,
    note: Option<String>,
}

#[tauri::command]
pub fn workspace_load() -> Result<workspace::WorkspaceData, String> {
    workspace::load(&workspace_root()?)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn workspace_create_folder(
    name: String,
    parentId: Option<String>,
) -> Result<workspace::Folder, String> {
    workspace::create_folder(&workspace_root()?, &name, parentId)
}

#[tauri::command]
pub fn workspace_delete_folder(id: String) -> Result<(), String> {
    workspace::delete_folder(&workspace_root()?, &id)
}

#[tauri::command]
pub fn workspace_create_resource(
    resource: workspace::Resource,
) -> Result<workspace::Resource, String> {
    workspace::create_resource(&workspace_root()?, resource)
}

#[tauri::command]
pub fn workspace_delete_resource(id: String) -> Result<(), String> {
    workspace::delete_resource(&workspace_root()?, &id)
}

#[tauri::command]
pub fn workspace_launch_app(id: String) -> Result<String, String> {
    workspace::launch_app(&workspace_root()?, &id)?;
    Ok("软件已启动".to_string())
}

#[tauri::command]
pub fn workspace_create_topic(title: String, question: String) -> Result<workspace::Topic, String> {
    workspace::create_topic(&workspace_root()?, &title, &question)
}

#[tauri::command]
pub fn workspace_update_topic(topic: workspace::Topic) -> Result<(), String> {
    workspace::update_topic(&workspace_root()?, topic)
}

#[tauri::command]
pub fn workspace_delete_topic(id: String) -> Result<(), String> {
    workspace::delete_topic(&workspace_root()?, &id)
}

#[tauri::command]
pub async fn workspace_generate_report<R: tauri::Runtime>(
    ctx: tauri::AppHandle<R>,
    id: String,
) -> Result<TopicReportResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let ctx = ctx.state::<AppCtx>();
        let root = workspace_root()?;
        let mut data = workspace::load(&root)?;
        let topic = data
            .topics
            .iter()
            .find(|t| t.id == id)
            .cloned()
            .ok_or_else(|| "课题不存在".to_string())?;
        let mut source = String::new();
        for rid in &topic.resource_ids {
            if let Some(resource) = data.resources.iter().find(|r| r.id == *rid) {
                source.push_str(&format!(
                    "\n### {} [{}]\n{}\n",
                    resource.title, resource.kind, resource.value
                ));
                if resource.kind == "path" {
                    if let Ok(text) = std::fs::read_to_string(&resource.value) {
                        source.push_str(&text.chars().take(6000).collect::<String>());
                        source.push('\n');
                    }
                }
                if !resource.note.trim().is_empty() {
                    source.push_str(&format!("备注：{}\n", resource.note));
                }
            }
        }
        let user_prompt = format!(
            "请围绕课题《{}》生成一份完整、有深度、可执行的研究报告。\n研究问题：{}\n相关资源：{}\n要求：先给结论摘要，再给概念框架、证据与引用、实践步骤、风险和后续问题；不要虚构资源中没有的事实。",
            topic.title,
            topic.question,
            if source.is_empty() { "（暂无资源）" } else { &source }
        );
        let registry = load_registry(&ctx)?;
        let layers = llm::ConfigLayers::collect(&registry.root, registry.llm_default.clone());
        let (config, _) = layers.merged();
        let result = if config.base_url.is_empty() {
            TopicReportResult {
                source: "offline".into(),
                content: offline_report(&topic, &source),
                note: Some("未配置 LLM，已生成离线报告草稿。".into()),
            }
        } else {
            match (llm::LlmClient { config }).chat_messages(&[
                llm::ChatMessage::new("system", "你是严谨的研究助理，输出结构化中文 Markdown 报告。"),
                llm::ChatMessage::new("user", &user_prompt),
            ]) {
                Ok(content) => TopicReportResult { source: "llm".into(), content, note: None },
                Err(error) => TopicReportResult {
                    source: "offline".into(),
                    content: offline_report(&topic, &source),
                    note: Some(format!("LLM 调用失败，已降级为离线草稿：{}", error)),
                },
            }
        };
        if let Some(saved) = data.topics.iter_mut().find(|t| t.id == topic.id) {
            saved.report = result.content.clone();
            saved.updated_at = format!("{}", chrono_like_now());
            workspace::save(&root, &data)?;
        }
        Ok(result)
    })
    .await
    .map_err(|e| format!("报告生成任务异常: {}", e))?
}

fn chrono_like_now() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn offline_report(topic: &workspace::Topic, source: &str) -> String {
    let resources = if source.is_empty() {
        "暂无已关联资源。"
    } else {
        source
    };
    format!(
        "# {}\n\n## 研究问题\n{}\n\n## 当前资料\n{}\n## 分析框架\n1. 明确核心概念与边界。\n2. 对照资料中的事实、示例和限制。\n3. 将结论拆解为可验证的实践步骤。\n\n## 待补充\n- 为每个关键判断补充原始出处与反例。\n- 用实际案例验证结论，并记录版本与环境。\n\n> 这是离线报告草稿。配置 LLM 后可再次生成完整研究报告。",
        topic.title,
        if topic.question.is_empty() { "（未填写）" } else { &topic.question },
        resources
    )
}

// ---- 代码浏览器阶段①（feature-2026-08-code-browser-phase1）----
// 只读：目录树 / 文件读取 / 搜索 / 轻量符号扫描。projectRoot 由用户经
// 系统目录选择器主动提供，core 侧做路径边界与大小限制。

fn project_root(root: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(root);
    if !p.is_dir() {
        return Err(format!("项目根不是目录: {root}"));
    }
    Ok(p)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn code_browser_tree(
    projectRoot: String,
    rel: String,
) -> Result<Vec<code_browser::TreeEntry>, String> {
    code_browser::tree(&project_root(&projectRoot)?, &rel)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn code_browser_read(
    projectRoot: String,
    rel: String,
) -> Result<code_browser::CodeDocument, String> {
    code_browser::read_file(&project_root(&projectRoot)?, &rel)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn code_browser_search(
    projectRoot: String,
    query: String,
    mode: String,
) -> Result<Vec<code_browser::SearchHit>, String> {
    code_browser::search(&project_root(&projectRoot)?, &query, &mode)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn code_browser_scan_symbols(
    projectRoot: String,
) -> Result<Vec<code_browser::SymbolHit>, String> {
    code_browser::scan_symbols(&project_root(&projectRoot)?)
}

/// 读取最近项目/文件（用户配置层 ~/.elwright/code-browser.json）。
#[tauri::command]
pub fn code_browser_recent_load() -> Result<code_browser::RecentStore, String> {
    let user = registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    Ok(code_browser::load_recent(&user))
}

/// 记录一次「打开项目 / 打开文件」，返回更新后的最近列表。
#[tauri::command]
#[allow(non_snake_case)]
pub fn code_browser_recent_open(
    projectRoot: String,
    rel: String,
) -> Result<code_browser::RecentStore, String> {
    let user = registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    let mut store = code_browser::load_recent(&user);
    let now = chrono_like_now();
    let name = Path::new(&projectRoot)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| projectRoot.clone());
    code_browser::push_recent_project(
        &mut store,
        code_browser::RecentProject {
            name,
            root_path: projectRoot.clone(),
            last_opened_at: now as u64,
        },
    );
    if !rel.is_empty() {
        code_browser::push_recent_file(
            &mut store,
            code_browser::RecentFile {
                project_root: projectRoot,
                path: rel,
                last_opened_at: now as u64,
            },
        );
    }
    code_browser::save_recent(&user, &store)?;
    Ok(store)
}

/// 切换收藏文件，返回更新后的收藏列表。
#[allow(non_snake_case)]
#[tauri::command]
pub fn code_browser_favorites_toggle(
    projectRoot: String,
    rel: String,
) -> Result<Vec<code_browser::Favorite>, String> {
    let user = registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    let mut store = code_browser::load_recent(&user);
    code_browser::toggle_favorite(&mut store, &projectRoot, &rel)?;
    code_browser::save_recent(&user, &store)?;
    Ok(store.favorites)
}

/// 切换代码书签，返回更新后的书签列表。
#[allow(non_snake_case)]
#[tauri::command]
pub fn code_browser_bookmarks_toggle(
    projectRoot: String,
    rel: String,
    line: u32,
    label: String,
) -> Result<Vec<code_browser::Bookmark>, String> {
    let user = registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    let mut store = code_browser::load_recent(&user);
    code_browser::toggle_bookmark(&mut store, &projectRoot, &rel, line, &label)?;
    code_browser::save_recent(&user, &store)?;
    Ok(store.bookmarks)
}

// ---- 代码浏览器阶段④：受控补丁编辑（ADR-001）----

/// 解析 unified diff 文本并预览（不写文件）。
#[allow(non_snake_case)]
#[tauri::command]
pub fn apply_patch_preview(
    projectRoot: String,
    patchText: String,
) -> Result<patch::PatchPreview, String> {
    let root = project_root(&projectRoot)?;
    let parsed = patch::parse_unified_diff(&patchText)?;
    patch::build_preview(&root, &parsed)
}

/// 应用预览（写入文件 + 落快照）。`previews` 由前端三栏对话框逐 hunk 选择后回传。
#[allow(non_snake_case)]
#[tauri::command]
pub fn apply_patch_apply(
    projectRoot: String,
    previews: Vec<patch::PatchFilePreview>,
) -> Result<patch::ApplyResult, String> {
    let root = project_root(&projectRoot)?;
    let user = registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    patch::apply_preview(&root, &previews, &user, &projectRoot)
}

/// 撤销一次已应用的补丁：按快照 ID 把原始内容写回。
#[allow(non_snake_case)]
#[tauri::command]
pub fn apply_patch_revert(
    projectRoot: String,
    snapshotId: String,
) -> Result<patch::RevertResult, String> {
    let root = project_root(&projectRoot)?;
    let user = registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    patch::revert_snapshot_in(&root, &user, &snapshotId)
}

/// 加载当前项目的所有未撤销快照（UI 撤销列表用）。
#[allow(non_snake_case)]
#[tauri::command]
pub fn apply_patch_snapshots(projectRoot: String) -> Result<Vec<patch::PatchSnapshot>, String> {
    let _ = project_root(&projectRoot)?;
    let user = registry::user_root().ok_or_else(|| "无法定位用户主目录".to_string())?;
    let path = patch::snapshot_path(&user)?;
    Ok(patch::load_snapshots(&path).unwrap_or_default())
}

// ----- Q19 设置中心：模型档案（多套 LLM 配置切换） -----

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfileMetaDto {
    pub name: String,
    pub active: bool,
    pub source: &'static str,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LlmProfileDto {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl From<LlmProfileDto> for llm::LlmProfile {
    fn from(v: LlmProfileDto) -> Self {
        Self {
            name: v.name,
            base_url: v.base_url,
            api_key: v.api_key,
            model: v.model,
        }
    }
}

#[tauri::command]
pub fn llm_list_profiles() -> Vec<ProfileMetaDto> {
    llm::list_profiles()
        .into_iter()
        .map(|m| ProfileMetaDto {
            name: m.name,
            active: m.active,
            source: m.source,
        })
        .collect()
}

#[tauri::command]
pub fn llm_get_active_profile() -> Option<String> {
    llm::active_profile_name()
}

#[tauri::command]
pub fn llm_set_active_profile(name: String) -> Result<(), String> {
    llm::set_active_profile(&name)
}

#[tauri::command]
pub fn llm_save_profile(profile: LlmProfileDto) -> Result<(), String> {
    llm::save_profile(profile.into())
}

#[tauri::command]
pub fn llm_delete_profile(name: String) -> Result<(), String> {
    llm::delete_profile(&name)
}

/// AI 对话系统提示：基础提示 + 能力清单（阶段③能力协作）。
/// 模型只能提议（固定格式），执行永远在用户确认后由前端走既有 run/view/invoke 路径。
fn chat_system_prompt(caps: &[registry::Capability]) -> String {
    let mut sys = String::from(llm::CHAT_SYSTEM_PROMPT);
    let mut lines: Vec<String> = Vec::new();
    for cap in caps {
        lines.push(format!("- {}（{}）{}", cap.id, cap.kind, cap.name));
    }
    if !lines.is_empty() {
        sys.push_str("\n\n你可以向用户提议使用以下本地能力。仅提议，用户确认后才会执行；提议必须严格使用单独一行格式（不要包代码块）：【能力提议】id: <id>。可用能力：\n");
        sys.push_str(&lines.join("\n"));
    }
    sys
}

// ---- AI 对话阶段④：流式输出与请求级取消（ADR-003）----

fn chat_cancels() -> &'static std::sync::Mutex<std::collections::HashSet<u64>> {
    static SET: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<u64>>> =
        std::sync::OnceLock::new();
    SET.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()))
}

fn is_chat_cancelled(request_id: u64) -> bool {
    chat_cancels().lock().unwrap().contains(&request_id)
}

/// 取消一个在途流式请求：流式循环逐块检查取消表，命中即中断读取。
#[tauri::command]
pub fn chat_cancel(request_id: u64) -> Result<(), String> {
    chat_cancels().lock().unwrap().insert(request_id);
    Ok(())
}

/// 流式对话：增量经 Channel 推 JSON 事件（delta/done/error/cancelled）。
/// 消息角色与 chat_completion 同规则校验；系统提示同 chat_system_prompt。
#[allow(non_snake_case)]
#[tauri::command]
pub async fn chat_completion_stream<R: tauri::Runtime>(
    ctx: tauri::AppHandle<R>,
    request_id: u64,
    messages: Vec<ChatMessageArg>,
    channel: tauri::ipc::Channel<tauri::ipc::InvokeResponseBody>,
) -> Result<(), String> {
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
    tauri::async_runtime::spawn_blocking(move || {
        let send = |event: &str, text: &str| {
            let payload = format!(
                "{{\"type\":\"{event}\",\"text\":{}}}",
                serde_json::to_string(text).unwrap_or_else(|_| "null".into())
            );
            let _ = channel.send(tauri::ipc::InvokeResponseBody::Json(payload.to_string()));
        };
        let send_error = |message: &str| {
            let payload = format!(
                "{{\"type\":\"error\",\"message\":{}}}",
                serde_json::to_string(message).unwrap_or_else(|_| "null".into())
            );
            let _ = channel.send(tauri::ipc::InvokeResponseBody::Json(payload.to_string()));
        };

        let ctx = ctx.state::<AppCtx>();
        let registry = load_registry(&ctx)?;
        let layers = llm::ConfigLayers::collect(&registry.root, registry.llm_default.clone());
        let (config, _) = layers.merged();
        if config.base_url.is_empty() {
            send_error("未配置 LLM：请在「⚙ 模型设置」填写 base_url 后使用 AI 对话");
            return Ok(());
        }
        let client = llm::LlmClient { config };
        let sys = chat_system_prompt(registry.list());
        let mut all = vec![llm::ChatMessage::new("system", sys)];
        all.extend(messages.into_iter().map(|m| llm::ChatMessage {
            role: m.role,
            content: m.content,
        }));

        let outcome =
            client.chat_messages_streaming(&all, request_id, is_chat_cancelled, |delta| {
                send("delta", delta);
            });
        chat_cancels().lock().unwrap().remove(&request_id);

        match outcome {
            Ok(o) if o.cancelled => send("cancelled", ""),
            Ok(o) if o.text.trim().is_empty() => {
                // 供应商不支持流式 / 格式不兼容：按 ADR-003 回退非流式
                match client.chat_messages(&all) {
                    Ok(text) => {
                        send("delta", &text);
                        send("done", "");
                    }
                    Err(e) => send_error(&e),
                }
            }
            Ok(_) => send("done", ""),
            Err(e) => send_error(&e),
        }
        Ok(())
    })
    .await
    .map_err(|e| format!("流式任务异常: {}", e))?
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

#[cfg(test)]
mod chat_prompt_tests {
    use super::*;

    fn cap(id: &str, kind: &str, name: &str) -> registry::Capability {
        registry::Capability {
            id: id.into(),
            name: name.into(),
            kind: kind.into(),
            category: Some("示例".into()),
            entry: None,
            doc: None,
            offline: Some(true),
            prompt: None,
            degrade_doc: None,
            release_tier: 1,
            unlock_after_uses: None,
        }
    }

    #[test]
    fn chat_system_prompt_lists_capabilities_and_proposal_format() {
        let caps = vec![cap("text-stats", "script", "文本统计")];
        let sys = chat_system_prompt(&caps);
        assert!(sys.contains(llm::CHAT_SYSTEM_PROMPT));
        assert!(sys.contains("- text-stats（script）文本统计"));
        assert!(sys.contains("【能力提议】id: <id>"), "必须约定提议格式");
        let empty = chat_system_prompt(&[]);
        assert!(!empty.contains("【能力提议】"), "空注册表不注入提议约定");
    }
}
