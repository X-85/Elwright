use crate::core::llm;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A single capability entry loaded from `capabilities.json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub category: Option<String>,
    pub entry: Option<String>,
    pub doc: Option<String>,
    pub offline: Option<bool>,
    pub prompt: Option<String>,
    #[serde(rename = "degradeDoc")]
    pub degrade_doc: Option<String>,
}

/// Loaded capability registry, bound to an Elwright project root.
#[derive(Debug)]
pub struct Registry {
    pub root: PathBuf,
    pub items: Vec<Capability>,
    /// `$meta.llmDefault`：未配置环境变量时的 LLM 默认端点（架构方案 §5
    /// 「默认指向本地模型」）。
    pub llm_default: Option<llm::LlmConfig>,
}

impl Registry {
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = root.join("capabilities.json");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取 {} 失败: {}", path.display(), e))?;
        #[derive(Deserialize)]
        struct RegistryFile {
            #[serde(default)]
            capabilities: Vec<Capability>,
            #[serde(rename = "$meta", default)]
            meta: Option<RegistryMeta>,
        }
        #[derive(Deserialize)]
        struct RegistryMeta {
            #[serde(rename = "llmDefault", default)]
            llm_default: Option<llm::LlmConfig>,
        }
        let file: RegistryFile = serde_json::from_str(&text)
            .map_err(|e| format!("解析 capabilities.json 失败: {}", e))?;
        Ok(Self {
            root: root.to_path_buf(),
            items: file.capabilities,
            llm_default: file.meta.and_then(|m| m.llm_default),
        })
    }

    pub fn list(&self) -> &[Capability] {
        &self.items
    }

    pub fn get(&self, id: &str) -> Option<&Capability> {
        self.items.iter().find(|c| c.id == id)
    }

    /// Resolve a capability's `entry` to an absolute path under the project root.
    pub fn resolve_entry(&self, cap: &Capability) -> Option<PathBuf> {
        cap.entry.as_ref().map(|e| self.root.join(e))
    }
}

/// Locate the project root from the current directory, then from the
/// executable location. The latter keeps the CLI and desktop shell usable
/// when launched outside the repository root during development.
/// Locate the Elwright resource root, three tiers (design doc
/// docs/work/active/feature-2026-08-stage4-release/design.md §2):
///
/// 1. `ELWRIGHT_ROOT` env override (troubleshooting / advanced users);
/// 2. walk up from the current directory (development mode, unchanged);
/// 3. caller-supplied probe dirs — the Tauri shell passes the bundle
///    resource dir (`Contents/Resources` on macOS), the CLI passes the
///    executable location so installed binaries find exe-adjacent resources.
pub fn resolve_root(extra_probe_dirs: &[PathBuf]) -> PathBuf {
    if let Ok(root) = std::env::var("ELWRIGHT_ROOT") {
        let path = PathBuf::from(&root);
        if path.join("capabilities.json").is_file() {
            return path;
        }
    }

    if let Ok(current) = std::env::current_dir() {
        if let Some(root) = find_root_from(&current) {
            return root;
        }
    }

    for dir in extra_probe_dirs {
        if let Some(root) = find_root_from(dir) {
            return root;
        }
    }

    // Installed CLI fallback: resources placed next to the executable
    // (Windows bundle layout; harmless elsewhere).
    if let Ok(executable) = std::env::current_exe() {
        if let Some(parent) = executable.parent() {
            if let Some(root) = find_root_from(parent) {
                return root;
            }
        }
    }

    PathBuf::from(".")
}

fn find_root_from(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("capabilities.json").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{find_root_from, resolve_root, Capability};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;

    // resolve_root 读进程级环境变量与 cwd，测试间需串行
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// temp 目录树: root/capabilities.json + root/nested/child/
    fn temp_root(tag: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "elwright-root-{}-{}",
            tag,
            std::process::id()
        ));
        let child = root.join("nested/child");
        fs::create_dir_all(&child).unwrap();
        fs::write(root.join("capabilities.json"), "{\"capabilities\":[]}").unwrap();
        (root, child)
    }

    #[test]
    fn env_override_wins() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (root, _) = temp_root("env");
        std::env::set_var("ELWRIGHT_ROOT", &root);
        let resolved = resolve_root(&[]);
        std::env::remove_var("ELWRIGHT_ROOT");
        assert_eq!(resolved, root);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cwd_walkup_finds_ancestor() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (root, child) = temp_root("cwd");
        // 本仓库内运行时 cwd 上溯会命中仓库根；这里仅验证上溯语义：
        // 从 child 起找（借道 find_root_from 的纯函数）
        assert_eq!(find_root_from(&child).unwrap(), root);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn probe_dir_matches_bundle_layout() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (root, _) = temp_root("probe");
        // bundle 布局：资源目录下直接是 capabilities.json（如 Contents/Resources）
        assert_eq!(find_root_from(&root).unwrap(), root);
        // 嵌套子目录也能上溯命中
        assert_eq!(find_root_from(&root.join("nested/child")).unwrap(), root);
        // 空目录树上溯到根也不会误报（/ 与 C:\ 等无注册表）
        let empty_tree = std::env::temp_dir().join(format!(
            "elwright-empty-{}",
            std::process::id()
        ));
        fs::create_dir_all(&empty_tree).unwrap();
        assert_eq!(find_root_from(&empty_tree), None);
        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&empty_tree).ok();
    }

    #[test]
    fn parses_meta_llm_default() {
        let dir = std::env::temp_dir().join(format!("elwright-meta-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("capabilities.json"),
            r#"{"$meta":{"llmDefault":{"base_url":"http://localhost:11434/v1","api_key":"","model":"qwen3:8b"}},"capabilities":[]}"#,
        )
        .unwrap();
        let reg = super::Registry::load(&dir).unwrap();
        let default = reg.llm_default.expect("$meta.llmDefault 应被解析");
        assert_eq!(default.base_url, "http://localhost:11434/v1");
        assert_eq!(default.model, "qwen3:8b");
        fs::write(dir.join("capabilities.json"), "{\"capabilities\":[]}").unwrap();
        let reg = super::Registry::load(&dir).unwrap();
        assert!(reg.llm_default.is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn serializes_ipc_fields_with_json_names() {
        let capability = Capability {
            id: "example".to_string(),
            name: "示例".to_string(),
            kind: "skill".to_string(),
            category: None,
            entry: None,
            doc: None,
            offline: None,
            prompt: None,
            degrade_doc: Some("resources/docs/example.md".to_string()),
        };

        let value = serde_json::to_value(capability).unwrap();
        assert_eq!(value["type"], "skill");
        assert_eq!(value["degradeDoc"], "resources/docs/example.md");
        assert!(value.get("kind").is_none());
        assert!(value.get("degrade_doc").is_none());
    }
}
