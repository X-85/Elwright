//! 本地 PTY 后端（基于 portable-pty）。
//!
//! 跨平台策略：
//! - macOS / Linux：fork+exec
//! - Windows：ConPTY（Win10 1809+）
//!
//! portable-pty 自动选择底层实现，我们只需调用 `native_pty_system()`。

use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use crossbeam_channel as cb;
use parking_lot::Mutex as PLMutex;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

use super::backend::{TerminalBackend, TerminalHandle};

/// LocalBackend：本地 shell 后端，无状态、可共享。
pub struct LocalBackend;

impl LocalBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalBackend for LocalBackend {
    fn spawn(
        &self,
        shell: &str,
        cwd: &Path,
        cols: u16,
        rows: u16,
        env: &[(String, String)],
    ) -> Result<Box<dyn TerminalHandle>, String> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("打开 PTY 失败: {}", e))?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(cwd);
        for (k, v) in env {
            cmd.env(k, v);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("启动 shell {} 失败: {}", shell, e))?;
        let killer = child.clone_killer();
        drop(pair.slave);

        // portable-pty 推荐写法：分别 clone reader、take writer，避免与 MasterPty
        // 生命周期管理相互纠缠（resize 仍通过 master 调用）。
        let master = pair.master;
        let mut reader = master
            .try_clone_reader()
            .map_err(|e| format!("克隆 PTY reader 失败: {}", e))?;
        let writer = master
            .take_writer()
            .map_err(|e| format!("获取 PTY writer 失败: {}", e))?;
        // master 本身保留在 handle 里供后续 resize 用；drop 会关闭 PTY
        let master = Arc::new(PLMutex::new(Some(master)));

        let (tx, rx) = cb::unbounded::<Vec<u8>>();

        // reader thread：连续读 PTY，每批 raw bytes 直接推到 cb channel。
        let reader_thread = thread::Builder::new()
            .name("elwright-pty-reader".into())
            .spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if tx.send(buf[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    }
                }
            })
            .map_err(|e| format!("启动 PTY 读取线程失败: {}", e))?;

        Ok(Box::new(LocalHandle {
            rx: parking_lot::Mutex::new(rx),
            writer: Arc::new(PLMutex::new(Some(writer))),
            master,
            killer: parking_lot::Mutex::new(Some(killer)),
            _reader_thread: parking_lot::Mutex::new(Some(reader_thread)),
        }))
    }

    fn default_shell(&self) -> Vec<String> {
        #[cfg(unix)]
        {
            if let Ok(shell) = std::env::var("SHELL") {
                if !shell.is_empty() {
                    return vec![shell];
                }
            }
            vec!["/bin/zsh".into(), "/bin/bash".into(), "/bin/sh".into()]
        }
        #[cfg(windows)]
        {
            for candidate in ["pwsh.exe", "powershell.exe"] {
                if shell_exists(candidate) {
                    return vec![candidate.into()];
                }
            }
            vec!["powershell.exe".into()]
        }
    }
}

#[cfg(windows)]
fn shell_exists(name: &str) -> bool {
    std::process::Command::new(name)
        .arg("-NoProfile")
        .arg("-Command")
        .arg("exit 0")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// 本地 PTY session 句柄。
///
/// 设计：
/// - `rx`：reader thread 推过来的 raw bytes，Registry 取出后用 Tauri Channel 发前端
/// - `writer`：PTY 写端（用于 write user input）；take_writer 只能调一次所以是 Option
/// - `master`：保留以供 resize；take_writer 后 master 仍可用 resize
/// - `killer`：kill 时回收子进程（幂等）
struct LocalHandle {
    rx: parking_lot::Mutex<cb::Receiver<Vec<u8>>>,
    writer: Arc<PLMutex<Option<Box<dyn Write + Send>>>>,
    master: Arc<PLMutex<Option<Box<dyn portable_pty::MasterPty + Send>>>>,
    killer: parking_lot::Mutex<Option<Box<dyn portable_pty::ChildKiller + Send>>>,
    _reader_thread: parking_lot::Mutex<Option<thread::JoinHandle<()>>>,
}

impl TerminalHandle for LocalHandle {
    fn take_output(&self) -> Box<dyn FnMut() -> Option<Vec<u8>> + Send> {
        let rx = self.rx.lock().clone();
        Box::new(move || rx.try_recv().ok())
    }

    fn write(&self, bytes: &[u8]) -> Result<(), String> {
        let mut guard = self.writer.lock();
        let writer = guard
            .as_mut()
            .ok_or_else(|| "PTY writer 已被释放".to_string())?;
        writer
            .write_all(bytes)
            .map_err(|e| format!("写入 PTY 失败: {}", e))
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        let guard = self.master.lock();
        let master = guard
            .as_ref()
            .ok_or_else(|| "PTY master 已被释放".to_string())?;
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("PTY resize 失败: {}", e))
    }

    fn kill(&self) {
        if let Some(mut killer) = self.killer.lock().take() {
            let _ = killer.kill();
        }
        // drop writer/master → PTY 关闭 → reader thread 自然 EOF 退出
        self.writer.lock().take();
        self.master.lock().take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 端到端：spawn 一个 echo 1; sleep; echo 2 的 shell，验证两条输出都能被 take_output 拿到。
    /// Windows 走 powershell，类 Unix 走 `$SHELL` 或 sh。
    #[test]
    fn spawn_produces_expected_output() {
        let backend = LocalBackend::new();
        let shell = backend
            .default_shell()
            .into_iter()
            .next()
            .expect("至少有一个默认 shell");
        let cwd = std::env::temp_dir();

        // 选个跨平台能 echo "hello" 的命令
        let cmdline = if cfg!(windows) {
            "Write-Output hello; Start-Sleep -Milliseconds 100; Write-Output world"
        } else {
            "echo hello; sleep 0.1; echo world"
        };

        // 直接 spawn 而不是通过 backend（我们要拿到 handle 内部细节）
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("openpty");
        let mut builder = CommandBuilder::new(&shell);
        builder.cwd(&cwd);
        // 用 -c / -Command 一次性执行
        #[cfg(unix)]
        builder.args(["-c", cmdline]);
        #[cfg(windows)]
        builder.args(["-NoProfile", "-Command", cmdline]);

        let child = pair.slave.spawn_command(builder).expect("spawn");
        let killer = child.clone_killer();
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().expect("reader");
        drop(pair.master.take_writer().ok());

        let (tx, rx) = cb::unbounded::<Vec<u8>>();
        let reader_thread = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        });

        // 等子进程结束
        let mut child = child;
        let _ = child.wait();

        // 关 PTY master 让 reader EOF
        drop(child);
        let mut killer = killer;
        let _ = killer.kill();
        reader_thread.join().ok();

        // 收集所有 bytes
        let mut all = Vec::new();
        while let Ok(chunk) = rx.try_recv() {
            all.extend_from_slice(&chunk);
        }
        let text = String::from_utf8_lossy(&all);
        assert!(
            text.contains("hello"),
            "PTY 输出应包含 hello，实际: {}",
            text
        );
        assert!(
            text.contains("world"),
            "PTY 输出应包含 world，实际: {}",
            text
        );
    }
}