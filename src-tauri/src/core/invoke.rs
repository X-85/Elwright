use serde::Serialize;
use std::path::Path;

use crate::core::{degrade, llm, registry::Capability};

#[derive(Debug, Clone, Serialize)]
pub struct InvokeOutcome {
    pub source: String,
    pub content: String,
    pub note: Option<String>,
}

/// Invoke a skill through the configured LLM, falling back to its local SOP
/// whenever configuration or the request is unavailable.
pub fn invoke_skill(root: &Path, cap: &Capability, prompt: &str) -> InvokeOutcome {
    if let Some(client) = llm::LlmClient::from_env() {
        let system = cap.prompt.as_deref().unwrap_or("");
        let user = if prompt.is_empty() {
            "（无附加输入，请按模板直接执行）"
        } else {
            prompt
        };

        match client.chat(system, user) {
            Ok(content) => {
                return InvokeOutcome {
                    source: "llm".to_string(),
                    content,
                    note: None,
                }
            }
            Err(error) => {
                return degraded(
                    root,
                    cap,
                    format!("【LLM 调用失败】{}\n已自动降级为离线 SOP。", error),
                )
            }
        }
    }

    degraded(
        root,
        cap,
        "【离线降级】未配置 LLM（设置 ELWRIGHT_LLM_BASE_URL 等环境变量可解锁）".to_string(),
    )
}

fn degraded(root: &Path, cap: &Capability, note: String) -> InvokeOutcome {
    InvokeOutcome {
        source: "degraded".to_string(),
        content: degrade::show_sop(root, cap),
        note: Some(note),
    }
}
