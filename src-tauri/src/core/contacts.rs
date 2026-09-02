//! 联系人存储（ADR-003 §D1）。
//!
//! `~/.elwright/contacts.json`：接受邀请后经用户确认落盘。
//! DH 公钥在此持久化——后续每次握手用它校验 `remote_static`（防中间人）。

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    /// 对端 16 字符 base32 ID
    pub peer_id: String,
    /// 对端 ed25519 签名公钥（hex）
    pub signing_pub_hex: String,
    /// 对端 X25519 DH 公钥（hex）——握手 remote_static 校验基准
    pub dh_pub_hex: String,
    /// 本地备注名（可空）
    #[serde(default)]
    pub alias: String,
    /// 添加时间（unix 秒）
    pub added_at: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum ContactsError {
    #[error("联系人目录创建失败：{0}")]
    DirCreate(String),
    #[error("联系人文件读取失败：{0}")]
    FileRead(String),
    #[error("联系人文件写入失败：{0}")]
    FileWrite(String),
    #[error("序列化失败：{0}")]
    Serialize(String),
    #[error("联系人不存在：{0}")]
    NotFound(String),
    #[error("联系人已存在：{0}")]
    AlreadyExists(String),
}

/// 联系人文件路径：`<user_root>/contacts.json`。
pub fn contacts_path(user_root: &Path) -> PathBuf {
    user_root.join("contacts.json")
}

/// 列出全部联系人（按 peer_id 排序，稳定展示）。
pub fn list(user_root: &Path) -> Result<Vec<Contact>, ContactsError> {
    let path = contacts_path(user_root);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(ContactsError::FileRead(format!(
                "{}: {}",
                path.display(),
                e
            )))
        }
    };
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut contacts: Vec<Contact> = serde_json::from_str(&text)
        .map_err(|e| ContactsError::FileRead(format!("{} 解析失败: {}", path.display(), e)))?;
    contacts.sort_by(|a, b| a.peer_id.cmp(&b.peer_id));
    Ok(contacts)
}

/// 添加联系人（peer_id 重复则报错；alias 空则缺省为 peer_id 前 8 位）。
pub fn add(user_root: &Path, mut contact: Contact) -> Result<(), ContactsError> {
    let mut all = list(user_root)?;
    if all.iter().any(|c| c.peer_id == contact.peer_id) {
        return Err(ContactsError::AlreadyExists(contact.peer_id));
    }
    if contact.alias.is_empty() {
        let head: String = contact.peer_id.chars().take(8).collect();
        contact.alias = head;
    }
    all.push(contact);
    write(user_root, all)
}

/// 删除联系人。
pub fn remove(user_root: &Path, peer_id: &str) -> Result<(), ContactsError> {
    let all = list(user_root)?;
    if !all.iter().any(|c| c.peer_id == peer_id) {
        return Err(ContactsError::NotFound(peer_id.to_string()));
    }
    write(
        user_root,
        all.into_iter().filter(|c| c.peer_id != peer_id).collect(),
    )
}

/// 按 ID 取单个联系人。
pub fn get(user_root: &Path, peer_id: &str) -> Result<Option<Contact>, ContactsError> {
    Ok(list(user_root)?.into_iter().find(|c| c.peer_id == peer_id))
}

fn write(user_root: &Path, contacts: Vec<Contact>) -> Result<(), ContactsError> {
    let path = contacts_path(user_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ContactsError::DirCreate(format!("{}: {}", parent.display(), e)))?;
    }
    let text = serde_json::to_string_pretty(&contacts)
        .map_err(|e| ContactsError::Serialize(e.to_string()))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, text + "\n")
        .map_err(|e| ContactsError::FileWrite(format!("{}: {}", tmp.display(), e)))?;
    std::fs::rename(&tmp, &path).map_err(|e| {
        ContactsError::FileWrite(format!("{} → {}: {}", tmp.display(), path.display(), e))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample(id: &str) -> Contact {
        Contact {
            peer_id: id.to_string(),
            signing_pub_hex: "aa".repeat(32),
            dh_pub_hex: "bb".repeat(32),
            alias: String::new(),
            added_at: 1_700_000_000,
        }
    }

    #[test]
    fn add_list_remove_roundtrip() {
        let dir = tempdir().unwrap();
        assert!(list(dir.path()).unwrap().is_empty());
        add(dir.path(), sample("PEER01AAAAAAAAAA")).unwrap();
        add(dir.path(), sample("PEER02BBBBBBBBBB")).unwrap();
        // 重复添加拒绝
        assert!(matches!(
            add(dir.path(), sample("PEER01AAAAAAAAAA")),
            Err(ContactsError::AlreadyExists(_))
        ));
        let all = list(dir.path()).unwrap();
        assert_eq!(all.len(), 2);
        // 排序稳定
        assert!(all[0].peer_id < all[1].peer_id);
        // alias 自动填 ID 前 8 位
        assert_eq!(all[0].alias, "PEER01AA");
        // get / remove
        assert!(get(dir.path(), "PEER02BBBBBBBBBB").unwrap().is_some());
        remove(dir.path(), "PEER01AAAAAAAAAA").unwrap();
        assert!(get(dir.path(), "PEER01AAAAAAAAAA").unwrap().is_none());
        assert!(matches!(
            remove(dir.path(), "PEER01AAAAAAAAAA"),
            Err(ContactsError::NotFound(_))
        ));
    }

    #[test]
    fn corrupted_file_yields_error_not_panic() {
        let dir = tempdir().unwrap();
        std::fs::write(contacts_path(dir.path()), "{broken").unwrap();
        assert!(list(dir.path()).is_err());
    }
}
