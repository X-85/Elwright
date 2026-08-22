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
/// 文件解析走双根（overlay 优先），被遮蔽的内置能力导出的是 overlay 版本。
pub fn export_capability(reg: &Registry, id: &str) -> Result<String, String> {
    let cap = reg.get(id).ok_or_else(|| format!("未找到能力: {}", id))?;
    let mut files = Vec::new();
    for rel in referenced_files(cap) {
        let path = reg.resolve_resource(&rel);
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

/// 导入能力到指定目标根（调用方决定语义：CLI/桌面壳均传用户叠加层根，
/// 保证装机场景写用户目录而非只读的 bundle 内部）。
/// 写回目标根的 capabilities.json（保持既有键序）+ 落地 files。
/// id 冲突时默认报错，force=true 覆盖。
pub fn import_capability(target_root: &Path, text: &str, force: bool) -> Result<String, String> {
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

    let reg_path = target_root.join("capabilities.json");
    let reg_text = std::fs::read_to_string(&reg_path)
        .unwrap_or_else(|_| "{\"capabilities\":[]}".to_string());
    let mut reg_value: serde_json::Value = serde_json::from_str(&reg_text)
        .map_err(|e| format!("解析 {} 失败: {}", reg_path.display(), e))?;
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

    // 先建目录再写文件，最后写注册表：中途失败重跑安全（文件写入幂等）
    if let Some(parent) = reg_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录 {} 失败: {}", parent.display(), e))?;
    }
    for f in &bundle.files {
        let dest = target_root.join(&f.path);
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

/// 删除自定义能力：从 overlay 注册表移除条目并清理其独占的引用文件。
/// 仅允许删 overlay 条目（内置不可删）；文件若同时被 overlay 其他条目引用则保留。
/// 返回删除摘要。未引用但已不存在的文件按成功处理（幂等）。
pub fn delete_capability(overlay_root: &Path, reg: &Registry, id: &str) -> Result<String, String> {
    if reg.get(id).is_none() {
        return Err(format!("未找到能力: {}", id));
    }
    if reg.origin_of(id) != crate::core::registry::Origin::Custom {
        return Err(format!("{} 是内置能力，不可删除（可被 overlay 同 id 覆盖）", id));
    }

    // overlay 注册表现状（条目可能被遮蔽语义合并过，读文件为准）
    let reg_path = overlay_root.join("capabilities.json");
    let reg_text = std::fs::read_to_string(&reg_path)
        .map_err(|e| format!("读取 {} 失败: {}", reg_path.display(), e))?;
    let mut reg_value: serde_json::Value = serde_json::from_str(&reg_text)
        .map_err(|e| format!("解析 {} 失败: {}", reg_path.display(), e))?;
    let caps = reg_value
        .get_mut("capabilities")
        .and_then(|v| v.as_array_mut())
        .ok_or("overlay capabilities.json 缺少 capabilities 数组")?;
    let target = caps
        .iter()
        .find(|c| c.get("id").and_then(|i| i.as_str()) == Some(id))
        .ok_or_else(|| format!("叠加层中不存在能力 {}", id))?;
    let cap: crate::core::registry::Capability =
        serde_json::from_value(target.clone()).map_err(|e| e.to_string())?;
    caps.retain(|c| c.get("id").and_then(|i| i.as_str()) != Some(id));

    // 收集 overlay 其余条目仍引用的相对路径，删除目标独占的文件
    let mut still_referenced: Vec<String> = Vec::new();
    for c in caps.iter() {
        for field in ["entry", "doc", "degradeDoc"] {
            if let Some(v) = c.get(field).and_then(|f| f.as_str()) {
                still_referenced.push(v.to_string());
            }
        }
    }

    let mut removed_files = 0;
    for rel in referenced_files(&cap) {
        if still_referenced.contains(&rel) {
            continue;
        }
        let path = overlay_root.join(&rel);
        // 只清理 overlay 目录内的文件（防误删 base 的同名文件）
        if path.starts_with(overlay_root) && path.is_file() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("删除 {} 失败: {}", rel, e))?;
            removed_files += 1;
        }
    }

    let output = serde_json::to_string_pretty(&reg_value).map_err(|e| e.to_string())?;
    std::fs::write(&reg_path, output + "\n")
        .map_err(|e| format!("写回 {} 失败: {}", reg_path.display(), e))?;

    Ok(format!("已删除自定义能力 {}（清理 {} 个文件）", id, removed_files))
}

#[cfg(test)]
mod tests {
    use super::{delete_capability, export_capability, import_capability};
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

    #[test]
    fn import_creates_target_root_when_missing() {
        // 装机首导：overlay 目录还不存在 capabilities.json，应自动创建
        let base = temp_root("imp-base");
        let overlay = std::env::temp_dir().join(format!("elwright-imp-over-{}", std::process::id()));
        let _ = fs::remove_dir_all(&overlay);

        let bundle = r#"{"schema":"elwright-skill/0.1","capability":{"id":"new","name":"新","type":"script","entry":"resources/tools/new/run.py"},"files":[{"path":"resources/tools/new/run.py","content":"print(1)"}]}"#;
        let msg = import_capability(&overlay, bundle, false).unwrap();
        assert!(msg.contains("已导入能力 new"));
        assert!(overlay.join("resources/tools/new/run.py").is_file());
        assert!(overlay.join("capabilities.json").is_file());
        // base 不受影响
        let base_text = fs::read_to_string(base.join("capabilities.json")).unwrap();
        assert!(!base_text.contains("new"));

        fs::remove_dir_all(&base).ok();
        fs::remove_dir_all(&overlay).ok();
    }

    #[test]
    fn delete_only_custom_and_cleans_files() {
        let base = temp_root("del-base");
        let overlay = std::env::temp_dir().join(format!("elwright-del-over-{}", std::process::id()));
        let _ = fs::remove_dir_all(&overlay);
        fs::create_dir_all(overlay.join("resources/docs")).unwrap();

        // overlay 里有 shared（两条目引用同一文件）和 solo（独占文件）
        fs::write(
            overlay.join("capabilities.json"),
            r#"{"capabilities":[
                {"id":"solo","name":"独占","type":"knowledge","doc":"resources/docs/solo.md"},
                {"id":"shared-a","name":"共A","type":"knowledge","doc":"resources/docs/shared.md"},
                {"id":"shared-b","name":"共B","type":"knowledge","doc":"resources/docs/shared.md"}
            ]}"#,
        ).unwrap();
        fs::write(overlay.join("resources/docs/solo.md"), "solo").unwrap();
        fs::write(overlay.join("resources/docs/shared.md"), "shared").unwrap();

        let reg = Registry::load_with_overlay(&base, Some(&overlay)).unwrap();

        // 内置条目拒删
        let err = delete_capability(&overlay, &reg, "demo").unwrap_err();
        assert!(err.contains("内置"));

        // 独占文件被清理
        let msg = delete_capability(&overlay, &reg, "solo").unwrap();
        assert!(msg.contains("已删除自定义能力 solo"));
        assert!(!overlay.join("resources/docs/solo.md").exists());

        // 共享文件保留
        delete_capability(&overlay, &reg, "shared-a").unwrap();
        assert!(overlay.join("resources/docs/shared.md").exists());
        // 最后一个引用者删除后文件才清理
        delete_capability(&overlay, &reg, "shared-b").unwrap();
        assert!(!overlay.join("resources/docs/shared.md").exists());

        // 注册表状态正确
        let reg2 = Registry::load_with_overlay(&base, Some(&overlay)).unwrap();
        assert!(reg2.get("solo").is_none());
        assert!(reg2.get("shared-a").is_none());
        assert!(reg2.get("shared-b").is_none());
        assert!(reg2.get("demo").is_some());

        fs::remove_dir_all(&base).ok();
        fs::remove_dir_all(&overlay).ok();
    }
}
