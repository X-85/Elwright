//! Messaging offline queue（ADR-003 §D4 重构）。
//!
//! 存储格式：`<dir>/outbox.jsonl`，一行一条 JSON。**只存密文**——
//! 但与 step 5 版本不同：不再是会话密文（会话密钥不跨连接持久，重连后无法解密），
//! 而是「本地静态密钥加密的明文」（见 [`crate::core::local_crypto`]）。
//! flush 时解出明文，用当次握手的新会话 `Transport::send()` 重加密投递。
//!
//! 用法：发送一律先 `enqueue` 再触发 `sync_peer`（统一代码路径，离线安全）；
//! 投递成功 `remove`，失败 `record_attempt` 供退避/排查。
//!
//! 「明文不入盘」由单测强制（读原始文件字节断言）。

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::local_crypto;

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("队列目录创建失败：{0}")]
    DirCreate(String),
    #[error("队列文件读取失败：{0}")]
    FileRead(String),
    #[error("队列文件写入失败：{0}")]
    FileWrite(String),
    #[error("消息序列化失败：{0}")]
    Serialize(String),
    #[error("消息内容不能为空")]
    EmptyPayload,
    #[error("未找到队列条目：id={0}")]
    NotFound(u64),
    #[error("队列条目解密失败（本地密钥不符或内容被篡改）：id={0}")]
    DecryptFailed(u64),
}

/// 一条待投递消息（落盘形态，payload 为本地密钥密文 hex）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct QueuedMessage {
    pub id: u64,
    /// 目标对端 ID（16 字符 base32）
    pub peer_id: String,
    /// 入队时间（unix 秒；FIFO 排序键）
    pub created_at: i64,
    /// `local_crypto::encrypt(key, aad=peer_id, plaintext)` 输出
    pub payload_hex: String,
    /// 已尝试投递次数（失败重试时递增）
    pub attempts: u32,
}

pub struct Outbox {
    path: PathBuf,
}

impl Outbox {
    /// 打开（或首次创建）发件箱。`dir` 一般为 `~/.elwright/messaging/`。
    pub fn open(dir: &Path) -> Result<Self, QueueError> {
        std::fs::create_dir_all(dir)
            .map_err(|e| QueueError::DirCreate(format!("{}: {}", dir.display(), e)))?;
        Ok(Self {
            path: dir.join("outbox.jsonl"),
        })
    }

    /// 入队一条明文消息（内部加密落盘）。AAD 绑定 peer_id。
    pub fn enqueue(
        &self,
        local_key: &[u8; 32],
        peer_id: &str,
        plaintext: &[u8],
    ) -> Result<QueuedMessage, QueueError> {
        if plaintext.is_empty() {
            return Err(QueueError::EmptyPayload);
        }
        let existing = self.list(None)?;
        let id = existing.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        let msg = QueuedMessage {
            id,
            peer_id: peer_id.to_string(),
            created_at: current_unix_secs(),
            payload_hex: local_crypto::encrypt(local_key, peer_id, plaintext),
            attempts: 0,
        };
        let line = serde_json::to_string(&msg).map_err(|e| QueueError::Serialize(e.to_string()))?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| QueueError::FileWrite(format!("{}: {}", self.path.display(), e)))?;
        writeln!(f, "{}", line)
            .map_err(|e| QueueError::FileWrite(format!("{}: {}", self.path.display(), e)))?;
        Ok(msg)
    }

    /// 列出待投递消息（FIFO：按 created_at 再按 id）。`peer_id` 过滤指定对端。
    pub fn list(&self, peer_id: Option<&str>) -> Result<Vec<QueuedMessage>, QueueError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(QueueError::FileRead(format!(
                    "{}: {}",
                    self.path.display(),
                    e
                )))
            }
        };
        let mut out: Vec<QueuedMessage> = text
            .lines()
            // 损坏行跳过（崩溃中间态容忍），不阻塞其余条目
            .filter_map(|l| serde_json::from_str::<QueuedMessage>(l).ok())
            .filter(|m| peer_id.is_none_or(|p| m.peer_id == p))
            .collect();
        out.sort_by_key(|m| (m.created_at, m.id));
        Ok(out)
    }

    /// 解出某条目的明文（flush 投递前调用；AAD 绑定校验含归属方）。
    pub fn decrypt_payload(
        &self,
        local_key: &[u8; 32],
        entry: &QueuedMessage,
    ) -> Result<Vec<u8>, QueueError> {
        local_crypto::decrypt(local_key, &entry.peer_id, &entry.payload_hex)
            .map_err(|_| QueueError::DecryptFailed(entry.id))
    }

    /// 投递成功后移除。文件按剩余条目原子重写。
    pub fn remove(&self, id: u64) -> Result<(), QueueError> {
        let all = self.list(None)?;
        if !all.iter().any(|m| m.id == id) {
            return Err(QueueError::NotFound(id));
        }
        self.rewrite(all.into_iter().filter(|m| m.id != id))
    }

    /// 记录一次投递尝试（attempts+1）。
    pub fn record_attempt(&self, id: u64) -> Result<(), QueueError> {
        let all = self.list(None)?;
        if !all.iter().any(|m| m.id == id) {
            return Err(QueueError::NotFound(id));
        }
        self.rewrite(all.into_iter().map(|mut m| {
            if m.id == id {
                m.attempts += 1;
            }
            m
        }))
    }

    fn rewrite(&self, keep: impl Iterator<Item = QueuedMessage>) -> Result<(), QueueError> {
        let mut buf = String::new();
        for m in keep {
            let line =
                serde_json::to_string(&m).map_err(|e| QueueError::Serialize(e.to_string()))?;
            buf.push_str(&line);
            buf.push('\n');
        }
        let tmp = self.path.with_extension("jsonl.tmp");
        std::fs::write(&tmp, buf)
            .map_err(|e| QueueError::FileWrite(format!("{}: {}", tmp.display(), e)))?;
        std::fs::rename(&tmp, &self.path).map_err(|e| {
            QueueError::FileWrite(format!(
                "{} → {}: {}",
                tmp.display(),
                self.path.display(),
                e
            ))
        })?;
        Ok(())
    }
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
    use tempfile::tempdir;

    fn open_tmp_outbox(tag: &str) -> (tempfile::TempDir, Outbox) {
        let dir = tempdir().unwrap();
        let outbox = Outbox::open(&dir.path().join(tag)).unwrap();
        (dir, outbox)
    }

    #[test]
    fn enqueue_list_remove_roundtrip_fifo() {
        let key = [9u8; 32];
        let (_d, outbox) = open_tmp_outbox("rt");
        let m1 = outbox.enqueue(&key, "PEER_AAAAAAAA", b"c1").unwrap();
        let m2 = outbox.enqueue(&key, "PEER_BBBBBBBB", b"c2").unwrap();
        let m3 = outbox.enqueue(&key, "PEER_AAAAAAAA", b"c3").unwrap();
        assert_eq!((m1.id, m2.id, m3.id), (1, 2, 3));

        let all = outbox.list(None).unwrap();
        let ids: Vec<u64> = all.iter().map(|m| m.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
        let only_a = outbox.list(Some("PEER_AAAAAAAA")).unwrap();
        assert_eq!(only_a.len(), 2);
        // 解密回明文（AAD 绑定 peer_id）
        assert_eq!(outbox.decrypt_payload(&key, &only_a[0]).unwrap(), b"c1");
        assert_eq!(only_a[0].attempts, 0);

        outbox.remove(m2.id).unwrap();
        let all = outbox.list(None).unwrap();
        let ids: Vec<u64> = all.iter().map(|m| m.id).collect();
        assert_eq!(ids, vec![1, 3]);
        assert!(matches!(outbox.remove(m2.id), Err(QueueError::NotFound(2))));
    }

    #[test]
    fn plaintext_never_hits_disk() {
        // 验证清单「队列明文不入磁盘」：落盘的是本地密钥密文，原始文件字节不含明文
        let key = [11u8; 32];
        let secret = b"queue plaintext must never appear in outbox file";
        let (_d, outbox) = open_tmp_outbox("noplain");
        outbox.enqueue(&key, "PEER_CCCCCCCC", secret).unwrap();

        let raw = std::fs::read(&outbox.path).unwrap();
        let raw_str = String::from_utf8_lossy(&raw);
        assert!(
            !raw_str.contains("queue plaintext"),
            "outbox 文件不得含明文片段：\n{}",
            raw_str
        );
        // 且不是旧版会话密文形态——必须能被本地密钥解出
        let stored = outbox.list(None).unwrap()[0].clone();
        assert_eq!(outbox.decrypt_payload(&key, &stored).unwrap(), secret);
    }

    #[test]
    fn wrong_key_cannot_decrypt_entries() {
        let key = [12u8; 32];
        let (_d, outbox) = open_tmp_outbox("wrongkey");
        let m = outbox.enqueue(&key, "PEER_DDDDDDDD", b"secret").unwrap();
        let other_key = [13u8; 32];
        assert!(matches!(
            outbox.decrypt_payload(&other_key, &m),
            Err(QueueError::DecryptFailed(1))
        ));
    }

    #[test]
    fn corrupted_line_skipped_and_valid_entries_survive() {
        let key = [14u8; 32];
        let (_d, outbox) = open_tmp_outbox("corrupt");
        outbox.enqueue(&key, "PEER_EEEEEEEE", b"good-1").unwrap();
        {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&outbox.path)
                .unwrap();
            writeln!(f, "{{broken json").unwrap();
        }
        outbox.enqueue(&key, "PEER_EEEEEEEE", b"good-2").unwrap();

        let all = outbox.list(None).unwrap();
        assert_eq!(all.len(), 2, "损坏行应被跳过");

        outbox.remove(all[1].id).unwrap();
        let raw = std::fs::read_to_string(&outbox.path).unwrap();
        assert!(!raw.contains("broken"), "重写后损坏行应被清除");
    }

    #[test]
    fn record_attempt_increments_counter() {
        let key = [15u8; 32];
        let (_d, outbox) = open_tmp_outbox("attempts");
        let m = outbox.enqueue(&key, "PEER_FFFFFFFF", b"retry-me").unwrap();
        assert_eq!(m.attempts, 0);
        outbox.record_attempt(m.id).unwrap();
        outbox.record_attempt(m.id).unwrap();
        let listed = outbox.list(None).unwrap();
        assert_eq!(listed[0].attempts, 2);
    }

    #[test]
    fn empty_payload_rejected_and_missing_file_is_empty_list() {
        let key = [16u8; 32];
        let (_d, outbox) = open_tmp_outbox("edge");
        assert!(matches!(
            outbox.enqueue(&key, "PEER_GGGGGGGG", b""),
            Err(QueueError::EmptyPayload)
        ));
        assert!(outbox.list(None).unwrap().is_empty());
    }
}
