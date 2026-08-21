use serde::Deserialize;
use std::path::{Path, PathBuf};

/// A single capability entry loaded from `capabilities.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub category: Option<String>,
    pub entry: Option<String>,
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
        let items: Vec<Capability> = serde_json::from_str(&text)
            .map_err(|e| format!("解析 capabilities.json 失败: {}", e))?;
        Ok(Self {
            root: root.to_path_buf(),
            items,
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
