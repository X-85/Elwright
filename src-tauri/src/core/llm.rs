use std::time::Duration;

/// LLM client configuration.
///
/// Thin self-written client against any OpenAI-compatible endpoint
/// (cloud, Ollama, llama.cpp, ...). Config comes from environment variables:
/// `ELWRIGHT_LLM_BASE_URL`, `ELWRIGHT_LLM_API_KEY`, `ELWRIGHT_LLM_MODEL`.
#[derive(Debug, Clone, Default)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
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
    use super::chat_url;

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
}
