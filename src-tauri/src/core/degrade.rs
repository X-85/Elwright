use std::path::Path;

use crate::core::registry::Capability;

/// Offline degradation: when an LLM is unreachable, a skill-type capability
/// falls back to showing its SOP document instead of failing.
pub fn show_sop(root: &Path, cap: &Capability) -> String {
    if let Some(rel) = &cap.degrade_doc {
        let p = root.join(rel);
        if p.exists() {
            return std::fs::read_to_string(&p)
                .unwrap_or_else(|_| "SOP 文件读取失败".to_string());
        }
        return format!("SOP 文件不存在: {}", p.display());
    }
    "该技能型暂无离线 SOP，请联网并配置 LLM 后使用。".to_string()
}
