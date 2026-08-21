use std::path::Path;
use std::process::Command;

/// Run a script-type capability entry with the given arguments.
///
/// The interpreter is chosen by file extension:
/// `.py` -> python3, `.ps1` -> powershell, `.sh` -> bash, `.bat`/`.cmd` -> cmd.
/// Returns the process exit code on success. Designed to work fully offline
/// as long as the interpreter exists on the machine.
pub fn run_script(entry: &Path, args: &[String]) -> Result<i32, String> {
    let ext = entry
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let mut cmd = match ext.as_str() {
        "py" => {
            let mut c = Command::new("python3");
            c.arg(entry);
            c
        }
        "ps1" => {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
            c.arg(entry);
            c
        }
        "sh" => {
            let mut c = Command::new("bash");
            c.arg(entry);
            c
        }
        "bat" | "cmd" => {
            let mut c = Command::new("cmd");
            c.args(["/C"]);
            c.arg(entry);
            c
        }
        other => {
            return Err(format!(
                "不支持的脚本类型: .{}（仅支持 py/ps1/sh/bat/cmd）",
                other
            ))
        }
    };

    cmd.args(args);
    let status = cmd
        .status()
        .map_err(|e| format!("启动 {} 失败: {}", entry.display(), e))?;
    Ok(status.code().unwrap_or(-1))
}
