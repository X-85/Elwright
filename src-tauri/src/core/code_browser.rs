//! 代码浏览器阶段①：本地项目只读查看（feature-2026-08-code-browser-phase1）。
//!
//! 边界（docs/features/code-browser/architecture.md）：
//! - 所有读取限制在用户主动选择的项目根内，拒绝绝对路径与 `..` 穿越。
//! - 大文件、二进制、超深目录、超量条目一律截断或拒绝，不阻塞 UI。
//! - 敏感文件（.env / 密钥 / 证书类）只报元数据，不读内容。
//! - 轻量符号索引为按需行级扫描，不常驻、不做完整语义分析；
//!   无法唯一确定目标时返回候选列表。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

// ---- 限制常量（防阻塞 / 防越界）----

/// 单个文本文件读取上限（512 KB），超出截断。
pub const MAX_FILE_BYTES: u64 = 512 * 1024;
/// 目录树单层条目上限。
pub const MAX_TREE_ENTRIES: usize = 2000;
/// 目录树最大深度。
pub const MAX_TREE_DEPTH: usize = 8;
/// 内容搜索的单文件上限（1 MB）。
pub const MAX_SEARCH_FILE_BYTES: u64 = 1024 * 1024;
/// 搜索结果条数上限。
pub const MAX_SEARCH_RESULTS: usize = 200;
/// 符号扫描的 .java 文件数上限。
pub const MAX_SYMBOL_FILES: usize = 5000;

/// 目录树跳过的目录名（构建产物 / VCS / IDE）。
const SKIP_DIRS: &[&str] = &[
    "target",
    "node_modules",
    ".git",
    "build",
    "dist",
    ".idea",
    "out",
    "bin",
    "obj",
    ".gradle",
];

// ---- 数据模型（camelCase 下发，架构文档 §第一阶段建议数据模型）----

/// 目录树条目。
#[derive(Debug, Clone, Serialize)]
pub struct TreeEntry {
    /// 相对项目根的路径（`/` 分隔）。
    pub path: String,
    pub name: String,
    /// "dir" | "file"
    pub kind: String,
    pub size: u64,
    /// false = 敏感文件或超大文件，前端禁用打开。
    pub readable: bool,
    /// .env / 密钥 / 证书类文件：只报元数据，永不读内容。
    pub sensitive: bool,
}

/// 打开的代码文档。
#[derive(Debug, Clone, Serialize)]
pub struct CodeDocument {
    pub path: String,
    pub name: String,
    /// java / xml / yaml / properties / json / md / sql / js / ts / html / css / sh / py / kotlin / gradle / toml / text
    pub language: String,
    pub size: u64,
    /// 超过 MAX_FILE_BYTES 截断时为 true，content 只含前段。
    pub truncated: bool,
    /// 敏感文件时为空字符串，前端展示风险提示。
    pub content: String,
    pub sensitive: bool,
    /// 敏感文件提示或截断说明（中文，面向用户）。
    pub notice: String,
}

/// 搜索命中。
#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub path: String,
    pub name: String,
    /// 文件名搜索为 0；内容搜索为 1 起的行号。
    pub line: u32,
    /// 内容搜索的命中行截断片段。
    pub snippet: String,
}

/// 轻量符号命中。
#[derive(Debug, Clone, Serialize)]
pub struct SymbolHit {
    pub name: String,
    /// "class" | "interface" | "enum" | "record" | "method"
    pub kind: String,
    pub path: String,
    /// 1 起行号。
    pub line: u32,
    /// 声明行原文（截断），供候选列表消歧。
    pub declaration: String,
}

// ---- 路径边界 ----

/// 规范化相对路径并拼接，拒绝绝对路径、`..` 与空路径；返回 (绝对路径, 相对路径)。
pub fn resolve_in_root(root: &Path, rel: &str) -> Result<(PathBuf, String), String> {
    if rel.is_empty() {
        return Ok((root.to_path_buf(), String::new()));
    }
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        return Err("路径必须是项目内相对路径".into());
    }
    for c in rel_path.components() {
        match c {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => return Err(format!("路径包含不允许的组件: {rel}")),
        }
    }
    let joined = root.join(rel_path);
    // root 自身规范化一次；已存在部分规范化后必须仍在 root 内。
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("项目根不可访问: {e}"))?;
    if joined.exists() {
        let canonical = joined
            .canonicalize()
            .map_err(|e| format!("路径不可访问: {e}"))?;
        if !canonical.starts_with(&canonical_root) {
            return Err("路径越出项目根".into());
        }
        if canonical == canonical_root {
            return Ok((root.to_path_buf(), String::new()));
        }
    }
    let normalized = rel_path
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    Ok((joined, normalized))
}

// ---- 敏感文件识别 ----

/// 敏感文件判定：.env、密钥、证书、凭据类。只报元数据，永不读内容。
pub fn is_sensitive(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with(".env")
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower.ends_with(".p12")
        || lower.ends_with(".pfx")
        || lower.ends_with(".jks")
        || lower.ends_with(".keystore")
        || lower.starts_with("id_rsa")
        || lower.starts_with("id_ed25519")
        || lower.contains("credential")
        || lower.contains("password")
        || lower == "secrets.properties"
        || lower.ends_with("secrets.yaml")
        || lower.ends_with("secrets.yml")
}

// ---- 语言识别 ----

/// 按扩展名识别语言（第一阶段够用的白名单）。
pub fn detect_language(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "java" => "java",
        "xml" => "xml",
        "yaml" | "yml" => "yaml",
        "properties" => "properties",
        "json" => "json",
        "md" => "markdown",
        "sql" => "sql",
        "js" | "cjs" | "mjs" => "javascript",
        "ts" => "typescript",
        "html" | "htm" => "html",
        "css" | "scss" => "css",
        "sh" | "bash" => "shell",
        "py" => "python",
        "kt" | "kts" => "kotlin",
        "gradle" => "gradle",
        "toml" => "toml",
        _ => "text",
    }
    .into()
}

// ---- 目录树 ----

/// 读取目录树一层；`rel` 为空表示项目根。
pub fn tree(root: &Path, rel: &str) -> Result<Vec<TreeEntry>, String> {
    let (abs, normalized) = resolve_in_root(root, rel)?;
    let meta = fs::metadata(&abs).map_err(|e| format!("路径不可访问: {e}"))?;
    if !meta.is_dir() {
        return Err("不是目录".into());
    }
    let mut entries: Vec<TreeEntry> = Vec::new();
    let read = fs::read_dir(&abs).map_err(|e| format!("读取目录失败: {e}"))?;
    for entry in read.take(MAX_TREE_ENTRIES) {
        let entry = entry.map_err(|e| format!("读取目录失败: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let child_rel = if normalized.is_empty() {
            name.clone()
        } else {
            format!("{normalized}/{name}")
        };
        if ft.is_dir() {
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            entries.push(TreeEntry {
                path: child_rel,
                name,
                kind: "dir".into(),
                size: 0,
                readable: true,
                sensitive: false,
            });
        } else {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let sensitive = is_sensitive(&name);
            let readable = !sensitive && size <= MAX_FILE_BYTES;
            entries.push(TreeEntry {
                path: child_rel,
                name,
                kind: "file".into(),
                size,
                readable,
                sensitive,
            });
        }
        if entries.len() >= MAX_TREE_ENTRIES {
            break;
        }
    }
    // 目录在前，各自按名称排序。
    entries.sort_by(|a, b| match (a.kind.as_str(), b.kind.as_str()) {
        ("dir", "file") => std::cmp::Ordering::Less,
        ("file", "dir") => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}

// ---- 文件读取 ----

/// 读取文本文件（边界、敏感、大小、二进制嗅探都在这里收口）。
pub fn read_file(root: &Path, rel: &str) -> Result<CodeDocument, String> {
    let (abs, normalized) = resolve_in_root(root, rel)?;
    let meta = fs::metadata(&abs).map_err(|e| format!("路径不可访问: {e}"))?;
    if meta.is_dir() {
        return Err("是目录，不是文件".into());
    }
    let name = abs
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let size = meta.len();
    if is_sensitive(&name) {
        return Ok(CodeDocument {
            path: normalized,
            name: name.clone(),
            language: detect_language(&name),
            size,
            truncated: false,
            content: String::new(),
            sensitive: true,
            notice: "敏感文件（密钥/证书/凭据类）只显示元数据，不读取内容。".into(),
        });
    }
    if size > MAX_FILE_BYTES {
        return Ok(CodeDocument {
            path: normalized,
            name: name.clone(),
            language: detect_language(&name),
            size,
            truncated: true,
            content: String::new(),
            sensitive: false,
            notice: format!(
                "文件超过 {} KB 上限，第一阶段不读取；请在 IDE 中查看。",
                MAX_FILE_BYTES / 1024
            ),
        });
    }
    let bytes = fs::read(&abs).map_err(|e| format!("读取文件失败: {e}"))?;
    if bytes.contains(&0) {
        return Ok(CodeDocument {
            path: normalized,
            name,
            language: "binary".into(),
            size,
            truncated: false,
            content: String::new(),
            sensitive: false,
            notice: "二进制文件不支持预览。".into(),
        });
    }
    let (content, truncated) = if bytes.len() as u64 > MAX_FILE_BYTES {
        (String::from_utf8_lossy(&bytes).to_string(), true)
    } else {
        (String::from_utf8_lossy(&bytes).to_string(), false)
    };
    Ok(CodeDocument {
        path: normalized,
        name: name.clone(),
        language: detect_language(&name),
        size,
        truncated,
        content,
        sensitive: false,
        notice: String::new(),
    })
}

// ---- 搜索 ----

/// 项目内搜索：`mode` = "filename"（文件名包含）或 "content"（行包含）。
/// 内容搜索跳过跳过目录、二进制与大文件；命中上限 MAX_SEARCH_RESULTS。
pub fn search(root: &Path, query: &str, mode: &str) -> Result<Vec<SearchHit>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Err("搜索词为空".into());
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("项目根不可访问: {e}"))?;
    let mut hits = Vec::new();
    let lower_query = query.to_lowercase();
    walk_files(&canonical_root, "", 0, &mut |abs, rel| {
        if hits.len() >= MAX_SEARCH_RESULTS {
            return;
        }
        let name = abs
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let name_clone = name.clone();
        if mode == "filename" {
            if name.to_lowercase().contains(&lower_query) {
                hits.push(SearchHit {
                    path: rel.to_string(),
                    name: name_clone,
                    line: 0,
                    snippet: String::new(),
                });
            }
            return;
        }
        // content 模式
        let Ok(meta) = fs::metadata(abs) else { return };
        if meta.len() > MAX_SEARCH_FILE_BYTES {
            return;
        }
        let Ok(bytes) = fs::read(abs) else { return };
        if bytes.contains(&0) {
            return;
        }
        let text = String::from_utf8_lossy(&bytes);
        for (idx, line) in text.lines().enumerate() {
            if line.to_lowercase().contains(&lower_query) {
                let trimmed = line.trim();
                let snippet = if trimmed.len() > 200 {
                    format!("{}…", &trimmed[..200])
                } else {
                    trimmed.to_string()
                };
                hits.push(SearchHit {
                    path: rel.to_string(),
                    name: name.clone(),
                    line: (idx + 1) as u32,
                    snippet,
                });
                if hits.len() >= MAX_SEARCH_RESULTS {
                    return;
                }
            }
        }
    });
    Ok(hits)
}

// ---- 轻量符号索引 ----

/// 按需扫描项目内 Java 符号（类型声明 / implements / extends / 方法声明）。
/// 行级启发式，不构建完整语法树；命中上限同搜索。
pub fn scan_symbols(root: &Path) -> Result<Vec<SymbolHit>, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("项目根不可访问: {e}"))?;
    let mut hits = Vec::new();
    let mut scanned = 0usize;
    walk_files(&canonical_root, "", 0, &mut |abs, rel| {
        if scanned >= MAX_SYMBOL_FILES || hits.len() >= MAX_SEARCH_RESULTS {
            return;
        }
        if !rel.ends_with(".java") {
            return;
        }
        scanned += 1;
        let Ok(bytes) = fs::read(abs) else { return };
        if bytes.contains(&0) || bytes.len() as u64 > MAX_FILE_BYTES {
            return;
        }
        let text = String::from_utf8_lossy(&bytes);
        for (idx, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if let Some(hit) = parse_type_declaration(line, rel, (idx + 1) as u32) {
                hits.push(hit);
            } else if let Some(hit) = parse_method_declaration(line, rel, (idx + 1) as u32) {
                hits.push(hit);
            }
            if hits.len() >= MAX_SEARCH_RESULTS {
                return;
            }
        }
    });
    Ok(hits)
}

/// 解析类型声明行：class/interface/enum/record + implements/extends 关系。
fn parse_type_declaration(line: &str, rel: &str, line_no: u32) -> Option<SymbolHit> {
    for keyword in ["interface", "class", "enum", "record"] {
        // 粗略词边界：关键字前不是字母数字下划线
        if let Some(pos) = find_word(line, keyword) {
            let rest = &line[pos + keyword.len()..];
            let name: String = rest
                .chars()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                .collect();
            if name.is_empty() {
                return None;
            }
            // 注释行跳过
            if line.starts_with('*') || line.starts_with("//") {
                return None;
            }
            return Some(SymbolHit {
                name,
                kind: keyword.into(),
                path: rel.into(),
                line: line_no,
                declaration: truncate_decl(line),
            });
        }
    }
    None
}

/// 解析方法声明行：修饰符 + 返回类型 + 名称( —— 跳过控制流关键字。
fn parse_method_declaration(line: &str, rel: &str, line_no: u32) -> Option<SymbolHit> {
    const SKIP: &[&str] = &[
        "if",
        "for",
        "while",
        "switch",
        "catch",
        "return",
        "new",
        "do",
        "else",
        "try",
        "synchronized",
    ];
    if !(line.contains('(') && !line.starts_with("//") && !line.starts_with('*')) {
        return None;
    }
    let open = line.find('(')?;
    let head = &line[..open];
    let method: String = head
        .chars()
        .rev()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if method.is_empty() || SKIP.contains(&method.as_str()) {
        return None;
    }
    // 方法名前必须有返回类型（避免 `foo(` 这种裸调用）
    let before_method = head[..head.len() - method.len()].trim();
    if before_method.is_empty() {
        return None;
    }
    Some(SymbolHit {
        name: method,
        kind: "method".into(),
        path: rel.into(),
        line: line_no,
        declaration: truncate_decl(line),
    })
}

fn find_word(haystack: &str, word: &str) -> Option<usize> {
    let bytes = haystack.as_bytes();
    let wb = word.as_bytes();
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(word) {
        let abs = start + pos;
        let before_ok =
            abs == 0 || !bytes[abs - 1].is_ascii_alphanumeric() && bytes[abs - 1] != b'_';
        let after = abs + wb.len();
        let after_ok =
            after >= bytes.len() || !bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_';
        if before_ok && after_ok {
            return Some(abs);
        }
        start = abs + 1;
    }
    None
}

fn truncate_decl(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.len() > 160 {
        format!("{}…", &trimmed[..160])
    } else {
        trimmed.to_string()
    }
}

// ---- 目录遍历（带深度 / 跳过目录 / 上限）----

fn walk_files(dir: &Path, prefix: &str, depth: usize, f: &mut dyn FnMut(&Path, &str)) {
    if depth > MAX_TREE_DEPTH {
        return;
    }
    let Ok(read) = fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        let name = entry.file_name().to_string_lossy().to_string();
        let rel = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if ft.is_dir() {
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            walk_files(&entry.path(), &rel, depth + 1, f);
        } else {
            f(&entry.path(), &rel);
        }
    }
}

// ---- 最近项目 / 最近文件（用户配置层）----

/// 最近项目 / 最近文件持久化结构（~/.elwright/code-browser.json）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecentStore {
    #[serde(default, rename = "projects")]
    pub projects: Vec<RecentProject>,
    #[serde(default, rename = "files")]
    pub files: Vec<RecentFile>,
    /// 收藏文件（阶段③）：按项目根记录。
    #[serde(default, rename = "favorites")]
    pub favorites: Vec<Favorite>,
    /// 代码书签（阶段③）：项目根 + 相对路径 + 行号。
    #[serde(default, rename = "bookmarks")]
    pub bookmarks: Vec<Bookmark>,
}

pub const MAX_FAVORITES: usize = 100;
pub const MAX_BOOKMARKS: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    #[serde(rename = "projectRoot")]
    pub project_root: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    #[serde(rename = "projectRoot")]
    pub project_root: String,
    pub path: String,
    /// 1 起行号。
    pub line: u32,
    /// 可选备注（如书签名）。
    #[serde(default)]
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    #[serde(rename = "rootPath")]
    pub root_path: String,
    #[serde(rename = "lastOpenedAt")]
    pub last_opened_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    #[serde(rename = "projectRoot")]
    pub project_root: String,
    pub path: String,
    #[serde(rename = "lastOpenedAt")]
    pub last_opened_at: u64,
}

pub const MAX_RECENT_PROJECTS: usize = 10;
pub const MAX_RECENT_FILES: usize = 30;

/// 追加/提升一条最近项目记录（同名 rootPath 提升到最前）。
pub fn push_recent_project(store: &mut RecentStore, project: RecentProject) {
    store.projects.retain(|p| p.root_path != project.root_path);
    store.projects.insert(0, project);
    store.projects.truncate(MAX_RECENT_PROJECTS);
}

/// 追加/提升一条最近文件记录（同 rootPath+path 提升到最前）。
pub fn push_recent_file(store: &mut RecentStore, file: RecentFile) {
    store
        .files
        .retain(|f| !(f.project_root == file.project_root && f.path == file.path));
    store.files.insert(0, file);
    store.files.truncate(MAX_RECENT_FILES);
}

/// 从用户配置层读取；不存在 / 损坏时返回默认。
pub fn load_recent(root: &Path) -> RecentStore {
    fs::read_to_string(root.join("code-browser.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 写用户配置层；目录不存在时自建（新机器首次使用也直接可用）。
pub fn save_recent(root: &Path, store: &RecentStore) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| format!("创建用户配置目录失败: {e}"))?;
    let json = serde_json::to_string_pretty(store).map_err(|e| format!("序列化失败: {e}"))?;
    fs::write(root.join("code-browser.json"), json).map_err(|e| format!("写入失败: {e}"))
}

/// 切换收藏：存在则移除（返回 false），不存在则追加（返回 true）。
/// 同一文件全项目共享一条收藏记录；上限 MAX_FAVORITES。
pub fn toggle_favorite(
    store: &mut RecentStore,
    project_root: &str,
    path: &str,
) -> Result<bool, String> {
    if let Some(pos) = store
        .favorites
        .iter()
        .position(|f| f.project_root == project_root && f.path == path)
    {
        store.favorites.remove(pos);
        return Ok(false);
    }
    if store.favorites.len() >= MAX_FAVORITES {
        return Err(format!("收藏数已达上限 {}", MAX_FAVORITES));
    }
    store.favorites.push(Favorite {
        project_root: project_root.into(),
        path: path.into(),
    });
    Ok(true)
}

/// 切换书签：同项目同路径同行存在则移除（返回 false），否则追加（返回 true）。
pub fn toggle_bookmark(
    store: &mut RecentStore,
    project_root: &str,
    path: &str,
    line: u32,
    label: &str,
) -> Result<bool, String> {
    if let Some(pos) = store
        .bookmarks
        .iter()
        .position(|b| b.project_root == project_root && b.path == path && b.line == line)
    {
        store.bookmarks.remove(pos);
        return Ok(false);
    }
    if store.bookmarks.len() >= MAX_BOOKMARKS {
        return Err(format!("书签数已达上限 {}", MAX_BOOKMARKS));
    }
    store.bookmarks.push(Bookmark {
        project_root: project_root.into(),
        path: path.into(),
        line,
        label: label.into(),
    });
    Ok(true)
}

// ---- 单元测试 ----

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "elwright-cb-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_rejects_traversal_and_absolute() {
        let root = temp_project();
        assert!(resolve_in_root(&root, "../outside").is_err());
        assert!(resolve_in_root(&root, "/etc/passwd").is_err());
        assert!(resolve_in_root(&root, "a/../../b").is_err());
        let (abs, norm) = resolve_in_root(&root, "src/main.rs").unwrap();
        assert!(abs.starts_with(&root));
        assert_eq!(norm, "src/main.rs");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn tree_lists_sorted_and_marks_sensitive() {
        let root = temp_project();
        fs::create_dir_all(root.join("src/main")).unwrap();
        fs::write(root.join("src/main/App.java"), "class App {}").unwrap();
        fs::write(root.join(".env"), "SECRET=1").unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        let entries = tree(&root, "").unwrap();
        assert_eq!(entries[0].kind, "dir");
        assert_eq!(entries[0].name, "src");
        assert!(!entries.iter().any(|e| e.name == "target"), "跳过 target");
        let env = entries.iter().find(|e| e.name == ".env").unwrap();
        assert!(env.sensitive);
        assert!(!env.readable);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn read_file_detects_language_and_protects_sensitive() {
        let root = temp_project();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("App.java"), "public class App {}\n").unwrap();
        fs::write(root.join("id_rsa"), "PRIVATE").unwrap();
        let doc = read_file(&root, "App.java").unwrap();
        assert_eq!(doc.language, "java");
        assert!(doc.content.contains("class App"));
        let sensitive = read_file(&root, "id_rsa").unwrap();
        assert!(sensitive.sensitive);
        assert!(sensitive.content.is_empty());
        assert!(!sensitive.notice.is_empty());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn read_file_rejects_outside_root() {
        let root = temp_project();
        fs::create_dir_all(&root).unwrap();
        assert!(read_file(&root, "../escape.txt").is_err());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn search_by_filename_and_content() {
        let root = temp_project();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/UserService.java"),
            "class UserService {}\nfind me here\n",
        )
        .unwrap();
        fs::write(root.join("src/Binary.class"), vec![0u8, 1, 2]).unwrap();
        let by_name = search(&root, "service", "filename").unwrap();
        assert_eq!(by_name.len(), 1);
        assert_eq!(by_name[0].path, "src/UserService.java");
        let by_content = search(&root, "FIND ME", "content").unwrap();
        assert_eq!(by_content.len(), 1);
        assert_eq!(by_content[0].line, 2);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn scan_symbols_finds_types_implements_and_methods() {
        let root = temp_project();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/UserService.java"),
            "public interface UserService {\n    User getById(Long id);\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/UserServiceImpl.java"),
            "public class UserServiceImpl implements UserService {\n    @Override\n    public User getById(Long id) {\n        return null;\n    }\n}\n",
        )
        .unwrap();
        let hits = scan_symbols(&root).unwrap();
        let names: Vec<(&str, &str)> = hits
            .iter()
            .map(|h| (h.kind.as_str(), h.name.as_str()))
            .collect();
        assert!(names.contains(&("interface", "UserService")), "{names:?}");
        assert!(names.contains(&("class", "UserServiceImpl")), "{names:?}");
        let get_by_id = hits
            .iter()
            .find(|h| h.name == "getById" && h.declaration.contains("public User getById"))
            .unwrap();
        assert_eq!(get_by_id.kind, "method");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn scan_symbols_skips_control_flow_and_calls() {
        let root = temp_project();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/A.java"),
            "class A {\n    void run() {\n        if (x) {\n            doSomething(arg);\n        }\n        while (true) {}\n    }\n}\n",
        )
        .unwrap();
        let hits = scan_symbols(&root).unwrap();
        let methods: Vec<&str> = hits
            .iter()
            .filter(|h| h.kind == "method")
            .map(|h| h.name.as_str())
            .collect();
        assert!(methods.contains(&"run"), "{methods:?}");
        assert!(!methods.contains(&"if"), "{methods:?}");
        assert!(
            !methods.contains(&"doSomething"),
            "方法调用不应算声明: {methods:?}"
        );
        assert!(!methods.contains(&"while"), "{methods:?}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn save_recent_creates_missing_dir() {
        let root = temp_project().join("not-exist/nested");
        assert!(!root.exists());
        save_recent(&root, &RecentStore::default()).unwrap();
        assert!(root.join("code-browser.json").is_file());
        fs::remove_dir_all(root.ancestors().nth(2).unwrap()).ok();
    }

    #[test]
    fn favorite_and_bookmark_toggle_roundtrip() {
        let root = temp_project();
        let mut store = RecentStore::default();
        assert!(toggle_favorite(&mut store, "/proj", "src/A.java").unwrap());
        assert!(
            !toggle_favorite(&mut store, "/proj", "src/A.java").unwrap(),
            "二次切换应移除"
        );
        assert!(toggle_favorite(&mut store, "/proj", "src/A.java").unwrap());

        assert!(toggle_bookmark(&mut store, "/proj", "src/A.java", 10, "核心逻辑").unwrap());
        assert!(toggle_bookmark(&mut store, "/proj", "src/A.java", 20, "").unwrap());
        assert!(
            !toggle_bookmark(&mut store, "/proj", "src/A.java", 10, "备注不同也按行去重").unwrap()
        );
        assert_eq!(store.bookmarks.len(), 1);
        assert_eq!(store.bookmarks[0].line, 20);

        save_recent(&root, &store).unwrap();
        let loaded = load_recent(&root);
        assert_eq!(loaded.favorites.len(), 1);
        assert_eq!(loaded.bookmarks[0].label, "");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn recent_store_push_and_persist() {
        let root = temp_project();
        let mut store = RecentStore::default();
        for i in 0..(MAX_RECENT_PROJECTS + 3) {
            push_recent_project(
                &mut store,
                RecentProject {
                    name: format!("p{i}"),
                    root_path: format!("/p{i}"),
                    last_opened_at: i as u64,
                },
            );
        }
        assert_eq!(store.projects.len(), MAX_RECENT_PROJECTS);
        assert_eq!(
            store.projects[0].name,
            format!("p{}", MAX_RECENT_PROJECTS + 2)
        );
        push_recent_project(
            &mut store,
            RecentProject {
                name: "p0".into(),
                root_path: "/p0".into(),
                last_opened_at: 99,
            },
        );
        assert_eq!(
            store.projects[0].root_path, "/p0",
            "同名 rootPath 提升到最前"
        );
        save_recent(&root, &store).unwrap();
        let loaded = load_recent(&root);
        assert_eq!(loaded.projects.len(), MAX_RECENT_PROJECTS);
        fs::remove_dir_all(&root).ok();
    }
}
