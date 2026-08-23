//! AI 对话会话的本地存储（桌面壳专属，不进 shared core）。
//!
//! 每个会话一个 JSON 文件，存放在 `~/.elwright/chats/<id>.json`。
//! 复用 `registry::user_root()` 定位用户层根目录（`ELWRIGHT_USER_ROOT`
//! 覆盖同生效，便于测试）。messages 只存 role/content，API Key 永不进入会话文件。

use std::path::PathBuf;

use elwright_core::core::llm::ChatMessage;
use elwright_core::core::registry;

/// 一个完整会话：元数据 + 消息列表。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<ChatMessage>,
}

/// 会话列表项：只含元字段，不读 messages（列表可能很长，避免全量加载）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
}

/// 会话存储根目录：`~/.elwright/chats/`。无主目录时返回 None。
fn chats_dir() -> Option<PathBuf> {
    registry::user_root().map(|r| r.join("chats"))
}

/// 列出全部会话摘要，按 updated_at 倒序（最近在前）。
/// 损坏的 JSON 文件跳过（不影响列表其余项）。
pub fn list_sessions() -> Vec<ChatSessionSummary> {
    let Some(dir) = chats_dir() else {
        return Vec::new();
    };
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut summaries: Vec<ChatSessionSummary> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                return None;
            }
            // 只读元字段：反序列化为 ChatSession 取前四字段，messages 为空也无妨
            let session: ChatSession =
                serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
            Some(ChatSessionSummary {
                id: session.id,
                title: session.title,
                updated_at: session.updated_at,
            })
        })
        .collect();
    // 字符串 ISO8601 可字典序比较（同格式前缀下与时间顺序一致）
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    summaries
}

/// 加载单个会话；不存在或损坏返回 None。
pub fn load_session(id: &str) -> Option<ChatSession> {
    let path = chats_dir()?.join(format!("{id}.json"));
    let text = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&text).ok()
}

/// 保存（upsert）：写入 `<id>.json`，更新 updated_at。
/// id/title/messages 来自调用方；created_at 首次写时设为 now，已存在则保留。
pub fn save_session(id: &str, title: &str, messages: &[ChatMessage]) -> Result<(), String> {
    let dir = chats_dir().ok_or("无法定位用户主目录（~/.elwright）")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 {}: {}", dir.display(), e))?;
    let path = dir.join(format!("{id}.json"));
    let now = now_iso();
    // 已存在则保留 created_at
    let created_at = load_session(id)
        .map(|s| s.created_at)
        .unwrap_or_else(|| now.clone());
    let session = ChatSession {
        id: id.to_string(),
        title: title.to_string(),
        created_at,
        updated_at: now,
        messages: messages.to_vec(),
    };
    let text =
        serde_json::to_string_pretty(&session).map_err(|e| format!("序列化会话失败: {}", e))?;
    std::fs::write(&path, text + "\n").map_err(|e| format!("写入 {}: {}", path.display(), e))?;
    Ok(())
}

/// 删除会话文件；不存在视为成功。
pub fn delete_session(id: &str) -> Result<(), String> {
    let Some(dir) = chats_dir() else {
        return Ok(());
    };
    let path = dir.join(format!("{id}.json"));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("删除 {}: {}", path.display(), e))?;
    }
    Ok(())
}

fn now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 简易 ISO8601（UTC）：排序只需定长字典序一致，不必引入 chrono
    let days = secs / 86400;
    let rem = secs % 86400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // 1970-01-01 起的天数转 YYYY-MM-DD（不处理格里高利闰年细节也够排序用）
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// 天数 → 年月日（1970 起，含闰年）。
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut y = 1970;
    loop {
        let dy = if is_leap(y) { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        y += 1;
    }
    let months = [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mo = 0;
    for (i, &mdays) in months.iter().enumerate() {
        let mdays = if i == 1 && is_leap(y) { 29 } else { mdays };
        if days < mdays {
            mo = i as u64 + 1;
            break;
        }
        days -= mdays;
    }
    (y, mo, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // user_root 读进程级环境变量，测试间串行
    static LOCK: Mutex<()> = Mutex::new(());

    fn temp_root(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("elwright-chat-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage::new(role, content)
    }

    #[test]
    fn save_load_list_delete_roundtrip() {
        let _g = LOCK.lock().unwrap();
        let root = temp_root("roundtrip");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);

        let id = "sess-1";
        save_session(
            id,
            "第一对话",
            &[msg("user", "你好"), msg("assistant", "嗨")],
        )
        .unwrap();

        let loaded = load_session(id).unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.title, "第一对话");
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.messages[0].role, "user");
        // 非空时间戳，且 created/updated 同为首次
        assert!(!loaded.created_at.is_empty());
        assert_eq!(loaded.created_at, loaded.updated_at);

        let list = list_sessions();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);

        // upsert：保留 created_at，更新 updated_at 与 messages
        std::thread::sleep(std::time::Duration::from_millis(1100));
        save_session(id, "改名了", &[msg("user", "二问")]).unwrap();
        let again = load_session(id).unwrap();
        assert_eq!(again.title, "改名了");
        assert_eq!(again.messages.len(), 1);
        assert_eq!(again.created_at, loaded.created_at, "created_at 应保留");
        assert_ne!(again.updated_at, loaded.updated_at, "updated_at 应推进");

        delete_session(id).unwrap();
        assert!(load_session(id).is_none());
        assert!(list_sessions().is_empty());

        std::env::remove_var("ELWRIGHT_USER_ROOT");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn list_sorted_by_updated_at_desc() {
        let _g = LOCK.lock().unwrap();
        let root = temp_root("sort");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);

        save_session("old", "旧", &[msg("user", "a")]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        save_session("new", "新", &[msg("user", "b")]).unwrap();

        let list = list_sessions();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "new", "最近更新的应在前");
        assert_eq!(list[1].id, "old");

        std::env::remove_var("ELWRIGHT_USER_ROOT");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn corrupted_file_skipped_in_list_and_load() {
        let _g = LOCK.lock().unwrap();
        let root = temp_root("corrupt");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);
        let dir = root.join("chats");
        std::fs::create_dir_all(&dir).unwrap();

        // 一个合法、一个损坏
        save_session("good", "好的", &[msg("user", "x")]).unwrap();
        std::fs::write(dir.join("bad.json"), "{ not json").unwrap();

        let list = list_sessions();
        assert_eq!(list.len(), 1, "损坏文件应跳过");
        assert_eq!(list[0].id, "good");
        assert!(load_session("bad").is_none(), "损坏文件 load 返回 None");
        assert!(load_session("missing").is_none(), "不存在返回 None");

        std::env::remove_var("ELWRIGHT_USER_ROOT");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn delete_missing_is_ok() {
        let _g = LOCK.lock().unwrap();
        let root = temp_root("delmissing");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);
        // 不存在也成功（幂等）
        delete_session("nope").unwrap();
        std::env::remove_var("ELWRIGHT_USER_ROOT");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn now_iso_is_sortable_lexicographically() {
        let _g = LOCK.lock().unwrap();
        let root = temp_root("iso");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);
        save_session("s1", "t", &[msg("user", "a")]).unwrap();
        let t1 = load_session("s1").unwrap().updated_at;
        std::thread::sleep(std::time::Duration::from_millis(1100));
        save_session("s1", "t", &[msg("user", "b")]).unwrap();
        let t2 = load_session("s1").unwrap().updated_at;
        assert!(t2 > t1, "字典序应与时间顺序一致: {t1} vs {t2}");
        assert!(
            t1.ends_with('Z') && t1.len() == 20,
            "格式应为 YYYY-MM-DDTHH:MM:SSZ"
        );
        std::env::remove_var("ELWRIGHT_USER_ROOT");
        std::fs::remove_dir_all(&root).ok();
    }
}
