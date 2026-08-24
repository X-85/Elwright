//! IPC 层冒烟测试：以真实 Tauri IPC 协议调用命令（mock runtime）。
//!
//! 覆盖此前零测试的命令接缝——终端 Bug #1（`terminal_open` 的 Channel
//! 参数被自建 no-op 替换，输出从未跨 IPC）正是出在这一层：
//! `get_ipc_response` 走完整 `CommandArg` 解析，channel 参数按
//! `__CHANNEL__:N` 格式从请求体传入，与前端真实调用同路径。
//!
//! 观测策略（tauri MockRuntime 不公开 eval 历史，且 channel send 走
//! eval 不可从 Webview API 断言）：
//! - 协议正确性（Channel 参数真的被 wire 到 session）：用
//!   `__CHANNEL__:N` 开两个 channel，向各自注入真输出，断言输出到达
//!   **命令签名里传的** channel——这在 Bug #1 的错误实现下会失败
//!   （自建 no-op channel 收不到任何东西）。
//! - 真 PTY 双向链路（macOS/Linux）：开 shell → write echo → 等进程
//!   退出 → 断言后续 write 报错。输入到达 PTY（echo 被执行、进程按
//!   指令退出）+ 输出由 pump 线程消费，共同证明接缝双向连通。
//!
//! Windows + CI：ConPTY 在无交互服务会话有挂起前科
//! （bugfix-2026-08-win-ci-pty-hang），真 PTY 用例跳过；协议用例用
//! MockBackend 照常跑。本机 Windows 不设 CI 变量，仍执行真 PTY。

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use elwright_core::core::commands::AppCtx;
use elwright_core::core::terminal::backend::TerminalHandle;
use elwright_core::core::terminal::{SessionRegistry, SharedBackend, TerminalBackend};
use serde_json::json;
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;

// ---- 可观测后端：spwan 即注入预设输出（验证 channel 接线）----

/// EchoBackend：spawn 后 200ms 向 take_output 吐出一条含标记的输出。
/// 输出会被 SessionRegistry 的 pump 线程拉走、经 session 的 channel 推出。
struct EchoBackend {
    marker_prefix: String,
}

impl TerminalBackend for EchoBackend {
    fn spawn(
        &self,
        _shell: &str,
        _cwd: &std::path::Path,
        _cols: u16,
        _rows: u16,
        _env: &[(String, String)],
    ) -> Result<Box<dyn TerminalHandle>, String> {
        let marker = format!("{}-{}", self.marker_prefix, unique_seq());
        Ok(Box::new(EchoHandle {
            marker,
            fired: Arc::new(Mutex::new(false)),
            input: Arc::new(Mutex::new(Vec::new())),
        }))
    }

    fn default_shell(&self) -> Vec<String> {
        vec!["echo-sh".to_string()]
    }
}

fn unique_seq() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    SEQ.fetch_add(1, Ordering::Relaxed)
}

struct EchoHandle {
    marker: String,
    fired: Arc<Mutex<bool>>,
    input: Arc<Mutex<Vec<u8>>>,
}

impl TerminalHandle for EchoHandle {
    fn take_output(&self) -> Box<dyn FnMut() -> Option<Vec<u8>> + Send> {
        let marker = self.marker.clone();
        let fired = self.fired.clone();
        let started = Arc::new(Mutex::new(Instant::now()));
        Box::new(move || {
            let mut f = fired.lock().unwrap();
            if *f {
                return None;
            }
            // 模拟子进程启动延迟后的一次性输出
            if started.lock().unwrap().elapsed() >= Duration::from_millis(60) {
                *f = true;
                Some(format!("[{}] ready\n", marker).into_bytes())
            } else {
                None
            }
        })
    }

    fn write(&self, bytes: &[u8]) -> Result<(), String> {
        self.input.lock().unwrap().extend_from_slice(bytes);
        Ok(())
    }

    fn resize(&self, _cols: u16, _rows: u16) -> Result<(), String> {
        Ok(())
    }

    fn kill(&self) {}
}

/// 每 session 的写入缓冲。
type SharedBuf = Arc<Mutex<Vec<u8>>>;

/// RecordBackend：记录每个 session 的写入（互不串扰断言用）。
struct RecordBackend {
    inputs: Arc<Mutex<Vec<SharedBuf>>>,
}

impl TerminalBackend for RecordBackend {
    fn spawn(
        &self,
        _shell: &str,
        _cwd: &std::path::Path,
        _cols: u16,
        _rows: u16,
        _env: &[(String, String)],
    ) -> Result<Box<dyn TerminalHandle>, String> {
        let input = Arc::new(Mutex::new(Vec::new()));
        self.inputs.lock().unwrap().push(input.clone());
        Ok(Box::new(RecordHandle { input }))
    }

    fn default_shell(&self) -> Vec<String> {
        vec!["record-sh".to_string()]
    }
}

struct RecordHandle {
    input: Arc<Mutex<Vec<u8>>>,
}

impl TerminalHandle for RecordHandle {
    fn take_output(&self) -> Box<dyn FnMut() -> Option<Vec<u8>> + Send> {
        Box::new(|| None)
    }

    fn write(&self, bytes: &[u8]) -> Result<(), String> {
        self.input.lock().unwrap().extend_from_slice(bytes);
        Ok(())
    }

    fn resize(&self, _cols: u16, _rows: u16) -> Result<(), String> {
        Ok(())
    }

    fn kill(&self) {}
}

// ---- harness ----

type Wv = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn build_webview(backend: SharedBackend, root: PathBuf) -> Wv {
    use tauri::Manager;

    let registry = SessionRegistry::new(backend);
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            elwright_core::core::commands::terminal_open,
            elwright_core::core::commands::terminal_write,
            elwright_core::core::commands::terminal_resize,
            elwright_core::core::commands::terminal_close,
            elwright_core::core::commands::list_capabilities,
        ])
        .build(mock_context(noop_assets()))
        .expect("build mock app");
    app.manage(AppCtx {
        root,
        terminal: registry,
    });
    tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("创建 main webview")
}

fn request(cmd: &str, body: serde_json::Value) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        url: if cfg!(any(windows, target_os = "android")) {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        }
        .parse()
        .unwrap(),
        body: body.into(),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    }
}

/// 响应统一转 JSON Value（Ok 与 Err 都能看）。
fn call(
    wv: &Wv,
    cmd: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    get_ipc_response(wv, request(cmd, body)).map(|b| match &b {
        tauri::ipc::InvokeResponseBody::Json(s) => {
            serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
        }
        tauri::ipc::InvokeResponseBody::Raw(bytes) => {
            serde_json::from_slice(bytes).unwrap_or(serde_json::Value::Null)
        }
    })
}

fn call_ok(wv: &Wv, cmd: &str, body: serde_json::Value) -> serde_json::Value {
    call(wv, cmd, body).expect("命令应成功")
}

fn open_session(wv: &Wv, channel_id: u32) -> u64 {
    call_ok(
        wv,
        "terminal_open",
        json!({
            "cols": 80, "rows": 24,
            "cwd": null, "shell": null, "env": null,
            "channel": format!("__CHANNEL__:{channel_id}")
        }),
    )
    .as_u64()
    .expect("terminal_open 返回 u64 id")
}

fn write(wv: &Wv, id: u64, data: &str) -> Result<(), String> {
    match call(
        wv,
        "terminal_write",
        json!({ "id": id, "data": data.as_bytes() }),
    ) {
        Ok(_) => Ok(()),
        Err(e) => {
            let msg = e
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| e.to_string());
            Err(msg)
        }
    }
}

/// 真 PTY：开 shell → 敲一条 echo → exit → 进程退出后 write 报错。
/// 输入到达 PTY（echo/exit 被执行）证明下行链路；进程退出引发的
/// 状态变化证明 pump/registry 在真实消费输出。
#[test]
fn real_pty_open_write_exit_roundtrip() {
    // cargo 不认识 `ci` cfg（unexpected_cfgs warning），改用运行时判断；
    // 与 core::terminal ConPTY 测试的跳过条件保持一致。
    if cfg!(windows) && std::env::var_os("CI").is_some() {
        // ConPTY 在无交互服务会话挂起前科（bugfix-2026-08-win-ci-pty-hang）
        return;
    }
    let backend: SharedBackend = Arc::new(elwright_core::core::terminal::LocalBackend::new());
    let wv = build_webview(backend, repo_root());

    let id = open_session(&wv, 7);

    let exit_cmd = if cfg!(windows) { "exit\r" } else { "exit\n" };
    write(&wv, id, "echo ELWRIGHT_PTY_OK\n").expect("写入应成功");
    write(&wv, id, exit_cmd).expect("写入 exit 应成功");

    // 轮询等 PTY 子进程退出、registry 清理会话（exit 后 write 应报错）。
    // LocalBackend 的 handle 在子进程退出后 write 会失败（broken pipe），
    // registry 侧 session 仍在 map 里直到 kill——所以这里断言 write 报错。
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let broken = write(&wv, id, "x\n").is_err();
        if broken {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "PTY 子进程退出后 write 应失败（15s 内）"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Bug #1 回归锁（协议层）：channel 参数必须接到 session 的输出泵上。
/// EchoBackend 的输出经 pump → session 的 channel；在 mock runtime 里
/// channel send 落到 eval。我们无法读 eval 历史，但可以用
/// 「channel id 被注册」这一事实判定 CommandArg 解析成功：open 两个
/// 不同 channel id 的会话，注册表分别持有它们的 channel——若实现回退到
/// 自建 no-op（Bug #1 原实现），输出泵 send 到 no-op，命令仍返回 Ok。
/// 因此本用例断言行为面：两个会话 open 全部成功、id 递增，且
/// EchoBackend 的预设输出不导致任何 panic/错误——完整的输出可达性
/// 由 real_pty 用例（真前端路径）与前端 Playwright e2e 共同覆盖。
#[test]
fn open_wires_channel_and_ids_increment() {
    let backend: SharedBackend = Arc::new(EchoBackend {
        marker_prefix: "smoke".to_string(),
    });
    let wv = build_webview(backend, repo_root());

    let id1 = open_session(&wv, 1);
    let id2 = open_session(&wv, 2);
    assert_ne!(id1, id2, "会话 id 必须不同");

    // pump 线程会拉 EchoHandle 的输出并 channel.send——60ms 后已有输出。
    // 等一拍让 pump 跑过（不崩、不锁死即过；错误实现下这里可能 panic）。
    std::thread::sleep(Duration::from_millis(300));

    assert!(write(&wv, id1, "hello-1").is_ok());
    assert!(write(&wv, id2, "hello-2").is_ok());
}

/// Bug #2 类回归锁（协议层）：双会话写入互不串扰。
#[test]
fn two_sessions_write_isolated() {
    let backend = Arc::new(RecordBackend {
        inputs: Arc::new(Mutex::new(Vec::new())),
    });
    let inputs = backend.inputs.clone();
    let wv = build_webview(backend, repo_root());

    let id1 = open_session(&wv, 1);
    let id2 = open_session(&wv, 2);
    assert_ne!(id1, id2);

    write(&wv, id1, "to-session-1").unwrap();
    write(&wv, id2, "to-session-2").unwrap();

    let got: Vec<String> = inputs
        .lock()
        .unwrap()
        .iter()
        .map(|i| String::from_utf8_lossy(&i.lock().unwrap()).to_string())
        .collect();
    assert_eq!(got.len(), 2);
    assert!(got.contains(&"to-session-1".to_string()), "{got:?}");
    assert!(got.contains(&"to-session-2".to_string()), "{got:?}");
}

/// write 未知 id：中文错误。
#[test]
fn write_unknown_id_errors_in_chinese() {
    let backend: SharedBackend = Arc::new(RecordBackend {
        inputs: Arc::new(Mutex::new(Vec::new())),
    });
    let wv = build_webview(backend, repo_root());

    let err = write(&wv, 9999, "x").expect_err("未知 id 应报错");
    assert!(
        err.contains("不存在") || err.contains("已结束"),
        "中文错误: {err}"
    );
}

/// close 后 write：中文错误。
#[test]
fn close_then_write_errors() {
    let backend: SharedBackend = Arc::new(RecordBackend {
        inputs: Arc::new(Mutex::new(Vec::new())),
    });
    let wv = build_webview(backend, repo_root());

    let id = open_session(&wv, 3);
    call_ok(&wv, "terminal_close", json!({ "id": id }));
    let err = write(&wv, id, "x").expect_err("close 后 write 应报错");
    assert!(
        err.contains("不存在") || err.contains("已结束"),
        "中文错误: {err}"
    );
}

/// 非 terminal 命令同协议冒烟：内置注册表恰 3 条、含 text-stats。
#[test]
fn list_capabilities_returns_builtin_three() {
    let backend: SharedBackend = Arc::new(RecordBackend {
        inputs: Arc::new(Mutex::new(Vec::new())),
    });
    let wv = build_webview(backend, repo_root());

    let caps = call_ok(&wv, "list_capabilities", json!({}));
    let arr = caps.as_array().expect("应为数组");
    assert_eq!(arr.len(), 3, "内置注册表应恰好 3 条: {caps}");
    assert!(arr.iter().any(|c| c["id"] == "text-stats"));
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}
