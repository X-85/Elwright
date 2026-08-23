//! SessionRegistry：维护活跃终端会话并把 PTY 输出转发给 Tauri Channel。
//!
//! 数据流：
//! - 每个 session 一个独立 handle（来自 backend.spawn）
//! - 注册表起一个总轮询线程（spawn 在 new() 里）：每 ~16ms 拉一次所有 session 的输出
//! - 拉到的 bytes 通过该 session 关联的 Tauri Channel 推给前端
//! - 前端断开订阅时 Channel 自动丢消息（无害）
//!
//! 进程生命周期：
//! - `kill(id)` 关闭单个 session
//! - `kill_all()` 退出 App 时调用

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use tauri::ipc::Channel;

use super::backend::{SessionHandle, SessionId, SharedBackend};
// SessionHandle 是 TerminalHandle 的别名（见 backend.rs）

/// 批合并窗口：每 16ms flush 一次所有 session 的输出。
const FLUSH_INTERVAL: Duration = Duration::from_millis(16);

/// 每个 session 持有的状态。
struct SessionEntry {
    handle: Box<dyn SessionHandle>,
    channel: Channel<Vec<u8>>,
}

/// 全局 session 注册表。
///
/// 内部用 parking_lot::Mutex 保护 HashMap；reader thread 与 IPC 命令共用。
pub struct SessionRegistry {
    backend: SharedBackend,
    sessions: Mutex<HashMap<SessionId, SessionEntry>>,
    next_id: Mutex<u64>,
}

impl SessionRegistry {
    /// 创建注册表并启动后台 reader thread。
    pub fn new(backend: SharedBackend) -> Arc<Self> {
        let registry = Arc::new(Self {
            backend,
            sessions: Mutex::new(HashMap::new()),
            next_id: Mutex::new(0),
        });

        // 后台 reader：循环 poll 所有 session 输出并通过 Channel 推送
        let reg_for_thread = registry.clone();
        thread::Builder::new()
            .name("elwright-terminal-pump".into())
            .spawn(move || loop {
                {
                    let mut sessions = reg_for_thread.sessions.lock();
                    for entry in sessions.values_mut() {
                        while let Some(bytes) = entry.handle.take_output()() {
                            entry.channel.send(bytes).ok();
                        }
                    }
                }
                thread::sleep(FLUSH_INTERVAL);
            })
            .expect("启动终端 pump 线程失败");

        registry
    }

    /// 分配新 SessionId（单调递增）。
    fn alloc_id(&self) -> SessionId {
        let mut guard = self.next_id.lock();
        let id = SessionId(*guard);
        *guard += 1;
        id
    }

    /// 创建并注册新会话；channel 由 IPC 命令创建后传入。
    pub fn open(
        self: &Arc<Self>,
        shell: &str,
        cwd: &std::path::Path,
        cols: u16,
        rows: u16,
        env: &[(String, String)],
        channel: Channel<Vec<u8>>,
    ) -> Result<SessionId, String> {
        let handle = self.backend.spawn(shell, cwd, cols, rows, env)?;
        let id = self.alloc_id();
        self.sessions
            .lock()
            .insert(id, SessionEntry { handle, channel });
        Ok(id)
    }

    /// 写入用户输入到指定 session。
    pub fn write(&self, id: SessionId, bytes: &[u8]) -> Result<(), String> {
        let sessions = self.sessions.lock();
        let entry = sessions
            .get(&id)
            .ok_or_else(|| format!("会话 {} 不存在或已结束", id.0))?;
        entry.handle.write(bytes)
    }

    /// Resize 指定 session。
    pub fn resize(&self, id: SessionId, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock();
        let entry = sessions
            .get(&id)
            .ok_or_else(|| format!("会话 {} 不存在或已结束", id.0))?;
        entry.handle.resize(cols, rows)
    }

    /// 关闭指定 session 并回收子进程。
    pub fn kill(&self, id: SessionId) {
        if let Some(entry) = self.sessions.lock().remove(&id) {
            entry.handle.kill();
        }
    }

    /// 关闭所有 session（应用退出时调用）。
    pub fn kill_all(&self) {
        let mut sessions = self.sessions.lock();
        for (_, entry) in sessions.drain() {
            entry.handle.kill();
        }
    }

    /// 当前活跃 session 数量（供前端调试/状态显示）。
    pub fn active_count(&self) -> usize {
        self.sessions.lock().len()
    }

    /// 当前平台默认 shell 候选列表（首选排第一）。
    pub fn default_shells(&self) -> Vec<String> {
        self.backend.default_shell()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::terminal::backend::TerminalHandle;
    use crate::core::terminal::TerminalBackend;
    use std::path::Path;

    /// 内存 mock backend：每次 spawn 后立即塞入一段 banner 到共享 buffer。
    /// 完全不依赖真实 PTY/系统 shell，跨平台稳定。
    struct MockBackend {
        banner: Vec<u8>,
    }

    impl TerminalBackend for MockBackend {
        fn spawn(
            &self,
            _shell: &str,
            _cwd: &Path,
            _cols: u16,
            _rows: u16,
            _env: &[(String, String)],
        ) -> Result<Box<dyn TerminalHandle>, String> {
            let buf: Arc<parking_lot::Mutex<Vec<u8>>> =
                Arc::new(parking_lot::Mutex::new(self.banner.clone()));
            let input: Arc<parking_lot::Mutex<Vec<u8>>> =
                Arc::new(parking_lot::Mutex::new(Vec::new()));
            Ok(Box::new(MockHandle { buf, input }))
        }

        fn default_shell(&self) -> Vec<String> {
            vec!["mock".into()]
        }
    }

    struct MockHandle {
        buf: Arc<parking_lot::Mutex<Vec<u8>>>,
        input: Arc<parking_lot::Mutex<Vec<u8>>>,
    }

    impl TerminalHandle for MockHandle {
        fn take_output(&self) -> Box<dyn FnMut() -> Option<Vec<u8>> + Send> {
            let buf = self.buf.clone();
            Box::new(move || {
                let drained: Vec<u8> = buf.lock().drain(..).collect();
                if drained.is_empty() {
                    None
                } else {
                    Some(drained)
                }
            })
        }
        fn write(&self, bytes: &[u8]) -> Result<(), String> {
            self.input.lock().extend_from_slice(bytes);
            Ok(())
        }
        fn resize(&self, _cols: u16, _rows: u16) -> Result<(), String> {
            Ok(())
        }
        fn kill(&self) {
            self.buf.lock().clear();
            self.input.lock().clear();
        }
    }

    /// 多 session 并存：每个 session 独立 channel，不串扰。
    #[test]
    fn registry_keeps_sessions_independent() {
        let backend: SharedBackend = Arc::new(MockBackend {
            banner: b"BANNER\n".to_vec(),
        });
        let registry = SessionRegistry::new(backend);
        let channel_a = Channel::<Vec<u8>>::new(|_| Ok(()));
        let channel_b = Channel::<Vec<u8>>::new(|_| Ok(()));
        let id_a = registry
            .open(
                "mock",
                std::env::temp_dir().as_path(),
                80,
                24,
                &[],
                channel_a.clone(),
            )
            .unwrap();
        let id_b = registry
            .open(
                "mock",
                std::env::temp_dir().as_path(),
                80,
                24,
                &[],
                channel_b.clone(),
            )
            .unwrap();
        assert_ne!(id_a, id_b);
        assert_eq!(registry.active_count(), 2);

        registry.kill(id_a);
        assert_eq!(registry.active_count(), 1);

        registry.kill_all();
        assert_eq!(registry.active_count(), 0);
    }

    /// write / resize 对未知 id 返回友好错误。
    #[test]
    fn write_to_unknown_id_errors() {
        let backend: SharedBackend = Arc::new(MockBackend {
            banner: b"".to_vec(),
        });
        let registry = SessionRegistry::new(backend);
        let err = registry.write(SessionId(9999), b"hello").unwrap_err();
        assert!(err.contains("不存在"), "错误消息应包含「不存在」: {}", err);
    }
}
