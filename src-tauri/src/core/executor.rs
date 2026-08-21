use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptOutput {
    pub code: i32,
    pub output: String,
}

/// Run a script-type capability entry with the given arguments.
///
/// The interpreter is chosen by file extension:
/// `.py` -> python3, `.ps1` -> powershell, `.sh` -> bash, `.bat`/`.cmd` -> cmd.
/// Returns the process exit code on success. Designed to work fully offline
/// as long as the interpreter exists on the machine.
pub fn run_script(entry: &Path, args: &[String]) -> Result<i32, String> {
    let mut cmd = build_command(entry, args)?;
    let status = cmd
        .status()
        .map_err(|e| format!("启动 {} 失败: {}", entry.display(), e))?;
    Ok(status.code().unwrap_or(-1))
}

/// Run a script and retain stdout/stderr for a graphical caller.
pub fn run_script_capture(entry: &Path, args: &[String]) -> Result<ScriptOutput, String> {
    let mut cmd = build_command(entry, args)?;
    let output = cmd
        .output()
        .map_err(|e| format!("启动 {} 失败: {}", entry.display(), e))?;

    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&stderr);
    }

    Ok(ScriptOutput {
        code: output.status.code().unwrap_or(-1),
        output: text,
    })
}

/// Pick a Python interpreter once per process: `python3` first (unix 默认),
/// then `python` / `py`（Windows 官方启动器）。探测结果缓存，脚本运行不重复探测。
fn python_interpreter() -> &'static str {
    static PICKED: OnceLock<&'static str> = OnceLock::new();
    PICKED.get_or_init(|| {
        for candidate in ["python3", "python", "py"] {
            let ok = Command::new(candidate)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                return candidate;
            }
        }
        "python3"
    })
}

pub fn build_command(entry: &Path, args: &[String]) -> Result<Command, String> {
    let ext = entry
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let mut cmd = match ext.as_str() {
        "py" => {
            let mut c = Command::new(python_interpreter());
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
    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::{build_command, python_interpreter, run_script_capture};
    use std::fs;

    #[test]
    fn python_interpreter_picks_a_working_candidate() {
        // 本机至少装了一个 Python（CI runner 与开发机均满足）；探测应命中
        // 且 `--version` 可执行（证明缓存值是真实可用的解释器名）。
        let picked = python_interpreter();
        let ok = std::process::Command::new(picked)
            .arg("--version")
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "python_interpreter 选中的 {} 不可用", picked);
    }

    #[test]
    fn rejects_unsupported_extensions() {
        let error = build_command(std::path::Path::new("tool.txt"), &[]).unwrap_err();
        assert!(error.contains("不支持的脚本类型"));
    }

    #[cfg(unix)]
    #[test]
    fn captures_stdout_stderr_and_exit_code() {
        let path = std::env::temp_dir().join(format!(
            "elwright-executor-test-{}-{}.sh",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::write(
            &path,
            "printf 'stdout\\n'\nprintf 'stderr\\n' >&2\nexit 7\n",
        )
        .unwrap();

        let result = run_script_capture(&path, &[]).unwrap();
        fs::remove_file(&path).unwrap();

        assert_eq!(result.code, 7);
        assert!(result.output.contains("stdout"));
        assert!(result.output.contains("stderr"));
    }
}
