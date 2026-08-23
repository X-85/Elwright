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
///
/// 支持用户叠加层（overlay，`~/.elwright/capabilities.json` + `resources/`）：
/// base（bundle/仓库根）与 overlay 合并加载，同 id 时 overlay 条目遮蔽内置条目；
/// overlay 引用的文件优先从 `~/.elwright/resources/` 解析，不存在再回退 base。
#[derive(Debug)]
pub struct Registry {
    pub root: PathBuf,
    pub items: Vec<Capability>,
    /// 用户叠加层根（`~/.elwright`）。存在时文件解析先查它。
    pub overlay_root: Option<PathBuf>,
    /// 叠加层中的条目 id 集合（遮蔽内置的同 id 条目）。
    overlay_ids: std::collections::HashSet<String>,
    /// `$meta.llmDefault`：未配置环境变量时的 LLM 默认端点（架构方案 §5
    /// 「默认指向本地模型」）。
    pub llm_default: Option<llm::LlmConfig>,
}

/// 用户叠加层根目录（`~/.elwright`）。`ELWRIGHT_USER_ROOT` 可覆盖（测试/排障）。
/// 主目录缺失时返回 None——此时行为退回纯 base（与旧版一致）。
pub fn user_root() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("ELWRIGHT_USER_ROOT") {
        return Some(PathBuf::from(custom));
    }
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|h| PathBuf::from(h).join(".elwright"))
}

impl Registry {
    pub fn load(root: &Path) -> Result<Self, String> {
        let overlay = user_root().filter(|p| p.join("capabilities.json").is_file());
        Self::load_with_overlay(root, overlay.as_deref())
    }

    /// base + overlay 合并加载。overlay 目录缺 `capabilities.json` 时静默退化为纯 base。
    pub fn load_with_overlay(root: &Path, overlay: Option<&Path>) -> Result<Self, String> {
        let (base_items, llm_default) = read_registry_file(root)?;
        let mut items = base_items;
        let mut overlay_ids = std::collections::HashSet::new();
        let overlay_root = overlay.map(|p| p.to_path_buf());
        if let Some(dir) = overlay {
            if dir.join("capabilities.json").is_file() {
                let (overlay_items, _) = read_registry_file(dir)?;
                for cap in overlay_items {
                    overlay_ids.insert(cap.id.clone());
                    // 同 id 遮蔽：移除内置条目，overlay 版本排在原位
                    if let Some(pos) = items.iter().position(|c| c.id == cap.id) {
                        items[pos] = cap;
                    } else {
                        items.push(cap);
                    }
                }
            }
        }
        Ok(Self {
            root: root.to_path_buf(),
            items,
            overlay_root,
            overlay_ids,
            llm_default,
        })
    }

    /// 条目来源：`custom` = 用户叠加层导入，`builtin` = 内置注册表。
    pub fn origin_of(&self, id: &str) -> Origin {
        if self.overlay_ids.contains(id) {
            Origin::Custom
        } else {
            Origin::Builtin
        }
    }

    /// 相对路径双根解析：overlay 优先，回退 base。返回存在的那个。
    pub fn resolve_resource(&self, rel: &str) -> PathBuf {
        if let Some(overlay) = &self.overlay_root {
            let candidate = overlay.join(rel);
            if candidate.exists() {
                return candidate;
            }
        }
        self.root.join(rel)
    }

    /// Resolve a capability's `entry` to an absolute path under the project root.
    pub fn resolve_entry(&self, cap: &Capability) -> Option<PathBuf> {
        cap.entry.as_ref().map(|e| self.resolve_resource(e))
    }

    pub fn list(&self) -> &[Capability] {
        &self.items
    }

    pub fn get(&self, id: &str) -> Option<&Capability> {
        self.items.iter().find(|c| c.id == id)
    }
}

/// 条目来源标记（IPC 传给前端渲染「自定义」徽标）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    Builtin,
    Custom,
}

fn read_registry_file(root: &Path) -> Result<(Vec<Capability>, Option<llm::LlmConfig>), String> {
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
    let file: RegistryFile =
        serde_json::from_str(&text).map_err(|e| format!("解析 {} 失败: {}", path.display(), e))?;
    Ok((file.capabilities, file.meta.and_then(|m| m.llm_default)))
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
    use super::{find_root_from, resolve_root, Registry};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;

    // resolve_root 读进程级环境变量与 cwd，测试间需串行
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// temp 目录树: root/capabilities.json + root/nested/child/
    fn temp_root(tag: &str) -> (PathBuf, PathBuf) {
        let root =
            std::env::temp_dir().join(format!("elwright-root-{}-{}", tag, std::process::id()));
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
        let empty_tree =
            std::env::temp_dir().join(format!("elwright-empty-{}", std::process::id()));
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
        let capability = super::Capability {
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

    // ---- 叠加层（overlay）----

    /// base 根含 demo/builtin-doc 两项，overlay 根含 demo（遮蔽）+ custom 一项。
    fn overlay_roots(tag: &str) -> (PathBuf, PathBuf) {
        let base =
            std::env::temp_dir().join(format!("elwright-ov-base-{}-{}", tag, std::process::id()));
        let over =
            std::env::temp_dir().join(format!("elwright-ov-over-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let _ = fs::remove_dir_all(&over);
        fs::create_dir_all(base.join("resources/docs")).unwrap();
        fs::create_dir_all(over.join("resources/docs")).unwrap();
        fs::write(
            base.join("capabilities.json"),
            r#"{"capabilities":[
                {"id":"demo","name":"内置版","type":"knowledge","doc":"resources/docs/demo.md"},
                {"id":"builtin-doc","name":"仅内置","type":"knowledge","doc":"resources/docs/base-only.md"}
            ]}"#,
        ).unwrap();
        fs::write(base.join("resources/docs/demo.md"), "# 内置 demo").unwrap();
        fs::write(base.join("resources/docs/base-only.md"), "# 仅内置").unwrap();
        fs::write(
            over.join("capabilities.json"),
            r#"{"capabilities":[
                {"id":"demo","name":"用户版","type":"knowledge","doc":"resources/docs/demo.md"},
                {"id":"custom","name":"自定义","type":"knowledge","doc":"resources/docs/custom.md"}
            ]}"#,
        )
        .unwrap();
        fs::write(
            over.join("resources/docs/demo.md"),
            "# 用户 demo（遮蔽内置）",
        )
        .unwrap();
        fs::write(over.join("resources/docs/custom.md"), "# 自定义文档").unwrap();
        (base, over)
    }

    #[test]
    fn overlay_merges_and_shadows() {
        let (base, over) = overlay_roots("merge");
        let reg = Registry::load_with_overlay(&base, Some(&over)).unwrap();

        // 合并视图：demo（遮蔽）+ builtin-doc + custom
        assert_eq!(reg.list().len(), 3);
        assert_eq!(reg.get("demo").unwrap().name, "用户版");
        assert_eq!(reg.get("custom").unwrap().name, "自定义");
        assert_eq!(reg.get("builtin-doc").unwrap().name, "仅内置");

        // origin 标记
        assert_eq!(reg.origin_of("demo"), super::Origin::Custom);
        assert_eq!(reg.origin_of("custom"), super::Origin::Custom);
        assert_eq!(reg.origin_of("builtin-doc"), super::Origin::Builtin);

        // 遮蔽时文件解析走 overlay 根
        let demo = reg.get("demo").unwrap();
        let path = reg.resolve_resource(demo.doc.as_ref().unwrap());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("用户 demo"), "遮蔽条目应读 overlay 文件");

        // 仅内置存在的文件：overlay 查不到，回退 base
        let builtin = reg.get("builtin-doc").unwrap();
        let path = reg.resolve_resource(builtin.doc.as_ref().unwrap());
        assert!(path.starts_with(&base));

        fs::remove_dir_all(&base).ok();
        fs::remove_dir_all(&over).ok();
    }

    #[test]
    fn overlay_missing_registry_falls_back_to_base() {
        let (base, over) = overlay_roots("fallback");
        // overlay 目录没有 capabilities.json → 纯 base 行为
        fs::remove_file(over.join("capabilities.json")).unwrap();
        let reg = Registry::load_with_overlay(&base, Some(&over)).unwrap();
        assert_eq!(reg.list().len(), 2);
        assert_eq!(reg.get("demo").unwrap().name, "内置版");
        assert_eq!(reg.origin_of("demo"), super::Origin::Builtin);

        // None overlay 同样安全
        let reg = Registry::load_with_overlay(&base, None).unwrap();
        assert_eq!(reg.list().len(), 2);

        fs::remove_dir_all(&base).ok();
        fs::remove_dir_all(&over).ok();
    }
}
