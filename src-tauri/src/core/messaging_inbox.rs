//! Messaging 收件箱（ADR-003 §D3）。
//!
//! listener 线程收到并解密的对端消息先落 `inbox.jsonl`（本地静态密钥加密，
//! AAD 绑定对端 ID），前端经 `messaging_poll_inbox(since_id)` 增量取走后
//! 合并进阶段①的 localStorage 会话模型。cursor 由前端持久化。

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::local_crypto;

#[derive(Debug, thiserror::Error)]
pub enum InboxError {
    #[error("收件箱目录创建失败：{0}")]
    DirCreate(String),
    #[error("收件箱文件读取失败：{0}")]
    FileRead(String),
    #[error("收件箱文件写入失败：{0}")]
    FileWrite(String),
    #[error("序列化失败：{0}")]
    Serialize(String),
    #[error("消息内容不能为空")]
    EmptyPayload,
}

/// 落盘形态（payload 为本地密钥密文 hex）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InboxEntry {
    pub id: u64,
    /// 发送方 ID
    pub peer_id: String,
    /// `local_crypto::encrypt(key, aad=peer_id, text)`
    pub payload_hex: String,
    pub received_at: i64,
}

/// 前端视图形态（已解密）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxItem {
    pub id: u64,
    pub peer_id: String,
    pub text: String,
    pub received_at: i64,
}

pub struct Inbox {
    path: PathBuf,
}

impl Inbox {
    pub fn open(dir: &Path) -> Result<Self, InboxError> {
        std::fs::create_dir_all(dir)
            .map_err(|e| InboxError::DirCreate(format!("{}: {}", dir.display(), e)))?;
        Ok(Self {
            path: dir.join("inbox.jsonl"),
        })
    }

    /// 追加一条收到的文本消息（内部加密落盘）。
    pub fn append(
        &self,
        local_key: &[u8; 32],
        peer_id: &str,
        text: &str,
    ) -> Result<InboxEntry, InboxError> {
        if text.is_empty() {
            return Err(InboxError::EmptyPayload);
        }
        let id = self.max_id()?.wrapping_add(1);
        let entry = InboxEntry {
            id,
            peer_id: peer_id.to_string(),
            payload_hex: local_crypto::encrypt(local_key, peer_id, text.as_bytes()),
            received_at: current_unix_secs(),
        };
        let line =
            serde_json::to_string(&entry).map_err(|e| InboxError::Serialize(e.to_string()))?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| InboxError::FileWrite(format!("{}: {}", self.path.display(), e)))?;
        writeln!(f, "{}", line)
            .map_err(|e| InboxError::FileWrite(format!("{}: {}", self.path.display(), e)))?;
        Ok(entry)
    }

    /// 当前最大 id（空箱为 0）——前端 cursor 初值。
    pub fn max_id(&self) -> Result<u64, InboxError> {
        Ok(self.read_all()?.iter().map(|e| e.id).max().unwrap_or(0))
    }

    /// 增量取走 `since_id` 之后的条目（已解密，FIFO）。损坏行跳过。
    pub fn poll(&self, local_key: &[u8; 32], since_id: u64) -> Result<Vec<InboxItem>, InboxError> {
        let mut items: Vec<InboxItem> = self
            .read_all()?
            .into_iter()
            .filter(|e| e.id > since_id)
            .filter_map(|e| {
                let bytes = local_crypto::decrypt(local_key, &e.peer_id, &e.payload_hex).ok()?;
                let text = String::from_utf8(bytes).ok()?;
                Some(InboxItem {
                    id: e.id,
                    peer_id: e.peer_id,
                    text,
                    received_at: e.received_at,
                })
            })
            .collect();
        items.sort_by_key(|i| i.id);
        Ok(items)
    }

    fn read_all(&self) -> Result<Vec<InboxEntry>, InboxError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(InboxError::FileRead(format!(
                    "{}: {}",
                    self.path.display(),
                    e
                )))
            }
        };
        Ok(text
            .lines()
            .filter_map(|l| serde_json::from_str::<InboxEntry>(l).ok())
            .collect())
    }
}

fn current_unix_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn append_poll_roundtrip_and_cursor() {
        let key = [21u8; 32];
        let dir = tempdir().unwrap();
        let inbox = Inbox::open(&dir.path().join("msg")).unwrap();
        assert_eq!(inbox.max_id().unwrap(), 0);

        inbox.append(&key, "PEER_AAAAAAAA", "你好").unwrap();
        inbox.append(&key, "PEER_AAAAAAAA", "second").unwrap();
        inbox.append(&key, "PEER_BBBBBBBB", "from b").unwrap();
        assert_eq!(inbox.max_id().unwrap(), 3);

        let all = inbox.poll(&key, 0).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].text, "你好");
        assert_eq!(all[2].peer_id, "PEER_BBBBBBBB");

        // cursor 增量：只取 id>2
        let tail = inbox.poll(&key, 2).unwrap();
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].text, "from b");
    }

    #[test]
    fn plaintext_never_hits_disk() {
        let key = [22u8; 32];
        let dir = tempdir().unwrap();
        let inbox = Inbox::open(&dir.path().join("msg")).unwrap();
        inbox
            .append(&key, "PEER_CCCCCCCC", "inbox secret 中文")
            .unwrap();
        let raw = std::fs::read_to_string(&inbox.path).unwrap();
        assert!(
            !raw.contains("inbox secret"),
            "收件箱文件不得含明文：\n{}",
            raw
        );
        // 密钥不符解不出
        assert!(inbox.poll(&[23u8; 32], 0).unwrap().is_empty());
    }

    #[test]
    fn empty_text_rejected() {
        let key = [24u8; 32];
        let dir = tempdir().unwrap();
        let inbox = Inbox::open(&dir.path().join("msg")).unwrap();
        assert!(matches!(
            inbox.append(&key, "PEER_DDDDDDDD", ""),
            Err(InboxError::EmptyPayload)
        ));
    }
}
