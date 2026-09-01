//! Messaging offline queue（phase 2 step 5）。
//!
//! 存储格式：`<dir>/outbox.jsonl`，一行一条 JSON。**只存密文**
//! （hex 编码的 `messaging_transport::Transport::send` AEAD 帧输出）与
//! 路由元数据——明文不落盘，由单测强制。
//!
//! 用法：发送失败/对端离线时 `enqueue`；完整客户端连接成功后按 FIFO
//! `list` → 经中继重发 → `remove`。投递循环本身属完整客户端接线
//! （PeopleChatView 适配器替换，随前端接入落地），本模块只负责可靠的
//! 加密暂存。
//!
//! 与 plan 的偏差：原定 sled kv，实际零依赖 JSONL——理由见 ADR-002
//! 「实施偏差」段（密文已是自带完整性保护的 AEAD 帧，无需 kv 引擎；
//! 文件人类可检视；windows-gnu 工具链零负担）。

use std::io::Write;
use std::path::{Path, PathBuf};

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
    #[error("密文不能为空")]
    EmptyPayload,
    #[error("未找到队列条目：id={0}")]
    NotFound(u64),
}

/// 一条待投递消息。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct QueuedMessage {
    pub id: u64,
    /// 目标对端 ID（16 字符 base32）
    pub peer_id: String,
    /// 入队时间（unix 秒；FIFO 排序键）
    pub created_at: i64,
    /// AEAD 密文（`Transport::send` 输出）的 hex 编码
    pub payload_hex: String,
    /// 已尝试投递次数（失败重试时递增，供退避策略参考）
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

    /// 入队一条密文。返回完整条目。
    pub fn enqueue(&self, peer_id: &str, ciphertext: &[u8]) -> Result<QueuedMessage, QueueError> {
        if ciphertext.is_empty() {
            return Err(QueueError::EmptyPayload);
        }
        let existing = self.list(None)?;
        let id = existing.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        let msg = QueuedMessage {
            id,
            peer_id: peer_id.to_string(),
            created_at: current_unix_secs(),
            payload_hex: hex::encode(ciphertext),
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
    use crate::core::messaging_transport::complete_handshake;
    use tempfile::tempdir;

    fn open_tmp_outbox(tag: &str) -> (tempfile::TempDir, Outbox) {
        let dir = tempdir().unwrap();
        let outbox = Outbox::open(&dir.path().join(tag)).unwrap();
        (dir, outbox)
    }

    /// 产生一对真实 Transport（复用协议层测试辅助），返回 (alice, bob)。
    fn transports() -> (
        crate::core::messaging_transport::Transport,
        crate::core::messaging_transport::Transport,
    ) {
        complete_handshake([3u8; 32], [4u8; 32]).unwrap()
    }

    #[test]
    fn enqueue_list_remove_roundtrip_fifo() {
        let (_d, outbox) = open_tmp_outbox("rt");
        let m1 = outbox.enqueue("PEER_AAAAAAAA", b"c1").unwrap();
        let m2 = outbox.enqueue("PEER_BBBBBBBB", b"c2").unwrap();
        let m3 = outbox.enqueue("PEER_AAAAAAAA", b"c3").unwrap();
        assert_eq!((m1.id, m2.id, m3.id), (1, 2, 3));

        // 全量 FIFO
        let all = outbox.list(None).unwrap();
        let ids: Vec<u64> = all.iter().map(|m| m.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
        // 按对端过滤
        let only_a = outbox.list(Some("PEER_AAAAAAAA")).unwrap();
        assert_eq!(only_a.len(), 2);
        assert!(only_a.iter().all(|m| m.peer_id == "PEER_AAAAAAAA"));
        // payload hex 回读
        assert_eq!(hex::decode(&only_a[0].payload_hex).unwrap(), b"c1");

        // remove 后消失；未删的仍在
        outbox.remove(m2.id).unwrap();
        let all = outbox.list(None).unwrap();
        let ids: Vec<u64> = all.iter().map(|m| m.id).collect();
        assert_eq!(ids, vec![1, 3]);
        // 重复 remove → NotFound
        assert!(matches!(outbox.remove(m2.id), Err(QueueError::NotFound(2))));
    }

    #[test]
    fn plaintext_never_hits_disk() {
        // 验证清单「队列明文不入磁盘」：入队的是 AEAD 密文，原始文件字节不含明文
        let (mut alice, _bob) = transports();
        let secret = b"queue plaintext must never appear in outbox file";
        let ciphertext = alice.send(secret).unwrap();

        let (_d, outbox) = open_tmp_outbox("noplain");
        outbox.enqueue("PEER_CCCCCCCC", &ciphertext).unwrap();

        let raw = std::fs::read(&outbox.path).unwrap();
        let raw_str = String::from_utf8_lossy(&raw);
        assert!(
            !raw_str.contains("queue plaintext"),
            "outbox 文件不得含明文片段：\n{}",
            raw_str
        );
        // 密文 hex 在文件里可解回——但只能由持有 session key 的对端完成
        let stored = outbox.list(None).unwrap()[0].clone();
        assert_eq!(hex::decode(&stored.payload_hex).unwrap(), ciphertext);
    }

    #[test]
    fn corrupted_line_skipped_and_valid_entries_survive() {
        let (_d, outbox) = open_tmp_outbox("corrupt");
        outbox.enqueue("PEER_DDDDDDDD", b"good-1").unwrap();
        // 追加一行垃圾（模拟崩溃中间态）
        {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&outbox.path)
                .unwrap();
            writeln!(f, "{{broken json").unwrap();
        }
        outbox.enqueue("PEER_DDDDDDDD", b"good-2").unwrap();

        let all = outbox.list(None).unwrap();
        assert_eq!(all.len(), 2, "损坏行应被跳过");
        assert_eq!(hex::decode(&all[0].payload_hex).unwrap(), b"good-1");

        // remove 触发重写后，损坏行被自然清除
        outbox.remove(all[1].id).unwrap();
        let raw = std::fs::read_to_string(&outbox.path).unwrap();
        assert!(!raw.contains("broken"), "重写后损坏行应被清除");
    }

    #[test]
    fn record_attempt_increments_counter() {
        let (_d, outbox) = open_tmp_outbox("attempts");
        let m = outbox.enqueue("PEER_EEEEEEEE", b"retry-me").unwrap();
        assert_eq!(m.attempts, 0);
        outbox.record_attempt(m.id).unwrap();
        outbox.record_attempt(m.id).unwrap();
        let listed = outbox.list(None).unwrap();
        assert_eq!(listed[0].attempts, 2);
    }

    #[test]
    fn empty_payload_rejected_and_missing_file_is_empty_list() {
        let (_d, outbox) = open_tmp_outbox("edge");
        assert!(matches!(
            outbox.enqueue("PEER_FFFFFFFF", b""),
            Err(QueueError::EmptyPayload)
        ));
        // 文件尚不存在时 list 返回空而非报错
        assert!(outbox.list(None).unwrap().is_empty());
    }
}
