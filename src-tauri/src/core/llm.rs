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
pub fn user_config_path() -> Option<PathBuf> {
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
        let url = chat_url(&self.config.base_url);
        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
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
    use super::{chat_url, ConfigLayers, LlmConfig};
    use std::fs;

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
        // 清掉可能存在的环境变量，保证优先级测试不受宿主环境影响
        for k in ["ELWRIGHT_LLM_BASE_URL", "ELWRIGHT_LLM_API_KEY", "ELWRIGHT_LLM_MODEL"] {
            std::env::remove_var(k);
        }
        let dir = std::env::temp_dir().join(format!("elwright-cfg-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("config.local.json"),
            r#"{"base_url":"http://project:1/v1","model":"proj-model"}"#,
        )
        .unwrap();
        let layers = ConfigLayers::collect(
            &dir,
            Some(LlmConfig {
                base_url: "http://default:2/v1".into(),
                api_key: "reg-key".into(),
                model: "reg-model".into(),
            }),
        );
        // 项目文件覆盖 base_url/model，api_key 字段级回退到注册表默认
        let (cfg, source) = layers.merged();
        assert_eq!(cfg.base_url, "http://project:1/v1");
        assert_eq!(cfg.model, "proj-model");
        assert_eq!(cfg.api_key, "reg-key");
        assert_eq!(source[0], "项目 config.local.json");
        assert_eq!(source[1], "注册表默认 $meta.llmDefault");

        // 无任何配置时全空
        let empty_dir = std::env::temp_dir().join(format!("elwright-cfg-empty-{}", std::process::id()));
        fs::create_dir_all(&empty_dir).unwrap();
        let (cfg2, source2) = ConfigLayers::collect(&empty_dir, None).merged();
        assert!(cfg2.base_url.is_empty());
        assert_eq!(source2[0], "未设置");
        fs::remove_dir_all(&dir).ok();
        fs::remove_dir_all(&empty_dir).ok();
    }
}
