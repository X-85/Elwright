//! 脑图（mindmap MVP，ADR-001）。
//!
//! 数据模型：扁平 `Vec<MindNode>` + `parent` 指针，**Vec 顺序即 DFS 文档序**
//! （子树连续），所有树操作（增删/移动/缩进）都维持这一不变量——
//! 序列化稳定、渲染直接按序缩进、子树操作退化为连续区间处理。
//!
//! 存储：`~/.elwright/mindmaps/<id>.json` 一图一文件（与 chats 同模式），
//! tmp+rename 原子写；list 按更新时间倒序，损坏单文件跳过不拖累其他图。
//!
//! 弃选：嵌套 JSON 树（序列化抖动）、localStorage（配额/清理风险、CLI 不可读）。
//! 详见 features/mindmap/decisions/ADR-001-mindmap-mvp.md §D2/D3。

use std::path::{Path, PathBuf};

/// 单个节点。`parent: None` 表示根（每图恰好一个，恒为 nodes[0]）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MindNode {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub collapsed: bool,
    /// 已「转为 Todo」标记（关联工作台，ADR-001 §D4）
    #[serde(default)]
    pub converted_todo: bool,
}

/// 一张脑图文档。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MindmapDoc {
    pub id: String,
    pub title: String,
    pub nodes: Vec<MindNode>,
    pub updated_at: i64,
}

/// 列表视图（不含节点明细）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MindmapSummary {
    pub id: String,
    pub title: String,
    pub updated_at: i64,
    pub node_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum MindmapError {
    #[error("脑图目录创建失败：{0}")]
    DirCreate(String),
    #[error("脑图文件读取失败：{0}")]
    FileRead(String),
    #[error("脑图文件写入失败：{0}")]
    FileWrite(String),
    #[error("序列化失败：{0}")]
    Serialize(String),
    #[error("脑图不存在：{0}")]
    NotFound(String),
    #[error("非法操作：{0}")]
    Invalid(String),
}

// ---------- 纯函数树操作（Vec 顺序 = DFS 序，子树连续） ----------

/// 节点深度（根 = 0）。
pub fn depth_of(nodes: &[MindNode], id: &str) -> Option<usize> {
    let mut current = nodes.iter().find(|n| n.id == id)?;
    let mut depth = 0;
    while let Some(p) = &current.parent {
        depth += 1;
        if depth > nodes.len() {
            return None; // 环防御
        }
        current = nodes.iter().find(|n| &n.id == p)?;
    }
    Some(depth)
}

/// `idx` 处节点的子树结束位置（不含）——子树在 DFS 序中连续。
fn subtree_end(nodes: &[MindNode], idx: usize) -> usize {
    let base = depth_of(nodes, &nodes[idx].id).unwrap_or(0);
    let mut end = idx + 1;
    while end < nodes.len() {
        let d = depth_of(nodes, &nodes[end].id).unwrap_or(0);
        if d <= base {
            break;
        }
        end += 1;
    }
    end
}

/// 前一个兄弟的 idx（同 parent、紧邻目标子树块之前）。
fn prev_sibling_idx(nodes: &[MindNode], idx: usize) -> Option<usize> {
    let parent = nodes[idx].parent.clone();
    let mut i = idx;
    while i > 0 {
        i -= 1;
        if nodes[i].parent == parent {
            return Some(i);
        }
        // 越过更深的子树块继续向前
        if depth_of(nodes, &nodes[i].id).unwrap_or(0) == 0 {
            break;
        }
    }
    None
}

/// 后一个兄弟的 idx。
fn next_sibling_idx(nodes: &[MindNode], idx: usize) -> Option<usize> {
    let end = subtree_end(nodes, idx);
    let parent = nodes[idx].parent.clone();
    if end < nodes.len() && nodes[end].parent == parent {
        Some(end)
    } else {
        None
    }
}

/// 在 `at` 位置插入节点（批量安全：Vec 插入保持其余相对顺序）。
fn insert_node(nodes: &mut Vec<MindNode>, at: usize, node: MindNode) {
    nodes.insert(at, node);
}

/// 加兄弟节点（目标之后），返回新节点 id。
pub fn add_sibling(
    nodes: &mut Vec<MindNode>,
    target_id: &str,
    text: String,
    new_id: String,
) -> Result<(), MindmapError> {
    let idx = nodes
        .iter()
        .position(|n| n.id == target_id)
        .ok_or_else(|| MindmapError::NotFound(target_id.into()))?;
    if depth_of(nodes, target_id).unwrap_or(0) == 0 {
        return Err(MindmapError::Invalid("根节点不能加兄弟".into()));
    }
    let node = MindNode {
        id: new_id,
        text,
        parent: nodes[idx].parent.clone(),
        collapsed: false,
        converted_todo: false,
    };
    insert_node(nodes, subtree_end(nodes, idx), node);
    Ok(())
}

/// 加子节点（作为最后一个子节点），返回新节点 id。
pub fn add_child(
    nodes: &mut Vec<MindNode>,
    parent_id: &str,
    text: String,
    new_id: String,
) -> Result<(), MindmapError> {
    let idx = nodes
        .iter()
        .position(|n| n.id == parent_id)
        .ok_or_else(|| MindmapError::NotFound(parent_id.into()))?;
    let node = MindNode {
        id: new_id,
        text,
        parent: Some(parent_id.to_string()),
        collapsed: false,
        converted_todo: false,
    };
    insert_node(nodes, subtree_end(nodes, idx), node);
    Ok(())
}

/// 删除节点及其整棵子树（根不可删）。
pub fn remove_subtree(nodes: &mut Vec<MindNode>, id: &str) -> Result<usize, MindmapError> {
    let idx = nodes
        .iter()
        .position(|n| n.id == id)
        .ok_or_else(|| MindmapError::NotFound(id.into()))?;
    if depth_of(nodes, id).unwrap_or(0) == 0 {
        return Err(MindmapError::Invalid("根节点不能删除".into()));
    }
    let end = subtree_end(nodes, idx);
    nodes.drain(idx..end);
    Ok(end - idx)
}

/// 上移/下移：与相邻兄弟的子树块整体交换。返回是否发生移动。
pub fn move_vertical(nodes: &mut Vec<MindNode>, id: &str, up: bool) -> Result<bool, MindmapError> {
    let idx = nodes
        .iter()
        .position(|n| n.id == id)
        .ok_or_else(|| MindmapError::NotFound(id.into()))?;
    if depth_of(nodes, id).unwrap_or(0) == 0 {
        return Err(MindmapError::Invalid("根节点不能移动".into()));
    }
    let (a, b) = if up {
        match prev_sibling_idx(nodes, idx) {
            Some(p) => (p, idx),
            None => return Ok(false),
        }
    } else {
        match next_sibling_idx(nodes, idx) {
            Some(n) => (idx, n),
            None => return Ok(false),
        }
    };
    let end_a = subtree_end(nodes, a);
    let end_b = subtree_end(nodes, b);
    // 提取两块再按交换后的顺序拼回（a 块与 b 块中间无其他内容）
    let block_a: Vec<MindNode> = nodes[a..end_a].to_vec();
    let block_b: Vec<MindNode> = nodes[end_a..end_b].to_vec();
    // 交换相邻两块（up: a=前兄弟,b=目标；down: a=目标,b=后兄弟——语义已由选择确定，
    // 拼接顺序统一为「后块在前」）
    let mut rebuilt = Vec::with_capacity(nodes.len());
    rebuilt.extend_from_slice(&nodes[..a]);
    rebuilt.extend(block_b);
    rebuilt.extend(block_a);
    rebuilt.extend_from_slice(&nodes[end_b..]);
    *nodes = rebuilt;
    Ok(true)
}

/// 缩进：前一个兄弟变为父节点（挂为其最后一个子节点）。
pub fn indent(nodes: &mut Vec<MindNode>, id: &str) -> Result<(), MindmapError> {
    let idx = nodes
        .iter()
        .position(|n| n.id == id)
        .ok_or_else(|| MindmapError::NotFound(id.into()))?;
    if depth_of(nodes, id).unwrap_or(0) == 0 {
        return Err(MindmapError::Invalid("根节点不能缩进".into()));
    }
    let new_parent = prev_sibling_idx(nodes, idx)
        .ok_or_else(|| MindmapError::Invalid("没有前一个兄弟，无法缩进".into()))?;
    let new_parent_id = nodes[new_parent].id.clone();
    // 整块移动到新父的子树末尾
    let end = subtree_end(nodes, idx);
    let block: Vec<MindNode> = nodes.drain(idx..end).collect();
    let np_idx = nodes.iter().position(|n| n.id == new_parent_id).unwrap();
    let at = subtree_end(nodes, np_idx);
    nodes.splice(at..at, block);
    // 改父指针（块内只有首节点 parent 变化）
    nodes[at].parent = Some(new_parent_id);
    Ok(())
}

/// 外提：父节点变为祖父，节点块移动到原父之后。
pub fn outdent(nodes: &mut Vec<MindNode>, id: &str) -> Result<(), MindmapError> {
    let idx = nodes
        .iter()
        .position(|n| n.id == id)
        .ok_or_else(|| MindmapError::NotFound(id.into()))?;
    let parent_id = nodes[idx]
        .parent
        .clone()
        .ok_or_else(|| MindmapError::Invalid("根节点不能外提".into()))?;
    let p_idx = nodes.iter().position(|n| n.id == parent_id).unwrap();
    let grandparent = nodes[p_idx].parent.clone();
    if grandparent.is_none() {
        return Err(MindmapError::Invalid(
            "一级节点不能再外提（会脱离根）".into(),
        ));
    }
    let end = subtree_end(nodes, idx);
    let block: Vec<MindNode> = nodes.drain(idx..end).collect();
    let p_idx_new = nodes.iter().position(|n| n.id == parent_id).unwrap();
    let at = subtree_end(nodes, p_idx_new);
    nodes.splice(at..at, block);
    nodes[at].parent = grandparent;
    Ok(())
}

// ---------- 持久化 ----------

/// 脑图目录：`<user_root>/mindmaps/`。
pub fn mindmaps_dir(user_root: &Path) -> PathBuf {
    user_root.join("mindmaps")
}

fn new_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    format!("m{:x}{:02x}", millis, seq & 0xff)
}

/// 新建一张图：仅含根节点（根文本 = 标题）。
pub fn new_doc(title: &str) -> MindmapDoc {
    let id = new_id();
    let root_id = format!("{}-root", id);
    MindmapDoc {
        id: id.clone(),
        title: title.to_string(),
        nodes: vec![MindNode {
            id: root_id,
            text: title.to_string(),
            parent: None,
            collapsed: false,
            converted_todo: false,
        }],
        updated_at: current_unix_secs(),
    }
}

/// 列出全部脑图（按 updatedAt 倒序）；损坏文件跳过并返回其文件名列表。
pub fn list(user_root: &Path) -> Result<(Vec<MindmapSummary>, Vec<String>), MindmapError> {
    let dir = mindmaps_dir(user_root);
    std::fs::create_dir_all(&dir)
        .map_err(|e| MindmapError::DirCreate(format!("{}: {}", dir.display(), e)))?;
    let mut out = Vec::new();
    let mut corrupt = Vec::new();
    let entries = std::fs::read_dir(&dir)
        .map_err(|e| MindmapError::FileRead(format!("{}: {}", dir.display(), e)))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => {
                corrupt.push(
                    path.file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                );
                continue;
            }
        };
        match serde_json::from_str::<MindmapDoc>(&text) {
            Ok(doc) => out.push(MindmapSummary {
                id: doc.id,
                title: doc.title,
                updated_at: doc.updated_at,
                node_count: doc.nodes.len(),
            }),
            Err(_) => corrupt.push(
                path.file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
        }
    }
    out.sort_by(|a, b| {
        b.updated_at
            .cmp(&a.updated_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok((out, corrupt))
}

/// 加载单张图。
pub fn load(user_root: &Path, id: &str) -> Result<MindmapDoc, MindmapError> {
    let path = mindmaps_dir(user_root).join(format!("{}.json", sanitize_id(id)));
    let text = std::fs::read_to_string(&path)
        .map_err(|e| MindmapError::FileRead(format!("{}: {}", path.display(), e)))?;
    serde_json::from_str(&text)
        .map_err(|e| MindmapError::FileRead(format!("{} 解析失败: {}", path.display(), e)))
}

/// 保存（全量覆盖，原子写）。
pub fn save(user_root: &Path, doc: &mut MindmapDoc) -> Result<(), MindmapError> {
    if doc.nodes.is_empty() {
        return Err(MindmapError::Invalid("脑图至少需要一个根节点".into()));
    }
    doc.updated_at = current_unix_secs();
    let dir = mindmaps_dir(user_root);
    std::fs::create_dir_all(&dir)
        .map_err(|e| MindmapError::DirCreate(format!("{}: {}", dir.display(), e)))?;
    let path = dir.join(format!("{}.json", sanitize_id(&doc.id)));
    let text =
        serde_json::to_string_pretty(doc).map_err(|e| MindmapError::Serialize(e.to_string()))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, text + "\n")
        .map_err(|e| MindmapError::FileWrite(format!("{}: {}", tmp.display(), e)))?;
    std::fs::rename(&tmp, &path).map_err(|e| {
        MindmapError::FileWrite(format!("{} → {}: {}", tmp.display(), path.display(), e))
    })?;
    Ok(())
}

/// 删除一张图。
pub fn delete(user_root: &Path, id: &str) -> Result<(), MindmapError> {
    let path = mindmaps_dir(user_root).join(format!("{}.json", sanitize_id(id)));
    std::fs::remove_file(&path)
        .map_err(|e| MindmapError::FileWrite(format!("{}: {}", path.display(), e)))
}

/// id 只允许字母数字与 - _（防路径穿越）。
fn sanitize_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

fn current_unix_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ---------- 测试 ----------

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造：根(A) → A1, A2(→A2a)；B → B1
    fn fixture() -> Vec<MindNode> {
        let n = |id: &str, parent: Option<&str>| MindNode {
            id: id.into(),
            text: id.into(),
            parent: parent.map(|p| p.into()),
            collapsed: false,
            converted_todo: false,
        };
        vec![
            n("root", None),
            n("a1", Some("root")),
            n("a2", Some("root")),
            n("a2a", Some("a2")),
            n("b1", Some("root")),
        ]
    }

    fn ids(nodes: &[MindNode]) -> Vec<&str> {
        nodes.iter().map(|n| n.id.as_str()).collect()
    }

    #[test]
    fn add_sibling_and_child_maintain_dfs_order() {
        let mut nodes = fixture();
        add_sibling(&mut nodes, "a1", "a1b".into(), "a1b".into()).unwrap();
        assert_eq!(ids(&nodes), vec!["root", "a1", "a1b", "a2", "a2a", "b1"]);
        add_child(&mut nodes, "a1b", "a1bx".into(), "a1bx".into()).unwrap();
        assert_eq!(
            ids(&nodes),
            vec!["root", "a1", "a1b", "a1bx", "a2", "a2a", "b1"]
        );
        // a1b 是 root 的子（a1 的兄弟），深度 1；a1bx 深度 2
        assert_eq!(depth_of(&nodes, "a1b"), Some(1));
        assert_eq!(depth_of(&nodes, "a1bx"), Some(2));
    }

    #[test]
    fn remove_subtree_cascades_and_refuses_root() {
        let mut nodes = fixture();
        assert!(matches!(
            remove_subtree(&mut nodes, "root"),
            Err(MindmapError::Invalid(_))
        ));
        let removed = remove_subtree(&mut nodes, "a2").unwrap();
        assert_eq!(removed, 2, "a2 + a2a");
        assert_eq!(ids(&nodes), vec!["root", "a1", "b1"]);
    }

    #[test]
    fn move_vertical_swaps_whole_blocks() {
        let mut nodes = fixture();
        // a2（含子树 a2a）上移到 a1 之前
        assert!(move_vertical(&mut nodes, "a2", true).unwrap());
        assert_eq!(ids(&nodes), vec!["root", "a2", "a2a", "a1", "b1"]);
        // b1 上移：与 a1（无子树）交换
        assert!(move_vertical(&mut nodes, "b1", true).unwrap());
        assert_eq!(ids(&nodes), vec!["root", "a2", "a2a", "b1", "a1"]);
        // a1 已是最后兄弟，再上移一次回到 a1 在 b1 前
        assert!(move_vertical(&mut nodes, "a1", true).unwrap());
        assert_eq!(ids(&nodes), vec!["root", "a2", "a2a", "a1", "b1"]);
        // 根不能动
        assert!(matches!(
            move_vertical(&mut nodes, "root", true),
            Err(MindmapError::Invalid(_))
        ));
    }

    #[test]
    fn indent_outdent_keep_subtrees_together() {
        let mut nodes = fixture();
        // a1 缩进 → 成为 root 下… 不对：a1 的前一个兄弟不存在（它是第一个兄弟）→ Err
        assert!(matches!(
            indent(&mut nodes, "a1"),
            Err(MindmapError::Invalid(_))
        ));
        // a2 缩进 → 挂到 a1 下
        indent(&mut nodes, "a2").unwrap();
        assert_eq!(
            nodes
                .iter()
                .find(|n| n.id == "a2")
                .unwrap()
                .parent
                .as_deref(),
            Some("a1")
        );
        assert_eq!(ids(&nodes), vec!["root", "a1", "a2", "a2a", "b1"]);
        assert_eq!(depth_of(&nodes, "a2a"), Some(3));
        // a2 外提 → 回到 root 下
        outdent(&mut nodes, "a2").unwrap();
        assert_eq!(
            nodes
                .iter()
                .find(|n| n.id == "a2")
                .unwrap()
                .parent
                .as_deref(),
            Some("root")
        );
        assert_eq!(ids(&nodes), vec!["root", "a1", "a2", "a2a", "b1"]);
        // 一级节点外提被拒（b1 的父是 root，root 无父）
        assert!(matches!(
            outdent(&mut nodes, "a1"),
            Err(MindmapError::Invalid(_))
        ));
    }

    #[test]
    fn persist_roundtrip_list_order_and_corrupt_tolerance() {
        let dir = tempfile::tempdir().unwrap();
        let mut doc = new_doc("部署思路");
        // 挂两个子节点
        let root_id = doc.nodes[0].id.clone();
        add_child(&mut doc.nodes, &root_id, "准备工作".into(), "c1".into()).unwrap();
        add_sibling(&mut doc.nodes, "c1", "执行".into(), "c2".into()).unwrap();
        save(dir.path(), &mut doc).unwrap();

        // 第二张图（更新时间更晚或相同）
        let mut doc2 = new_doc("会议纪要");
        save(dir.path(), &mut doc2).unwrap();

        let (summaries, corrupt) = list(dir.path()).unwrap();
        assert!(corrupt.is_empty());
        assert_eq!(summaries.len(), 2);
        // doc（root+c1+c2）3 节点；doc2（仅 root）1 节点
        let counts: Vec<usize> = summaries.iter().map(|s| s.node_count).collect();
        assert!(counts.contains(&3) && counts.contains(&1));

        // 损坏文件容忍
        std::fs::write(dir.path().join("mindmaps").join("broken.json"), "{bad").unwrap();
        let (_s, corrupt) = list(dir.path()).unwrap();
        assert_eq!(corrupt.len(), 1);

        let loaded = load(dir.path(), &doc.id).unwrap();
        assert_eq!(loaded, doc);
        // id 消毒：路径穿越字符被过滤
        assert!(load(dir.path(), "../../etc/passwd").is_err());
        delete(dir.path(), &doc.id).unwrap();
        assert!(matches!(
            load(dir.path(), &doc.id),
            Err(MindmapError::FileRead(_))
        ));
    }
}
