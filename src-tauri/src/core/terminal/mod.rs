//! 集成终端后端抽象与本地实现。
//!
//! 设计要点（见 ADR-001）：
//! - 前端只跟字节流打交道，不关心流来自 portable-pty 还是将来的 SSH 通道
//! - 输出按 ~16ms 合并后一次性推送（减少 IPC 调用次数）
//! - v1 只实现 LocalBackend；v1.x 增 SshBackend 时只需新增一个 trait impl
//!
//! SessionRegistry 持有所有活跃 session 的句柄（PTY master 与 killer），
//! 由 Tauri IPC 命令与 AppHandle Exit 钩子调用。

pub mod backend;
pub mod ipc;
pub mod local;

pub use backend::{SessionHandle, SessionId, SharedBackend, TerminalBackend};
pub use ipc::SessionRegistry;
pub use local::LocalBackend;
