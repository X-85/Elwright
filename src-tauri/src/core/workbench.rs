//! 工作工具栏的本地存储：Todo 清单 + 今日记录（轻量记事本）。
//!
//! - Todo：单文件 `~/.elwright/todos.json`，整读整写；进程内 Mutex 串行化
//!   （单用户桌面场景足够，与 chats/ 的无跨进程锁口径一致）
//! - 今日记录：`~/.elwright/notes/YYYY-MM-DD.md` 一天一文件，纯文本由前端
//!   渲染 Markdown；日期参数严格校验（防路径穿越）
//! - 复用 `registry::user_root()` 定位用户层根目录（`ELWRIGHT_USER_ROOT`
//!   覆盖同生效，便于测试）

use std::path::PathBuf;

use super::registry;

/// 一条 Todo。JSON camelCase ↔ Rust snake_case（serde rename，与注册表同约定）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub id: u64,
    pub text: String,
    pub done: bool,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// todos.json 的顶层形态（对象不裸数组——与 capabilities.json 同教训）。
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TodoFile {
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    todos: Vec<TodoItem>,
}

/// 进程内串行化 todos.json 读写（读-改-写非原子，靠锁保证一致）。
static TODO_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn todos_path() -> Result<PathBuf, String> {
    let root = registry::user_root().ok_or("无法定位用户主目录（~/.elwright）")?;
    Ok(root.join("todos.json"))
}

fn load_todo_file() -> TodoFile {
    let Ok(path) = todos_path() else {
        return TodoFile::default();
    };
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn save_todo_file(file: &TodoFile) -> Result<(), String> {
    let path = todos_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建 {}: {}", parent.display(), e))?;
    }
    let text =
        serde_json::to_string_pretty(file).map_err(|e| format!("序列化 Todo 失败: {}", e))?;
    std::fs::write(&path, text + "\n").map_err(|e| format!("写入 {}: {}", path.display(), e))
}

/// 全量 Todo 列表（创建序）。
pub fn todo_list() -> Vec<TodoItem> {
    let _guard = TODO_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    load_todo_file().todos
}

/// 新增一条 Todo，返回完整条目。
pub fn todo_add(text: &str) -> Result<TodoItem, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Todo 内容不能为空".to_string());
    }
    if text.chars().count() > 500 {
        return Err("Todo 内容过长（上限 500 字符）".to_string());
    }
    let _guard = TODO_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut file = load_todo_file();
    let item = TodoItem {
        id: file.next_id,
        text: text.to_string(),
        done: false,
        created_at: now_iso(),
        completed_at: None,
    };
    file.next_id += 1;
    file.todos.push(item.clone());
    save_todo_file(&file)?;
    Ok(item)
}

/// 勾选/取消勾选：done 取反，completedAt 随之设置/清空。返回更新后的条目。
pub fn todo_toggle(id: u64) -> Result<TodoItem, String> {
    let _guard = TODO_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut file = load_todo_file();
    let item = file
        .todos
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| format!("Todo {} 不存在或已删除", id))?;
    item.done = !item.done;
    item.completed_at = if item.done { Some(now_iso()) } else { None };
    let updated = item.clone();
    save_todo_file(&file)?;
    Ok(updated)
}

/// 删除一条 Todo；不存在时报错（前端据 id 删除，静默成功会掩盖状态漂移）。
pub fn todo_remove(id: u64) -> Result<(), String> {
    let _guard = TODO_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut file = load_todo_file();
    let before = file.todos.len();
    file.todos.retain(|t| t.id != id);
    if file.todos.len() == before {
        return Err(format!("Todo {} 不存在或已删除", id));
    }
    save_todo_file(&file)
}

// ---- 今日记录 ----

fn notes_dir() -> Result<PathBuf, String> {
    let root = registry::user_root().ok_or("无法定位用户主目录（~/.elwright）")?;
    Ok(root.join("notes"))
}

/// 日期参数校验：`YYYY-MM-DD` 且数值合法（防 `../evil` 类路径穿越）。
fn validate_date(date: &str) -> Result<(), String> {
    let bytes = date.as_bytes();
    let shape_ok = bytes.len() == 10
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit);
    if !shape_ok {
        return Err(format!("日期格式无效（应为 YYYY-MM-DD）: {}", date));
    }
    let (y, m, d) = (
        date[0..4].parse::<u32>().unwrap(),
        date[5..7].parse::<u32>().unwrap(),
        date[8..10].parse::<u32>().unwrap(),
    );
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || y < 1970 || y > 9999 {
        return Err(format!("日期数值无效: {}", date));
    }
    Ok(())
}

fn note_path(date: &str) -> Result<PathBuf, String> {
    validate_date(date)?;
    Ok(notes_dir()?.join(format!("{}.md", date)))
}

/// 读某日记录；无记录返回 None（前端显示空编辑器，不视为错误）。
pub fn note_get(date: &str) -> Result<Option<String>, String> {
    let path = note_path(date)?;
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(Some(text)),
        Err(_) => Ok(None),
    }
}

/// 保存某日记录（整文件覆盖）。内容为空时写入空文件（保留该日"存在"事实）。
pub fn note_save(date: &str, content: &str) -> Result<(), String> {
    let path = note_path(date)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建 {}: {}", parent.display(), e))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("写入 {}: {}", path.display(), e))
}

/// 已有记录的日期列表，倒序（最近在前）。
pub fn note_list_dates() -> Vec<String> {
    let Ok(dir) = notes_dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut dates: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let stem = name.strip_suffix(".md")?;
            validate_date(stem).ok()?;
            Some(stem.to_string())
        })
        .collect();
    dates.sort_unstable();
    dates.reverse();
    dates
}

/// 简易 ISO8601（UTC）：与 chat_store 排序口径一致，不引入 chrono。
fn now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let rem = secs % 86400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

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
    let mut mo = 1;
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
    use crate::core::test_env::env_serialization_guard;

    fn temp_root(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("elwright-workbench-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn todo_add_list_toggle_remove_roundtrip() {
        let _guard = env_serialization_guard();
        let root = temp_root("todo-roundtrip");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);

        let a = todo_add("写周报").unwrap();
        let b = todo_add("复查 PR").unwrap();
        assert_ne!(a.id, b.id, "id 应单调递增");
        assert!(!a.done && a.completed_at.is_none());

        let toggled = todo_toggle(b.id).unwrap();
        assert!(toggled.done && toggled.completed_at.is_some());

        let todos = todo_list();
        assert_eq!(todos.len(), 2);
        assert!(todos.iter().any(|t| t.id == b.id && t.done));

        // 取消勾选清空 completedAt
        let back = todo_toggle(b.id).unwrap();
        assert!(!back.done && back.completed_at.is_none());

        todo_remove(a.id).unwrap();
        assert_eq!(todo_list().len(), 1);

        let err = todo_remove(a.id).unwrap_err();
        assert!(err.contains("不存在"), "中文错误: {err}");

        // 落盘形态：camelCase（注册表同约定）
        let raw = std::fs::read_to_string(root.join("todos.json")).unwrap();
        assert!(raw.contains("\"nextId\""), "camelCase 落盘: {raw}");
        assert!(raw.contains("\"createdAt\""));

        std::env::remove_var("ELWRIGHT_USER_ROOT");
    }

    #[test]
    fn todo_add_rejects_blank_and_overlong() {
        let _guard = env_serialization_guard();
        let root = temp_root("todo-validate");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);

        assert!(todo_add("   ").unwrap_err().contains("不能为空"));
        let long = "字".repeat(501);
        assert!(todo_add(&long).unwrap_err().contains("过长"));

        std::env::remove_var("ELWRIGHT_USER_ROOT");
    }

    #[test]
    fn note_save_get_list_roundtrip() {
        let _guard = env_serialization_guard();
        let root = temp_root("note-roundtrip");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);

        assert_eq!(note_get("2026-08-25").unwrap(), None, "无记录返回 None");
        note_save("2026-08-25", "# 今日\n- 完成工作台立项").unwrap();
        note_save("2026-08-24", "昨天").unwrap();
        assert_eq!(
            note_get("2026-08-25").unwrap().unwrap(),
            "# 今日\n- 完成工作台立项"
        );

        let dates = note_list_dates();
        assert_eq!(dates, vec!["2026-08-25", "2026-08-24"], "倒序");

        let note = root.join("notes/2026-08-25.md");
        assert!(note.exists(), "落盘于 notes/YYYY-MM-DD.md");

        std::env::remove_var("ELWRIGHT_USER_ROOT");
    }

    #[test]
    fn note_rejects_path_traversal_and_bad_dates() {
        let _guard = env_serialization_guard();
        let root = temp_root("note-validate");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);

        for bad in [
            "../evil",
            "2026-8-5",
            "",
            "2026/08/25",
            "2026-13-01",
            "2026-08-32",
        ] {
            let err = note_save(bad, "x").unwrap_err();
            assert!(err.contains("日期"), "应拒绝 {}: {err}", bad);
        }
        // 合法边界：闰日、年末
        note_save("2024-02-29", "ok").unwrap();
        note_save("2026-12-31", "ok").unwrap();

        // 非法日期文件不进列表
        std::fs::create_dir_all(root.join("notes")).unwrap();
        std::fs::write(root.join("notes/../../escape.md"), "x").ok();
        std::fs::write(root.join("notes/not-a-date.md"), "x").unwrap();
        assert!(note_list_dates().iter().all(|d| d.len() == 10));

        std::env::remove_var("ELWRIGHT_USER_ROOT");
    }

    #[test]
    fn corrupted_todo_file_degrades_to_empty() {
        let _guard = env_serialization_guard();
        let root = temp_root("todo-corrupt");
        std::env::set_var("ELWRIGHT_USER_ROOT", &root);
        std::fs::write(root.join("todos.json"), "{broken json").unwrap();

        assert!(todo_list().is_empty(), "损坏文件降级为空列表不 panic");
        let item = todo_add("重新开始").unwrap();
        assert_eq!(item.id, 0);

        std::env::remove_var("ELWRIGHT_USER_ROOT");
    }
}
