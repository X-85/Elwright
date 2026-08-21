use clap::{Parser, Subcommand};
use elwright_core::core::{executor, invoke, registry};

// 管道下游提前退出（如 `ew ls | grep -q ...`）会让 stdout 写入返回 EPIPE，
// println! 此时直接 panic。CLI 工具的标准行为是静默退出，故所有输出走容错宏。
macro_rules! outln {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}
macro_rules! errln {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stderr(), $($arg)*);
    }};
}

#[derive(Parser)]
#[command(name = "ew", about = "Elwright CLI · 个人工作流工具箱")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 列出所有能力
    Ls,
    /// 运行脚本型能力（离网可用）
    Run {
        id: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// 查看知识型/技能型文档
    View { id: String },
    /// 调用技能型（离线时降级为 SOP）
    Invoke {
        id: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        prompt: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let root = registry::find_project_root();
    let reg = match registry::Registry::load(&root) {
        Ok(r) => r,
        Err(e) => {
            errln!("错误: {}", e);
            std::process::exit(1);
        }
    };

    match cli.cmd {
        Cmd::Ls => {
            outln!("{:<26} {:<11} {:<9} {}", "ID", "TYPE", "OFFLINE", "NAME");
            outln!("{}", "-".repeat(72));
            for c in reg.list() {
                let offline = match c.offline {
                    Some(true) => "yes",
                    _ => "no",
                };
                outln!("{:<26} {:<11} {:<9} {}", c.id, c.kind, offline, c.name);
            }
            outln!("\n共 {} 项能力", reg.list().len());
        }
        Cmd::Run { id, args } => {
            let cap = match reg.get(&id) {
                Some(c) => c,
                None => {
                    errln!("未找到能力: {}", id);
                    std::process::exit(1);
                }
            };
            if cap.kind != "script" {
                errln!("{} 不是脚本型能力（type={}），无法 run", id, cap.kind);
                std::process::exit(1);
            }
            let entry = match reg.resolve_entry(cap) {
                Some(e) => e,
                None => {
                    errln!("能力 {} 缺少 entry 字段", id);
                    std::process::exit(1);
                }
            };
            if !entry.exists() {
                errln!("脚本不存在: {}", entry.display());
                std::process::exit(1);
            }
            outln!("▶ 运行 {} -> {}", id, entry.display());
            match executor::run_script(&entry, &args) {
                Ok(code) => outln!("退出码: {}", code),
                Err(e) => {
                    errln!("执行失败: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Cmd::View { id } => {
            let cap = match reg.get(&id) {
                Some(c) => c,
                None => {
                    errln!("未找到能力: {}", id);
                    std::process::exit(1);
                }
            };
            // 知识型用 doc 字段，脚本/文档型用 entry 字段
            let rel = cap.doc.as_ref().or(cap.entry.as_ref());
            match rel {
                Some(rel) => {
                    let p = reg.root.join(rel);
                    if p.exists() {
                        match std::fs::read_to_string(&p) {
                            Ok(s) => outln!("{}", s),
                            Err(e) => errln!("读取失败: {}", e),
                        }
                    } else {
                        errln!("文档不存在: {}", p.display());
                    }
                }
                None => outln!("能力 {} 无可查看文档", id),
            }
        }
        Cmd::Invoke { id, prompt } => {
            let cap = match reg.get(&id) {
                Some(c) => c,
                None => {
                    errln!("未找到能力: {}", id);
                    std::process::exit(1);
                }
            };
            if cap.kind != "skill" {
                errln!("{} 不是技能型能力（type={}），无法 invoke", id, cap.kind);
                std::process::exit(1);
            }
            let prompt_str = prompt.join(" ");
            outln!(
                "调用技能型: {}{}",
                id,
                if prompt_str.is_empty() {
                    String::new()
                } else {
                    format!(" (prompt: {})", prompt_str)
                }
            );

            let outcome = invoke::invoke_skill(&reg.root, cap, &prompt_str);
            if let Some(note) = outcome.note {
                errln!("{}\n", note);
            }
            if outcome.source == "degraded" {
                outln!("【离线降级】展示 SOP：\n{}", outcome.content);
            } else {
                outln!("\n{}", outcome.content);
            }
        }
    }
}
