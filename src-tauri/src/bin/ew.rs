use std::path::PathBuf;

use clap::{Parser, Subcommand};
use elwright_core::core::{degrade, executor, registry};

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
    View {
        id: String,
    },
    /// 调用技能型（离线时降级为 SOP）
    Invoke {
        id: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        prompt: Vec<String>,
    },
}

/// Walk up from the current directory looking for `capabilities.json`.
fn find_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("capabilities.json").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

fn main() {
    let cli = Cli::parse();
    let root = find_root();
    let reg = match registry::Registry::load(&root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    };

    match cli.cmd {
        Cmd::Ls => {
            println!("{:<26} {:<11} {:<9} {}", "ID", "TYPE", "OFFLINE", "NAME");
            println!("{}", "-".repeat(72));
            for c in reg.list() {
                let offline = match c.offline {
                    Some(true) => "yes",
                    _ => "no",
                };
                println!("{:<26} {:<11} {:<9} {}", c.id, c.kind, offline, c.name);
            }
            println!("\n共 {} 项能力", reg.list().len());
        }
        Cmd::Run { id, args } => {
            let cap = match reg.get(&id) {
                Some(c) => c,
                None => {
                    eprintln!("未找到能力: {}", id);
                    std::process::exit(1);
                }
            };
            if cap.kind != "script" {
                eprintln!("{} 不是脚本型能力（type={}），无法 run", id, cap.kind);
                std::process::exit(1);
            }
            let entry = match reg.resolve_entry(cap) {
                Some(e) => e,
                None => {
                    eprintln!("能力 {} 缺少 entry 字段", id);
                    std::process::exit(1);
                }
            };
            if !entry.exists() {
                eprintln!("脚本不存在: {}", entry.display());
                std::process::exit(1);
            }
            println!("▶ 运行 {} -> {}", id, entry.display());
            match executor::run_script(&entry, &args) {
                Ok(code) => println!("退出码: {}", code),
                Err(e) => {
                    eprintln!("执行失败: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Cmd::View { id } => {
            let cap = match reg.get(&id) {
                Some(c) => c,
                None => {
                    eprintln!("未找到能力: {}", id);
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
                            Ok(s) => println!("{}", s),
                            Err(e) => eprintln!("读取失败: {}", e),
                        }
                    } else {
                        eprintln!("文档不存在: {}", p.display());
                    }
                }
                None => println!("能力 {} 无可查看文档", id),
            }
        }
        Cmd::Invoke { id, prompt } => {
            let cap = match reg.get(&id) {
                Some(c) => c,
                None => {
                    eprintln!("未找到能力: {}", id);
                    std::process::exit(1);
                }
            };
            let prompt_str = prompt.join(" ");
            println!(
                "调用技能型: {}{}",
                id,
                if prompt_str.is_empty() {
                    String::new()
                } else {
                    format!(" (prompt: {})", prompt_str)
                }
            );
            // 阶段 2：若 LlmClient::from_env().is_some() 则调 LLM；否则降级 SOP
            let sop = degrade::show_sop(&reg.root, cap);
            println!("【离线降级】当前未配置 LLM，展示 SOP：\n{}", sop);
        }
    }
}
