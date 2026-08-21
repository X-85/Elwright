/// LLM client configuration.
///
/// Fully implemented in stage 2 (will use a self-written `reqwest` thin client
/// against the OpenAI-compatible `/v1/chat/completions` endpoint). For now it
/// only captures configuration from the environment so the rest of the core can
/// probe whether an LLM is available.
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

impl LlmClient {
    /// Build from environment variables:
    /// `ELWRIGHT_LLM_BASE_URL`, `ELWRIGHT_LLM_API_KEY`, `ELWRIGHT_LLM_MODEL`.
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

    /// Placeholder for stage 2.
    pub fn chat(&self, _prompt: &str) -> Result<String, String> {
        Err("LLM 客户端尚未实现（阶段 2）".to_string())
    }
}
