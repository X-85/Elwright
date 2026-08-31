//! 受控补丁编辑（Controlled Patch Apply）— 代码浏览器阶段④（ADR-001）。
//!
//! 解析 unified diff、对单文件应用 hunk、把改前内容快照写入用户配置层用于撤销。
//! 不引入语言服务、不内置编辑器；阶段④范围见 ADR-001。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::code_browser::{is_sensitive, resolve_in_root};

/// 单文件统一 diff 解析后的所有 hunk。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedDiff {
    /// 目标文件相对路径（从 `+++ b/path` 取；缺则整块拒绝）。
    pub file: String,
    /// 解析得到的 hunk 列表（按出现顺序）。
    pub hunks: Vec<Hunk>,
}

/// 单个 hunk：定位信息 + old/new 行内容。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Hunk {
    /// diff 文本中的 @@ 行号（仅展示用，应用以 oldStart/oldLines 校验）。
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    /// 旧文件对应行（含 " " 前缀行）。
    pub old_lines_content: Vec<String>,
    /// 新文件对应行（含 " " 前缀行）。
    pub new_lines_content: Vec<String>,
}

/// 解析后的多文件 diff 列表（解析单个 patch 文本的顶层结果）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedPatch {
    pub files: Vec<ParsedDiff>,
}

/// 写入前的预览（前端三栏渲染与逐 hunk 选择接受/拒绝用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchPreview {
    pub files: Vec<PatchFilePreview>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchFilePreview {
    pub file: String,
    pub current_content: String,
    pub new_content: String,
    pub hunks: Vec<Hunk>,
    pub rejected: bool,
}

/// 应用结果：哪些文件写入成功、哪些跳过、对应快照 ID。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub applied: Vec<String>,
    pub skipped: Vec<String>,
    pub snapshot_id: String,
}

/// 撤销结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevertResult {
    pub restored: Vec<String>,
    pub snapshot_id: String,
}

/// 写入快照：原文件 → 改前内容 / sha256 / 应用时间。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchSnapshot {
    pub id: String,
    pub project_id: String,
    pub entries: Vec<SnapshotEntry>,
    pub applied_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub path: String,
    pub original_content: String,
    pub sha256: String,
}

/// 用户层补丁快照文件路径：~/.elwright/code-browser/applied-patches.json
pub fn snapshot_path(user_root: &Path) -> Result<PathBuf, String> {
    let dir = user_root.join("code-browser");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("创建快照目录失败: {e}"))?;
    }
    Ok(dir.join("applied-patches.json"))
}

/// 解析 unified diff 文本，支持多文件（每文件一个 --- /+++ 头）。
///
/// 仅识别 unified diff（`+`/`-`/` ` 行）。空行表示上下文末尾。
/// 缺路径或缺 hunk 返回 Err；不含任何 diff 头的纯文本当作普通文本，不解析。
pub fn parse_unified_diff(text: &str) -> Result<ParsedPatch, String> {
    let mut files: Vec<ParsedDiff> = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_hunks: Vec<Hunk> = Vec::new();

    // 当前 hunk 解析状态（手动管理以避免闭包 borrow 冲突）
    let mut cur_hunk: Option<PendingHunk> = None;

    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.starts_with("--- ") {
            // 新文件头开始前先收尾上一文件
            flush_pending(&mut cur_hunk, &mut current_hunks);
            flush_file(&mut files, &mut current_file, &mut current_hunks);
            continue;
        }
        if let Some(rest) = line.strip_prefix("+++ ") {
            // 接 --- a/path，取新文件路径（剥掉 git 的 b/ 前缀）
            let path = rest.trim();
            if path == "/dev/null" {
                return Err("diff 头格式异常：+++ /dev/null 不支持".into());
            }
            let normalized = path.strip_prefix("b/").unwrap_or(path);
            current_file = Some(normalized.to_string());
            continue;
        }
        if line.starts_with("@@") {
            flush_pending(&mut cur_hunk, &mut current_hunks);
            cur_hunk = Some(parse_hunk_header(line)?);
            continue;
        }
        if cur_hunk.is_none() {
            continue;
        }
        let h = cur_hunk.as_mut().unwrap();
        if let Some(stripped) = line.strip_prefix('+') {
            h.new_lines_content.push(stripped.to_string());
        } else if let Some(stripped) = line.strip_prefix('-') {
            h.old_lines_content.push(stripped.to_string());
        } else if let Some(stripped) = line.strip_prefix(' ') {
            h.old_lines_content.push(stripped.to_string());
            h.new_lines_content.push(stripped.to_string());
        } else if line.is_empty() {
            // 空行视为非 diff 上下文：split('\n') 在文本末尾产生空字符串，不应作为 hunk 行。
        } else {
            // 非 diff 行（如 "\ No newline at end of file"）忽略
        }
    }
    flush_pending(&mut cur_hunk, &mut current_hunks);
    flush_file(&mut files, &mut current_file, &mut current_hunks);

    if files.is_empty() {
        return Err("未识别到任何 diff 块（需要 --- /+++ 头与 @@ hunk）".into());
    }
    for f in &files {
        if f.hunks.is_empty() {
            return Err(format!("文件 {} 未包含可应用的 hunk", f.file));
        }
    }
    Ok(ParsedPatch { files })
}

struct PendingHunk {
    old_start: u32,
    old_lines: u32,
    new_start: u32,
    new_lines: u32,
    old_lines_content: Vec<String>,
    new_lines_content: Vec<String>,
}

impl PendingHunk {
    fn into_hunk(self) -> Hunk {
        Hunk {
            old_start: self.old_start,
            old_lines: self.old_lines,
            new_start: self.new_start,
            new_lines: self.new_lines,
            old_lines_content: self.old_lines_content,
            new_lines_content: self.new_lines_content,
        }
    }
}

fn parse_hunk_header(line: &str) -> Result<PendingHunk, String> {
    let inner = line
        .strip_prefix("@@")
        .ok_or_else(|| "hunk 头必须以 @@ 开头".to_string())?
        .trim();
    let mut parts = inner.split_whitespace();
    let first = parts.next().ok_or_else(|| "hunk 头格式异常".to_string())?;
    let second = parts.next().ok_or_else(|| "hunk 头格式异常".to_string())?;
    let old_part = first.strip_prefix('-').unwrap_or(first);
    let new_part = second.strip_prefix('+').unwrap_or(second);
    let (old_start, old_lines) = parse_range(old_part)?;
    let (new_start, new_lines) = parse_range(new_part)?;
    Ok(PendingHunk {
        old_start,
        old_lines,
        new_start,
        new_lines,
        old_lines_content: Vec::new(),
        new_lines_content: Vec::new(),
    })
}

fn parse_range(s: &str) -> Result<(u32, u32), String> {
    let mut iter = s.split(',');
    let start: u32 = iter
        .next()
        .ok_or_else(|| "range 缺 start".to_string())?
        .parse()
        .map_err(|e| format!("range start 解析失败: {e}"))?;
    let lines: u32 = match iter.next() {
        Some(n) => n
            .parse()
            .map_err(|e| format!("range lines 解析失败: {e}"))?,
        None => 1,
    };
    Ok((start, lines))
}

fn flush_pending(cur: &mut Option<PendingHunk>, hunks: &mut Vec<Hunk>) {
    if let Some(p) = cur.take() {
        hunks.push(p.into_hunk());
    }
}

fn flush_file(
    files: &mut Vec<ParsedDiff>,
    current_file: &mut Option<String>,
    current_hunks: &mut Vec<Hunk>,
) {
    if let Some(f) = current_file.take() {
        files.push(ParsedDiff {
            file: f,
            hunks: std::mem::take(current_hunks),
        });
    }
}

/// 把一个 hunk 应用到当前内容：定位 old 行、替换为 new 行。
/// 返回改写后的完整内容（保留原内容末尾是否带 `\n` 的语义）。
pub fn apply_hunks_to_content(content: &str, hunks: &[Hunk]) -> Result<String, String> {
    let trailing_nl = content.ends_with('\n');
    let parts: Vec<&str> = if content.is_empty() {
        Vec::new()
    } else {
        // 去掉末尾换行后再 split，避免 trailing 产生空元素
        let trimmed = content.trim_end_matches('\n');
        trimmed.split('\n').collect()
    };
    let lines: Vec<String> = parts.into_iter().map(|s| s.to_string()).collect();
    let mut out: Vec<String> = lines.clone();
    // hunk 必须按 old_start 升序；倒序逐个应用（从后往前改，避免行号偏移）。
    let mut ordered: Vec<&Hunk> = hunks.iter().collect();
    ordered.sort_by_key(|h| h.old_start);
    for h in ordered.iter().rev() {
        let start = (h.old_start as usize).saturating_sub(1);
        if start + h.old_lines_content.len() > out.len() {
            return Err(format!(
                "hunk @@ -{},{} @@ 上下文越界（文件共 {} 行）",
                h.old_start,
                h.old_lines_content.len(),
                out.len()
            ));
        }
        for (i, expected) in h.old_lines_content.iter().enumerate() {
            if out[start + i] != *expected {
                return Err(format!(
                    "hunk @@ -{},{} @@ 上下文不匹配：第 {} 行应为 {:?}，实为 {:?}",
                    h.old_start,
                    h.old_lines_content.len(),
                    start + i + 1,
                    expected,
                    out[start + i]
                ));
            }
        }
        let mut new_block = h.new_lines_content.clone();
        out.splice(
            start..start + h.old_lines_content.len(),
            new_block.drain(..),
        );
    }
    let mut joined = out.join("\n");
    if trailing_nl {
        joined.push('\n');
    }
    Ok(joined)
}

/// 敏感路径判定（不读内容）：.env / 私钥 / 构建产物 / 版本控制目录。
/// 阶段④只在写入前调一次。
pub fn is_sensitive_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    if is_sensitive(
        Path::new(&lower)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(""),
    ) {
        return true;
    }
    let segments = lower.split(['/', '\\']).collect::<Vec<_>>();
    for seg in &segments {
        if *seg == ".env"
            || seg.starts_with(".env.")
            || seg.ends_with(".pem")
            || seg.ends_with(".key")
            || seg.ends_with(".p12")
            || seg.ends_with(".pfx")
            || seg.ends_with(".jks")
            || seg.ends_with(".keystore")
        {
            return true;
        }
    }
    for seg in &segments {
        if matches!(*seg, "node_modules" | "target" | ".git" | ".ssh" | ".aws") {
            return true;
        }
    }
    false
}

/// 项目内文件路径规范化 + 敏感路径拒收。
/// root：项目根；rel：diff 头解析出的相对路径。
pub fn resolve_target_path(root: &Path, rel: &str) -> Result<(PathBuf, String), String> {
    if is_sensitive_path(rel) {
        return Err(format!("敏感路径，已拒收: {rel}"));
    }
    resolve_in_root(root, rel)
}

/// sha256 摘要：用于快照完整性 + 写入冲突指纹。
/// 用 `sha2` crate（Cargo.lock 已由 reqwest 等传递依赖引入，0.10.9）；本 crate 直接复用避免手写 bug。
pub fn sha256_hex(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(content.as_bytes());
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

/// 把解析后的 patch 转成预览（每文件读取当前内容 + 算出 new_content）。
pub fn build_preview(root: &Path, parsed: &ParsedPatch) -> Result<PatchPreview, String> {
    let mut files = Vec::with_capacity(parsed.files.len());
    let mut warnings = Vec::new();
    for f in &parsed.files {
        if is_sensitive_path(&f.file) {
            warnings.push(format!("{} 命中敏感路径黑名单，已拒收", f.file));
            continue;
        }
        let (abs, rel) = match resolve_in_root(root, &f.file) {
            Ok(v) => v,
            Err(e) => {
                warnings.push(format!("{} 路径解析失败：{}，已拒收", f.file, e));
                continue;
            }
        };
        let current = if abs.exists() {
            fs::read_to_string(&abs).unwrap_or_default()
        } else {
            String::new()
        };
        let new_content = match apply_hunks_to_content(&current, &f.hunks) {
            Ok(s) => s,
            Err(e) => {
                warnings.push(format!("{} hunk 应用失败：{}", f.file, e));
                continue;
            }
        };
        files.push(PatchFilePreview {
            file: rel,
            current_content: current,
            new_content,
            hunks: f.hunks.clone(),
            rejected: false,
        });
    }
    Ok(PatchPreview { files, warnings })
}

/// 把多个文件预览应用到磁盘；写入前先做整文件快照，返回 ApplyResult。
/// rejected=true 的文件跳过；空白 new_content 视为空文件（仍写）。
pub fn apply_preview(
    root: &Path,
    previews: &[PatchFilePreview],
    user_root: &Path,
    project_id: &str,
) -> Result<ApplyResult, String> {
    let mut applied = Vec::new();
    let mut skipped = Vec::new();
    let mut entries = Vec::new();
    for p in previews {
        if p.rejected {
            skipped.push(p.file.clone());
            continue;
        }
        if is_sensitive_path(&p.file) {
            skipped.push(p.file.clone());
            continue;
        }
        let (abs, _) = match resolve_in_root(root, &p.file) {
            Ok(v) => v,
            Err(_) => {
                skipped.push(p.file.clone());
                continue;
            }
        };
        // 写前快照：原文件（若存在）
        let original_content = if abs.exists() {
            fs::read_to_string(&abs).unwrap_or_default()
        } else {
            String::new()
        };
        let sha = sha256_hex(&original_content);
        // 大文件上限：原内容超过 10 MB 拒收
        if original_content.len() > 10 * 1024 * 1024 {
            skipped.push(p.file.clone());
            continue;
        }
        if let Some(parent) = abs.parent() {
            if !parent.exists() && fs::create_dir_all(parent).is_err() {
                skipped.push(p.file.clone());
                continue;
            }
        }
        if fs::write(&abs, p.new_content.as_bytes()).is_err() {
            skipped.push(p.file.clone());
            continue;
        }
        entries.push(SnapshotEntry {
            path: p.file.clone(),
            original_content,
            sha256: sha,
        });
        applied.push(p.file.clone());
    }
    if applied.is_empty() {
        return Err("没有任何文件被写入".into());
    }
    // 写快照
    let snap_path = snapshot_path(user_root)?;
    let mut all = load_snapshots(&snap_path).unwrap_or_default();
    let id = format!("snap-{}-{}", now_secs(), entries.len());
    let snap = PatchSnapshot {
        id: id.clone(),
        project_id: project_id.to_string(),
        entries,
        applied_at: now_secs(),
    };
    all.push(snap);
    save_snapshots(&snap_path, &all)?;
    Ok(ApplyResult {
        applied,
        skipped,
        snapshot_id: id,
    })
}

/// 在指定项目根下撤销快照（写入原始内容并从快照列表中移除）。
pub fn revert_snapshot_in(
    project_root: &Path,
    user_root: &Path,
    snapshot_id: &str,
) -> Result<RevertResult, String> {
    let snap_path = snapshot_path(user_root)?;
    let mut all = load_snapshots(&snap_path).unwrap_or_default();
    let pos = all
        .iter()
        .position(|s| s.id == snapshot_id)
        .ok_or_else(|| format!("找不到快照 {snapshot_id}"))?;
    let snap = all.remove(pos);
    let mut restored = Vec::new();
    for entry in &snap.entries {
        let (abs, _) = match resolve_in_root(project_root, &entry.path) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(parent) = abs.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::write(&abs, entry.original_content.as_bytes()).is_ok() {
            restored.push(entry.path.clone());
        }
    }
    save_snapshots(&snap_path, &all)?;
    Ok(RevertResult {
        restored,
        snapshot_id: snap.id,
    })
}

/// 加载全部快照（损坏文件返回空 vec，不抛错——和 RecentStore 行为一致）。
pub fn load_snapshots(path: &Path) -> Result<Vec<PatchSnapshot>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("读快照失败: {e}"))?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&raw).map_err(|e| format!("解析快照失败: {e}"))
}

pub fn save_snapshots(path: &Path, snaps: &[PatchSnapshot]) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(snaps).map_err(|e| format!("序列化失败: {e}"))?;
    fs::write(path, raw).map_err(|e| format!("写快照失败: {e}"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_legal_single_file_single_hunk() {
        let text =
            "--- a/src/foo.rs\n+++ b/src/foo.rs\n@@ -1,3 +1,3 @@\n line a\n-old\n+new\n line b\n";
        let p = parse_unified_diff(text).unwrap();
        assert_eq!(p.files.len(), 1);
        assert_eq!(p.files[0].file, "src/foo.rs");
        assert_eq!(p.files[0].hunks.len(), 1);
        let h = &p.files[0].hunks[0];
        assert_eq!(h.old_start, 1);
        assert_eq!(h.old_lines, 3);
        assert_eq!(h.new_start, 1);
        assert_eq!(h.new_lines, 3);
        // 输入末尾 \n 不应产生额外空上下文行
        assert_eq!(h.old_lines_content, vec!["line a", "old", "line b"]);
        assert_eq!(h.new_lines_content, vec!["line a", "new", "line b"]);
    }

    #[test]
    fn parse_rejects_no_path_header() {
        let text = "@@ -1,1 +1,1 @@\n-old\n+new\n";
        assert!(parse_unified_diff(text).is_err());
    }

    #[test]
    fn parse_rejects_binary_marker() {
        // 模拟 git binary diff：含 "GIT binary patch" 字符。简化实现忽略它但没 hunk。
        let text = "--- a/img.png\n+++ b/img.png\nGIT binary patch\n";
        let err = parse_unified_diff(text).unwrap_err();
        assert!(err.contains("未识别到任何 diff 块") || err.contains("未包含可应用"));
    }

    #[test]
    fn parse_multi_file() {
        let text = "--- a/a.rs\n+++ b/a.rs\n@@ -1,1 +1,1 @@\n-oldA\n+newA\n--- a/b.rs\n+++ b/b.rs\n@@ -1,1 +1,1 @@\n-oldB\n+newB\n";
        let p = parse_unified_diff(text).unwrap();
        assert_eq!(p.files.len(), 2);
        assert_eq!(p.files[0].file, "a.rs");
        assert_eq!(p.files[1].file, "b.rs");
    }

    #[test]
    fn apply_hunks_match() {
        let content = "alpha\nbeta\ngamma\n";
        let hunks = vec![Hunk {
            old_start: 2,
            old_lines: 1,
            new_start: 2,
            new_lines: 2,
            old_lines_content: vec!["beta".into()],
            new_lines_content: vec!["beta".into(), "delta".into()],
        }];
        let out = apply_hunks_to_content(content, &hunks).unwrap();
        // 原内容以 \n 结尾，结果保留 trailing newline
        assert_eq!(out, "alpha\nbeta\ndelta\ngamma\n");
    }

    #[test]
    fn apply_hunks_context_mismatch() {
        let content = "alpha\nBETA\ngamma\n";
        let hunks = vec![Hunk {
            old_start: 2,
            old_lines: 1,
            new_start: 2,
            new_lines: 1,
            old_lines_content: vec!["beta".into()],
            new_lines_content: vec!["new".into()],
        }];
        let err = apply_hunks_to_content(content, &hunks).unwrap_err();
        assert!(err.contains("上下文不匹配"));
    }

    #[test]
    fn apply_hunks_empty_file() {
        let content = "";
        let hunks = vec![Hunk {
            old_start: 1,
            old_lines: 0,
            new_start: 1,
            new_lines: 2,
            old_lines_content: vec![],
            new_lines_content: vec!["first".into(), "second".into()],
        }];
        let out = apply_hunks_to_content(content, &hunks).unwrap();
        assert_eq!(out, "first\nsecond");
    }

    #[test]
    fn sensitive_env() {
        assert!(is_sensitive_path(".env"));
        assert!(is_sensitive_path("config/.env.production"));
        assert!(is_sensitive_path("keys/id_rsa"));
        assert!(is_sensitive_path("certs/server.pem"));
        assert!(is_sensitive_path("web/node_modules/foo/bar.js"));
        assert!(is_sensitive_path("rust/target/release/binary"));
        assert!(!is_sensitive_path("src/main.rs"));
    }

    #[test]
    fn sha256_basic() {
        assert_eq!(
            sha256_hex(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn snapshot_roundtrip() {
        // 进程级串行：ELWRIGHT_USER_ROOT 修改需持锁
        let _g = super::super::test_env::env_serialization_guard();
        let tmp = std::env::temp_dir().join(format!(
            "elwright-patch-test-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let s = PatchSnapshot {
            id: "snap-1".into(),
            project_id: "p1".into(),
            entries: vec![SnapshotEntry {
                path: "src/foo.rs".into(),
                original_content: "old".into(),
                sha256: sha256_hex("old"),
            }],
            applied_at: now_secs(),
        };
        let p = snapshot_path(&tmp).unwrap();
        save_snapshots(&p, std::slice::from_ref(&s)).unwrap();
        let loaded = load_snapshots(&p).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "snap-1");
        std::fs::remove_dir_all(&tmp).ok();
    }

    use std::sync::atomic::AtomicU64;
    static SEQ: AtomicU64 = AtomicU64::new(0);
}
