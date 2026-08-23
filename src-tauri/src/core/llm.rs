use std::path::{Path, PathBuf};
use std::time::Duration;

/// LLM client configuration.
///
/// Thin self-written client against any OpenAI-compatible endpoint
/// (cloud, Ollama, llama.cpp, ...). Config comes from environment variables:
/// `ELWRIGHT_LLM_BASE_URL`, `ELWRIGHT_LLM_API_KEY`, `ELWRIGHT_LLM_MODEL`.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct LlmConfig {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model: String,
}

/// 用户级配置文件路径（`~/.elwright/config.json`；Windows 用 USERPROFILE）。
/// `ELWRIGHT_USER_ROOT` 可覆盖（与叠加层同开关，测试/排障用）。
pub fn user_config_path() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("ELWRIGHT_USER_ROOT") {
        return Some(PathBuf::from(custom).join("config.json"));
    }
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|h| PathBuf::from(h).join(".elwright").join("config.json"))
}

/// 解析单个配置文件（flat: base_url/model/api_key）。
fn read_config_file(path: &Path) -> Option<LlmConfig> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text)
        .map_err(|e| {
            eprintln!("警告: 配置文件 {} 解析失败: {}（已忽略）", path.display(), e);
            e
        })
        .ok()
}

/// 字段级合并的配置来源（高 → 低）。
pub struct ConfigLayers {
    pub env: LlmConfig,
    pub project: Option<(PathBuf, LlmConfig)>,
    pub user: Option<(PathBuf, LlmConfig)>,
    pub registry_default: Option<LlmConfig>,
}

impl ConfigLayers {
    pub fn collect(root: &Path, registry_default: Option<LlmConfig>) -> Self {
        let env = LlmConfig {
            base_url: std::env::var("ELWRIGHT_LLM_BASE_URL").unwrap_or_default(),
            api_key: std::env::var("ELWRIGHT_LLM_API_KEY").unwrap_or_default(),
            model: std::env::var("ELWRIGHT_LLM_MODEL").unwrap_or_default(),
        };
        let project_path = root.join("config.local.json");
        let project = read_config_file(&project_path).map(|c| (project_path, c));
        let user = user_config_path().and_then(|p| read_config_file(&p).map(|c| (p, c)));
        Self { env, project, user, registry_default }
    }

    /// 每个字段取最高优先级的非空值；返回 (合并结果, 每字段来源标签)。
    pub fn merged(&self) -> (LlmConfig, [String; 3]) {
        let empty = LlmConfig::default();
        let mut cfg = LlmConfig::default();
        let mut source: [String; 3] = std::array::from_fn(|_| "未设置".to_string());
        let layers: [(&str, &LlmConfig); 4] = [
            ("环境变量", &self.env),
            ("项目 config.local.json", self.project.as_ref().map(|(_, c)| c).unwrap_or(&empty)),
            ("用户 ~/.elwright/config.json", self.user.as_ref().map(|(_, c)| c).unwrap_or(&empty)),
            ("注册表默认 $meta.llmDefault", self.registry_default.as_ref().unwrap_or(&empty)),
        ];
        for (label, layer) in layers {
            if cfg.base_url.is_empty() && !layer.base_url.is_empty() {
                cfg.base_url = layer.base_url.clone();
                source[0] = label.to_string();
            }
            if cfg.api_key.is_empty() && !layer.api_key.is_empty() {
                cfg.api_key = layer.api_key.clone();
                source[1] = label.to_string();
            }
            if cfg.model.is_empty() && !layer.model.is_empty() {
                cfg.model = layer.model.clone();
                source[2] = label.to_string();
            }
        }
        (cfg, source)
    }
}

#[derive(Debug, Default)]
pub struct LlmClient {
    pub config: LlmConfig,
}

/// 多轮对话消息（OpenAI 兼容 role/content）。
/// system 角色由应用侧控制（AI 对话页固定用 CHAT_SYSTEM_PROMPT），不经前端传入。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: &str, content: impl Into<String>) -> Self {
        Self {
            role: role.to_string(),
            content: content.into(),
        }
    }
}

/// AI 对话页的系统提示词：由应用控制，用户输入不可覆盖（chat behavior 的安全要求）。
pub const CHAT_SYSTEM_PROMPT: &str = "你是 Elwright 桌面应用内置的 AI 助手，帮助用户解答问题、整理思路和起草文本。\
用用户提问的语言回答。回复可使用 Markdown；输出中的命令与代码仅供参考，不会被自动执行。";

/// LLM 配置的生效视图：合并结果 + 每字段来源标签 + 用户层原文。
/// 桌面设置界面与 `ew config` 共用；api_key 下发前打码（不回传明文）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmConfigView {
    pub base_url: String,
    pub model: String,
    pub api_key_masked: String,
    /// 每字段来源标签：[base_url, api_key, model]
    pub source: [String; 3],
    /// 用户层文件路径（保存时写这里；None = 找不到主目录）
    pub user_config_path: Option<String>,
}

/// api_key 显示打码：保留前 4 位，短值全遮。
fn mask_key(key: &str) -> String {
    if key.is_empty() {
        String::new()
    } else if key.chars().count() > 8 {
        let head: String = key.chars().take(4).collect();
        format!("{}****", head)
    } else {
        "****".to_string()
    }
}

impl ConfigLayers {
    /// 汇总为下发视图（api_key 打码）。
    pub fn view(&self) -> LlmConfigView {
        let (cfg, source) = self.merged();
        LlmConfigView {
            base_url: cfg.base_url,
            model: cfg.model,
            api_key_masked: mask_key(&cfg.api_key),
            source,
            user_config_path: user_config_path().map(|p| p.display().to_string()),
        }
    }
}

/// 读用户层当前 api_key（明文，仅供「保存时保留现值」语义内部使用，不下发前端）。
pub fn read_user_api_key() -> String {
    user_config_path()
        .and_then(|p| read_config_file(&p))
        .map(|c| c.api_key)
        .unwrap_or_default()
}

/// 保存到用户层 `~/.elwright/config.json`：字段级合并（空值 = 清除该字段），
/// 保留文件里其他未知键。返回保存后的生效视图。
/// 桌面设置界面与 `ew config set`（非 --local）共用同一落盘逻辑。
pub fn set_user_config(base_url: &str, api_key: &str, model: &str) -> Result<(), String> {
    let path = user_config_path().ok_or("无法定位用户主目录")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录 {} 失败: {}", parent.display(), e))?;
    }
    let mut value: serde_json::Map<String, serde_json::Value> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    for (key, new) in [("base_url", base_url), ("api_key", api_key), ("model", model)] {
        if new.is_empty() {
            value.remove(key);
        } else {
            value.insert(key.to_string(), serde_json::Value::String(new.to_string()));
        }
    }
    let text = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&path, text + "\n")
        .map_err(|e| format!("写入 {} 失败: {}", path.display(), e))?;
    Ok(())
}

/// 连接测试：向 base_url 发一条 1 token 上限的最小请求。
/// 走完整鉴权与响应解析，能真实反映 invoke 是否可用。
pub fn test_connection(base_url: &str, api_key: &str, model: &str) -> Result<String, String> {
    // 最小请求：max_tokens=1 + 一字提示，探测成本最低
    let url = chat_url(base_url);
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "ping"}],
        "max_tokens": 1,
    });
    let http = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    let mut req = http.post(&url).json(&body);
    if !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }
    let resp = req
        .send()
        .map_err(|e| format!("无法连接 {}: {}", url, e))?;
    let status = resp.status();
    let text = resp.text().map_err(|e| format!("读取响应失败: {}", e))?;
    if !status.is_success() {
        return Err(format!(
            "端点返回 {}: {}",
            status.as_u16(),
            truncate(&text, 300)
        ));
    }
    // 只要求是合法 JSON 响应（choices 结构即可），不比对内容
    #[derive(serde::Deserialize)]
    struct AnyChat {
        #[serde(default)]
        choices: Vec<serde_json::Value>,
    }
    serde_json::from_str::<AnyChat>(&text)
        .map(|c| if c.choices.is_empty() {
            Err(format!("响应缺少 choices: {}", truncate(&text, 200)))
        } else {
            Ok(format!("连接正常（{}，model={}）", url, model))
        })
        .unwrap_or_else(|e| Err(format!("响应不是 OpenAI 兼容格式: {}（响应: {}）", e, truncate(&text, 200))))
}

/// Join a base_url (e.g. `http://localhost:11434/v1`) with the
/// chat-completions path, tolerating trailing slashes.
pub fn chat_url(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

impl LlmClient {
    /// Build from environment variables; returns None when no base_url is set.
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("ELWRIGHT_LLM_BASE_URL").ok()?;
        let api_key = std::env::var("ELWRIGHT_LLM_API_KEY").unwrap_or_default();
        let model = std::env::var("ELWRIGHT_LLM_MODEL").unwrap_or_default();
        Some(Self {
            config: LlmConfig {
                base_url,
                api_key,
                model,
            },
        })
    }

    /// Send a chat completion request. `system` is the capability's prompt
    /// template, `user` is the caller-supplied input.
    ///
    /// Any transport or protocol failure is returned as Err so the caller
    /// can degrade to the offline SOP instead of crashing.
    pub fn chat(&self, system: &str, user: &str) -> Result<String, String> {
        self.chat_messages(&[
            ChatMessage::new("system", system),
            ChatMessage::new("user", user),
        ])
    }

    /// 多轮对话请求：调用方控制完整消息列表（桌面 AI 对话页前置
    /// CHAT_SYSTEM_PROMPT 后传入）。传输/协议失败以 Err 返回。
    pub fn chat_messages(&self, messages: &[ChatMessage]) -> Result<String, String> {
        let url = chat_url(&self.config.base_url);
        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
        });

        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

        let mut req = http.post(&url).json(&body);
        if !self.config.api_key.is_empty() {
            req = req.bearer_auth(&self.config.api_key);
        }

        let resp = req
            .send()
            .map_err(|e| format!("请求 {} 失败: {}", url, e))?;
        let status = resp.status();
        let text = resp.text().map_err(|e| format!("读取响应失败: {}", e))?;

        if !status.is_success() {
            return Err(format!(
                "LLM 返回 {}: {}",
                status.as_u16(),
                truncate(&text, 500)
            ));
        }

        #[derive(serde::Deserialize)]
        struct ChatResponse {
            choices: Vec<Choice>,
        }
        #[derive(serde::Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(serde::Deserialize)]
        struct Message {
            content: String,
        }

        let parsed: ChatResponse = serde_json::from_str(&text)
            .map_err(|e| format!("解析 LLM 响应失败: {}（响应: {}）", e, truncate(&text, 200)))?;
        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| "LLM 响应中没有 choices".to_string())
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{}…", cut)
    }
}

#[cfg(test)]
mod tests {
    use super::{chat_url, mask_key, set_user_config, ConfigLayers, LlmConfig};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;

    // set_user_config/user_config_path 读进程级环境变量，测试间需串行
    static USER_ROOT_LOCK: Mutex<()> = Mutex::new(());

    fn temp_user_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "elwright-usercfg-{}-{}",
            tag,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn joins_chat_completions_path() {
        assert_eq!(
            chat_url("http://localhost:11434/v1"),
            "http://localhost:11434/v1/chat/completions"
        );
    }

    #[test]
    fn tolerates_trailing_slash() {
        assert_eq!(
            chat_url("https://api.example.com/v1/"),
            "https://api.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn works_without_version_path() {
        assert_eq!(
            chat_url("http://10.0.0.5:8000"),
            "http://10.0.0.5:8000/chat/completions"
        );
    }

    #[test]
    fn merges_fieldwise_with_priority() {
        let _guard = USER_ROOT_LOCK.lock().unwrap();
        // 清掉可能存在的环境变量，保证优先级测试不受宿主环境影响
        for k in ["ELWRIGHT_LLM_BASE_URL", "ELWRIGHT_LLM_API_KEY", "ELWRIGHT_LLM_MODEL", "ELWRIGHT_USER_ROOT"] {
            std::env::remove_var(k);
        }
        let dir = std::env::temp_dir().join(format!("elwright-cfg-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let project_path = dir.join("config.local.json");
        fs::write(
            &project_path,
            r#"{"base_url":"http://project:1/v1","model":"proj-model"}"#,
        )
        .unwrap();
        // 直接构造 ConfigLayers 把 user 设为 None，避免读到宿主真实 ~/.elwright/config.json
        let layers = ConfigLayers {
            env: LlmConfig::default(),
            project: Some((
                project_path,
                LlmConfig {
                    base_url: "http://project:1/v1".into(),
                    api_key: String::new(),
                    model: "proj-model".into(),
                },
            )),
            user: None,
            registry_default: Some(LlmConfig {
                base_url: "http://default:2/v1".into(),
                api_key: "reg-key".into(),
                model: "reg-model".into(),
            }),
        };
        // 项目文件覆盖 base_url/model，api_key 字段级回退到注册表默认
        let (cfg, source) = layers.merged();
        assert_eq!(cfg.base_url, "http://project:1/v1");
        assert_eq!(cfg.model, "proj-model");
        assert_eq!(cfg.api_key, "reg-key");
        assert_eq!(source[0], "项目 config.local.json");
        assert_eq!(source[1], "注册表默认 $meta.llmDefault");

        // 无任何配置时全空（直接构造避免读宿主 ~/.elwright/config.json）
        let empty_dir = std::env::temp_dir().join(format!("elwright-cfg-empty-{}", std::process::id()));
        fs::create_dir_all(&empty_dir).unwrap();
        let empty_layers = ConfigLayers {
            env: LlmConfig::default(),
            project: None,
            user: None,
            registry_default: None,
        };
        let (cfg2, source2) = empty_layers.merged();
        assert!(cfg2.base_url.is_empty());
        assert_eq!(source2[0], "未设置");
        fs::remove_dir_all(&dir).ok();
        fs::remove_dir_all(&empty_dir).ok();
    }

    #[test]
    fn set_user_config_merges_and_clears_fieldwise() {
        let _guard = USER_ROOT_LOCK.lock().unwrap();
        let root = temp_user_root("merge");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);

        // 首次保存三字段
        set_user_config("http://llm:9/v1", "sk-12345678", "qwen3:8b").unwrap();
        let first: LlmConfig =
            serde_json::from_str(&fs::read_to_string(root.join("config.json")).unwrap()).unwrap();
        assert_eq!(first.base_url, "http://llm:9/v1");
        assert_eq!(first.api_key, "sk-12345678");
        assert_eq!(first.model, "qwen3:8b");

        // 第二次：空值 = 清除该字段；只保留 model 新值
        set_user_config("", "", "gpt-4o-mini").unwrap();
        let second: LlmConfig =
            serde_json::from_str(&fs::read_to_string(root.join("config.json")).unwrap()).unwrap();
        assert_eq!(second.base_url, "", "空值应清除字段");
        assert!(second.api_key.is_empty());
        assert_eq!(second.model, "gpt-4o-mini");

        std::env::remove_var("ELWRIGHT_USER_ROOT");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn set_user_config_preserves_unknown_keys() {
        let _guard = USER_ROOT_LOCK.lock().unwrap();
        let root = temp_user_root("preserve");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);
        fs::write(
            root.join("config.json"),
            r#"{"base_url":"http://old:1/v1","custom_theme":"dark"}"#,
        )
        .unwrap();
        set_user_config("http://new:2/v1", "", "").unwrap();
        let text = fs::read_to_string(root.join("config.json")).unwrap();
        assert!(text.contains("custom_theme"));
        assert!(text.contains("http://new:2/v1"));
        assert!(!text.contains("http://old:1/v1"));
        std::env::remove_var("ELWRIGHT_USER_ROOT");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn view_masks_api_key() {
        assert_eq!(mask_key(""), "");
        assert_eq!(mask_key("short"), "****");
        assert_eq!(mask_key("sk-1234567890"), "sk-1****");
    }

    #[test]
    fn test_connection_reports_unreachable_in_chinese() {
        // 端口几乎必然无人监听；只验证错误是中文可读的传输错误
        let err = super::test_connection("http://127.0.0.1:1/v1", "", "m").unwrap_err();
        assert!(err.contains("无法连接"), "实际: {}", err);
    }

    /// 单连接 mock 端点：读完请求后回一个合法 chat completion。
    /// 返回收到的原始请求文本供断言（body / 鉴权 / 多轮 role 序列）。
    fn spawn_mock_llm(reply_content: &str) -> (String, std::thread::JoinHandle<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let reply = reply_content.to_string();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 8192];
            let mut raw = Vec::new();
            // 读到 header 结束且 body 长度收满为止
            loop {
                let n = stream.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break;
                }
                raw.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&raw);
                if let Some(pos) = text.find("\r\n\r\n") {
                    let len: usize = text[..pos]
                        .lines()
                        .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
                        .and_then(|l| l.split(':').nth(1))
                        .and_then(|v| v.trim().parse().ok())
                        .unwrap_or(0);
                    if raw.len() >= pos + 4 + len {
                        break;
                    }
                }
            }
            let body = format!(
                r#"{{"choices":[{{"message":{{"role":"assistant","content":{}}}}}]}}"#,
                serde_json::to_string(&reply).unwrap()
            );
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).unwrap();
            String::from_utf8_lossy(&raw).into_owned()
        });
        (format!("http://{}", addr), handle)
    }

    #[test]
    fn chat_messages_round_trips_via_mock_endpoint() {
        let (base_url, server) = spawn_mock_llm("你好，我是助手。");
        let client = super::LlmClient {
            config: super::LlmConfig {
                base_url,
                api_key: "sk-test".into(),
                model: "mock-model".into(),
            },
        };
        let reply = client
            .chat_messages(&[
                super::ChatMessage::new("system", "sys-prompt"),
                super::ChatMessage::new("user", "hi"),
                super::ChatMessage::new("assistant", "hello"),
                super::ChatMessage::new("user", "again"),
            ])
            .unwrap();
        assert_eq!(reply, "你好，我是助手。");

        let request = server.join().unwrap();
        assert!(request.contains("POST /chat/completions"), "实际: {}", request);
        assert!(request.contains("Bearer sk-test"));
        assert!(request.contains("\"model\":\"mock-model\""));
        // 多轮 role 按原顺序进 body
        assert!(request.contains("\"role\":\"system\",\"content\":\"sys-prompt\""));
        assert!(request.contains("\"role\":\"user\",\"content\":\"hi\""));
        assert!(request.contains("\"role\":\"assistant\",\"content\":\"hello\""));
        assert!(request.contains("\"role\":\"user\",\"content\":\"again\""));
    }

    #[test]
    fn chat_is_two_message_wrapper_of_chat_messages() {
        // chat(system,user) = chat_messages([system,user])，invoke 路径行为不变
        let (base_url, server) = spawn_mock_llm("ok");
        let client = super::LlmClient {
            config: super::LlmConfig {
                base_url,
                api_key: String::new(),
                model: "m".into(),
            },
        };
        let reply = client.chat("tpl", "input").unwrap();
        assert_eq!(reply, "ok");
        let request = server.join().unwrap();
        assert!(request.contains("\"role\":\"system\",\"content\":\"tpl\""));
        assert!(request.contains("\"role\":\"user\",\"content\":\"input\""));
        // 无 api_key 时不带 Authorization 头
        assert!(!request.contains("Authorization"));
    }
}
