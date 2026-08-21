//! 能力导入/导出：把一项能力（注册表条目 + 引用文件）打包为单文件分享。
//!
//! 格式 `elwright-skill/0.1`：纯 JSON、文本直存（零新依赖）：
//! `{schema, capability, files: [{path, content}]}`。
//! 安全约束：导入路径必须是 `resources/` 前缀的相对路径，拒绝 `..` 与绝对路径。

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::core::registry::{Capability, Registry};

pub const SCHEMA: &str = "elwright-skill/0.1";

#[derive(Serialize)]
struct ExportBundle {
    schema: &'static str,
    capability: Capability,
    files: Vec<BundleFile>,
}

#[derive(Serialize, Deserialize)]
struct BundleFile {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct ImportBundle {
    schema: String,
    capability: Capability,
    #[serde(default)]
    files: Vec<BundleFile>,
}

/// 收集能力引用的文件相对路径（entry/doc/degradeDoc，去重，仅保留存在的）。
fn referenced_files(cap: &Capability) -> Vec<String> {
    let mut rels: Vec<String> = Vec::new();
    for field in [&cap.entry, &cap.doc, &cap.degrade_doc] {
        if let Some(rel) = field {
            if !rels.contains(rel) {
                rels.push(rel.clone());
            }
        }
    }
    rels
}

/// 导出能力为 JSON 文本。文件内容按 UTF-8 直存，二进制文件报错。
pub fn export_capability(reg: &Registry, id: &str) -> Result<String, String> {
    let cap = reg.get(id).ok_or_else(|| format!("未找到能力: {}", id))?;
    let mut files = Vec::new();
    for rel in referenced_files(cap) {
        let path = reg.root.join(&rel);
        if !path.exists() {
            continue; // 规划中的 entry 尚未导入文件——打包元数据，不阻塞
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取 {} 失败: {}", rel, e))?;
        files.push(BundleFile { path: rel, content });
    }
    let bundle = ExportBundle {
        schema: SCHEMA,
        capability: cap.clone(),
        files,
    };
    serde_json::to_string_pretty(&bundle).map_err(|e| format!("序列化失败: {}", e))
}

/// 校验导入路径：必须为 resources/ 前缀的相对路径（防目录逃逸）。
fn safe_relative(rel: &str) -> Result<(), String> {
    if rel.starts_with('/') || rel.starts_with('\\') || Path::new(rel).is_absolute() {
        return Err(format!("拒绝绝对路径: {}", rel));
    }
    let normalized = rel.replace('\\', "/");
    if normalized.split('/').any(|seg| seg == "..") {
        return Err(format!("拒绝路径穿越: {}", rel));
    }
    if !normalized.starts_with("resources/") {
        return Err(format!("路径必须位于 resources/ 下: {}", rel));
    }
    Ok(())
}

/// 导入能力：写回文件 + 更新 capabilities.json（保持既有键序）。
/// id 冲突时默认报错，force=true 覆盖。
pub fn import_capability(root: &Path, text: &str, force: bool) -> Result<String, String> {
    let bundle: ImportBundle =
        serde_json::from_str(text).map_err(|e| format!("解析导入文件失败: {}", e))?;
    if bundle.schema != SCHEMA {
        return Err(format!(
            "不支持的 schema: {}（当前支持 {}）",
            bundle.schema, SCHEMA
        ));
    }
    for f in &bundle.files {
        safe_relative(&f.path)?;
    }

    let reg_path = root.join("capabilities.json");
    let reg_text = std::fs::read_to_string(&reg_path)
        .map_err(|e| format!("读取 {} 失败: {}", reg_path.display(), e))?;
    let mut reg_value: serde_json::Value = serde_json::from_str(&reg_text)
        .map_err(|e| format!("解析 capabilities.json 失败: {}", e))?;
    let caps = reg_value
        .get_mut("capabilities")
        .and_then(|v| v.as_array_mut())
        .ok_or("capabilities.json 缺少 capabilities 数组")?;

    let id = bundle.capability.id.clone();
    let existed = caps.iter().any(|c| c.get("id").and_then(|i| i.as_str()) == Some(&id));
    if existed && !force {
        return Err(format!("能力 {} 已存在；加 --force 覆盖", id));
    }
    if existed {
        caps.retain(|c| c.get("id").and_then(|i| i.as_str()) != Some(&id));
    }
    let cap_value = serde_json::to_value(&bundle.capability).map_err(|e| e.to_string())?;
    caps.push(cap_value);

    // 先写文件再写注册表：中途失败重跑安全（文件写入幂等）
    for f in &bundle.files {
        let dest = root.join(&f.path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录 {} 失败: {}", parent.display(), e))?;
        }
        std::fs::write(&dest, &f.content)
            .map_err(|e| format!("写入 {} 失败: {}", f.path, e))?;
    }
    let output = serde_json::to_string_pretty(&reg_value).map_err(|e| e.to_string())?;
    std::fs::write(&reg_path, output + "\n")
        .map_err(|e| format!("写回 {} 失败: {}", reg_path.display(), e))?;

    Ok(format!(
        "已导入能力 {}（含 {} 个文件）{}",
        id,
        bundle.files.len(),
        if existed { "，覆盖原有条目" } else { "" }
    ))
}

#[cfg(test)]
mod tests {
    use super::{export_capability, import_capability};
    use crate::core::registry::Registry;
    use std::fs;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("elwright-exp-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("resources/docs")).unwrap();
        fs::write(
            dir.join("capabilities.json"),
            r#"{"$meta":{"llmDefault":{"base_url":"http://localhost:11434/v1"}},"capabilities":[{"id":"demo","name":"演示","type":"skill","prompt":"模板","degradeDoc":"resources/docs/demo-sop.md"}]}"#,
        )
        .unwrap();
        fs::write(dir.join("resources/docs/demo-sop.md"), "# 演示 SOP\n内容").unwrap();
        dir
    }

    #[test]
    fn round_trip_export_import() {
        let dir_a = temp_root("a");
        let dir_b = temp_root("b");
        // dir_b 换成不含 demo 的注册表（模拟一台没有该能力的新机器）
        fs::write(
            dir_b.join("capabilities.json"),
            r#"{"capabilities":[{"id":"other","name":"其他","type":"knowledge"}]}"#,
        )
        .unwrap();

        let reg_a = Registry::load(&dir_a).unwrap();
        let bundle = export_capability(&reg_a, "demo").unwrap();
        assert!(bundle.contains("elwright-skill/0.1"));
        assert!(bundle.contains("演示 SOP"));

        let msg = import_capability(&dir_b, &bundle, false).unwrap();
        assert!(msg.contains("已导入能力 demo"));
        let reg_b = Registry::load(&dir_b).unwrap();
        assert_eq!(reg_b.get("demo").unwrap().name, "演示");
        let sop = fs::read_to_string(dir_b.join("resources/docs/demo-sop.md")).unwrap();
        assert!(sop.contains("演示 SOP"));

        // 冲突检测与 force 覆盖
        assert!(import_capability(&dir_b, &bundle, false).is_err());
        assert!(import_capability(&dir_b, &bundle, true).is_ok());

        fs::remove_dir_all(&dir_a).ok();
        fs::remove_dir_all(&dir_b).ok();
    }

    #[test]
    fn rejects_path_escape() {
        let dir = temp_root("escape");
        let evil = r#"{"schema":"elwright-skill/0.1","capability":{"id":"evil","name":"x","type":"script"},"files":[{"path":"../evil.sh","content":"x"}]}"#;
        let err = import_capability(&dir, evil, true).unwrap_err();
        assert!(err.contains("路径穿越") || err.contains("resources/"));
        let evil2 = r#"{"schema":"elwright-skill/0.1","capability":{"id":"evil","name":"x","type":"script"},"files":[{"path":"/etc/evil","content":"x"}]}"#;
        assert!(import_capability(&dir, evil2, true).is_err());
        fs::remove_dir_all(&dir).ok();
    }
}
