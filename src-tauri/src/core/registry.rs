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
        }
        let file: RegistryFile = serde_json::from_str(&text)
            .map_err(|e| format!("解析 capabilities.json 失败: {}", e))?;
        Ok(Self {
            root: root.to_path_buf(),
            items: file.capabilities,
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
pub fn find_project_root() -> PathBuf {
    if let Ok(current) = std::env::current_dir() {
        if let Some(root) = find_root_from(&current) {
            return root;
        }
    }

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
    use super::Capability;

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
