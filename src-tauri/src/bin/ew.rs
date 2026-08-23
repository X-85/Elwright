use clap::{Parser, Subcommand};
use elwright_core::core::{executor, export, invoke, registry};

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
    /// 查看/设置 LLM 配置（环境变量 > config.local.json > ~/.elwright/config.json > 注册表默认）
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    /// 导出能力为单文件（默认打印，给文件名则写入）
    Export { id: String, file: Option<String> },
    /// 导入能力到用户叠加层 ~/.elwright/（.elw.json；id 冲突需 --force）
    Import {
        file: String,
        #[arg(long)]
        force: bool,
    },
    /// 删除自定义能力（仅用户叠加层条目，内置不可删）
    Delete { id: String },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// set <base_url|model|api_key> <值>；--local 写项目 config.local.json（默认写用户级）
    Set {
        key: String,
        value: String,
        #[arg(long)]
        local: bool,
    },
    /// 删除配置文件；--local 删项目的（默认删用户级）
    Clear {
        #[arg(long)]
        local: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let root = registry::resolve_root(&[]);
    let reg = match registry::Registry::load(&root) {
        Ok(r) => r,
        Err(e) => {
            errln!("错误: {}", e);
            std::process::exit(1);
        }
    };

    match cli.cmd {
        Cmd::Ls => {
            outln!(
                "{:<26} {:<11} {:<9} {:<6} {}",
                "ID",
                "TYPE",
                "OFFLINE",
                "SRC",
                "NAME"
            );
            outln!("{}", "-".repeat(78));
            for c in reg.list() {
                let offline = match c.offline {
                    Some(true) => "yes",
                    _ => "no",
                };
                let src = if reg.origin_of(&c.id) == registry::Origin::Custom {
                    "user"
                } else {
                    "-"
                };
                outln!(
                    "{:<26} {:<11} {:<9} {:<6} {}",
                    c.id,
                    c.kind,
                    offline,
                    src,
                    c.name
                );
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

            let outcome = invoke::invoke_skill(&reg, cap, &prompt_str);
            if let Some(note) = outcome.note {
                errln!("{}\n", note);
            }
            if outcome.source == "degraded" {
                outln!("【离线降级】展示 SOP：\n{}", outcome.content);
            } else {
                outln!("\n{}", outcome.content);
            }
        }
        Cmd::Config { action } => config_command(&reg, action),
        Cmd::Export { id, file } => {
            let bundle = match export::export_capability(&reg, &id) {
                Ok(b) => b,
                Err(e) => {
                    errln!("错误: {}", e);
                    std::process::exit(1);
                }
            };
            match file {
                Some(path) => match std::fs::write(&path, &bundle) {
                    Ok(_) => outln!("已导出 {} -> {}", id, path),
                    Err(e) => {
                        errln!("写入 {} 失败: {}", path, e);
                        std::process::exit(1);
                    }
                },
                None => outln!("{}", bundle),
            }
        }
        Cmd::Import { file, force } => {
            let text = match std::fs::read_to_string(&file) {
                Ok(t) => t,
                Err(e) => {
                    errln!("读取 {} 失败: {}", file, e);
                    std::process::exit(1);
                }
            };
            // 装机后 bundle 根只读/更新即清零，导入统一写用户叠加层
            let overlay = match registry::user_root() {
                Some(p) => p,
                None => {
                    errln!("错误: 无法定位用户主目录（HOME/USERPROFILE 均缺失）");
                    std::process::exit(1);
                }
            };
            match export::import_capability(&overlay, &text, force) {
                Ok(msg) => outln!("{}\n（叠加层: {}）", msg, overlay.display()),
                Err(e) => {
                    errln!("错误: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Cmd::Delete { id } => {
            let overlay = match registry::user_root() {
                Some(p) => p,
                None => {
                    errln!("错误: 无法定位用户主目录");
                    std::process::exit(1);
                }
            };
            match export::delete_capability(&overlay, &reg, &id) {
                Ok(msg) => outln!("{}", msg),
                Err(e) => {
                    errln!("错误: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn config_command(reg: &registry::Registry, action: Option<ConfigAction>) {
    use elwright_core::core::llm;
    match action {
        None => {
            let layers = llm::ConfigLayers::collect(&reg.root, reg.llm_default.clone());
            let (cfg, source) = layers.merged();
            let mask = |k: &str| {
                if k.is_empty() {
                    "（未设置）".to_string()
                } else if k.len() > 8 {
                    format!("{}****", &k[..4])
                } else {
                    "****".to_string()
                }
            };
            outln!("当前生效的 LLM 配置：");
            outln!(
                "  base_url : {}",
                if cfg.base_url.is_empty() {
                    "（未设置）".into()
                } else {
                    cfg.base_url.clone()
                }
            );
            outln!("    来源   : {}", source[0]);
            outln!(
                "  model    : {}",
                if cfg.model.is_empty() {
                    "（未设置）".into()
                } else {
                    cfg.model.clone()
                }
            );
            outln!("    来源   : {}", source[2]);
            outln!("  api_key  : {}", mask(&cfg.api_key));
            outln!("    来源   : {}", source[1]);
            outln!("\n设置：ew config set base_url <值>   清除：ew config clear");
        }
        Some(ConfigAction::Set { key, value, local }) => {
            if !matches!(key.as_str(), "base_url" | "model" | "api_key") {
                errln!("错误: key 只能是 base_url / model / api_key");
                std::process::exit(1);
            }
            let path = if local {
                reg.root.join("config.local.json")
            } else {
                match llm::user_config_path() {
                    Some(p) => p,
                    None => {
                        errln!("错误: 无法定位用户主目录（HOME/USERPROFILE 均缺失）");
                        std::process::exit(1);
                    }
                }
            };
            let mut cfg: std::collections::BTreeMap<String, String> =
                std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|t| serde_json::from_str(&t).ok())
                    .unwrap_or_default();
            cfg.insert(key.clone(), value);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let text = serde_json::to_string_pretty(&cfg).unwrap();
            match std::fs::write(&path, text + "\n") {
                Ok(_) => outln!("已设置 {} -> {}", key, path.display()),
                Err(e) => {
                    errln!("写入 {} 失败: {}", path.display(), e);
                    std::process::exit(1);
                }
            }
        }
        Some(ConfigAction::Clear { local }) => {
            let path = if local {
                reg.root.join("config.local.json")
            } else {
                match llm::user_config_path() {
                    Some(p) => p,
                    None => {
                        errln!("错误: 无法定位用户主目录");
                        std::process::exit(1);
                    }
                }
            };
            match std::fs::remove_file(&path) {
                Ok(_) => outln!("已删除 {}", path.display()),
                Err(_) => outln!("（无配置文件可删: {}）", path.display()),
            }
        }
    }
}
