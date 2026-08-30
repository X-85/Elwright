//! Elwright 桌面壳入口。IPC 命令实现全部在 `elwright_core::core::commands`
//! （见该文件头注释的下沉原因）；这里只做 Builder 装配：
//! 资源根解析、终端注册表、AppCtx 注入与窗口关闭清理。

use std::sync::Arc;

use elwright_core::core::commands::{self, AppCtx};
use elwright_core::core::{registry, terminal};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let resource_dir = app
                .path()
                .resource_dir()
                .map_err(|e| format!("定位资源目录失败: {}", e))?;
            let root = registry::resolve_root(&[resource_dir]);

            // 终端注册表（LocalBackend）：跨 IPC 调用保持
            // SessionRegistry::new 自带后台 reader 线程，返回 Arc<Self>
            let backend: terminal::SharedBackend = Arc::new(terminal::LocalBackend::new());
            let session_registry = terminal::SessionRegistry::new(backend);

            app.manage(AppCtx {
                root,
                terminal: session_registry,
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // 关闭主窗口时 kill 所有终端 session
                let app = window.app_handle();
                if let Some(ctx) = app.try_state::<AppCtx>() {
                    ctx.terminal.kill_all();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_capabilities,
            commands::view_doc,
            commands::run_script,
            commands::invoke_skill,
            commands::check_update,
            commands::import_capability,
            commands::export_capability,
            commands::delete_capability,
            commands::get_llm_config,
            commands::set_llm_config,
            commands::test_llm_connection,
            commands::chat_completion,
            commands::chat_list_sessions,
            commands::chat_load_session,
            commands::chat_save_session,
            commands::chat_delete_session,
            commands::terminal_open,
            commands::terminal_write,
            commands::terminal_resize,
            commands::terminal_close,
            commands::todo_list,
            commands::todo_add,
            commands::todo_toggle,
            commands::todo_remove,
            commands::note_get,
            commands::note_save,
            commands::note_list
        ])
        .run(tauri::generate_context!())
        .expect("启动 Elwright 桌面应用失败");
}
