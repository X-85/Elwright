use serde::Serialize;
use std::path::Path;

use crate::core::degrade;
use crate::core::llm;
use crate::core::registry::{Capability, Registry};

#[derive(Debug, Clone, Serialize)]
pub struct InvokeOutcome {
    pub source: String,
    pub content: String,
    pub note: Option<String>,
}

/// Invoke a skill through the configured LLM, falling back to its local SOP
/// whenever configuration or the request is unavailable.
///
/// LLM 配置链（架构方案 §5）：环境变量 `ELWRIGHT_LLM_*` > 项目
/// `config.local.json` > 用户 `~/.elwright/config.json` > 注册表
/// `$meta.llmDefault`（默认本地模型，如 Ollama localhost:11434）。
pub fn invoke_skill(reg: &Registry, cap: &Capability, prompt: &str) -> InvokeOutcome {
    let layers = llm::ConfigLayers::collect(&reg.root, reg.llm_default.clone());
    let (config, _) = layers.merged();
    let client = if config.base_url.is_empty() {
        None
    } else {
        Some(llm::LlmClient { config })
    };
    if let Some(client) = client {
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
                    &reg.root,
                    cap,
                    format!("【LLM 调用失败】{}\n已自动降级为离线 SOP。", error),
                )
            }
        }
    }

    degraded(
        &reg.root,
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
