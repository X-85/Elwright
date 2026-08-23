//! TerminalBackend trait —— 所有终端后端（local / ssh）共用同一接口。
//!
//! 约束：
//! - 读写与 resize 必须线程安全
//! - `kill` 幂等：第二次调用 no-op
//! - 输出字节流是原始 PTY bytes（不假定 UTF-8；TUI 程序会输出部分序列）

use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

/// 终端后端抽象。
///
/// 设计要点：
/// - `spawn` 返回 `Box<dyn TerminalHandle>`：句柄与生命周期绑在一起
/// - handle 持有 reader channel、writer、killer、resize 句柄
/// - 后端无状态可共享：`Arc<dyn TerminalBackend>` 在注册表里只存一份
pub trait TerminalBackend: Send + Sync {
    fn spawn(
        &self,
        shell: &str,
        cwd: &Path,
        cols: u16,
        rows: u16,
        env: &[(String, String)],
    ) -> Result<Box<dyn TerminalHandle>, String>;

    /// 列出可用于当前平台的默认 shell 候选（首选排第一）。
    fn default_shell(&self) -> Vec<String>;
}

/// 单个 session 的运行时句柄。
///
/// `take_output` 返回一个非阻塞闭包：每次调用尝试取一批 bytes（无则 None）。
/// 返回的闭包是 `FnMut + Send`，注册表在专用线程里反复 poll 即可。
pub trait TerminalHandle: Send {
    fn take_output(&self) -> Box<dyn FnMut() -> Option<Vec<u8>> + Send>;

    /// 写入用户输入到 PTY master（按键直接写 bytes）。
    fn write(&self, bytes: &[u8]) -> Result<(), String>;

    /// 通知子进程窗口尺寸变化（TUI 程序靠 TIOCSWINSZ 重绘）。
    fn resize(&self, cols: u16, rows: u16) -> Result<(), String>;

    /// 关闭 PTY、回收子进程。幂等。
    fn kill(&self);
}

pub type SharedBackend = Arc<dyn TerminalBackend>;

/// 别名：表达"per-session 句柄"，对外阅读更清晰。`TerminalHandle` 与
/// `SessionHandle` 指向同一 trait。
pub use TerminalHandle as SessionHandle;
