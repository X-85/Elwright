# change-note：后端 elwright.exe 弹可见控制台窗口（P1）

## 问题（真机点验 v0.1.13，Windows）

msi 安装启动桌面程序后，除了主窗口还弹出一个 `elwright.exe` 的
控制台式窗口；关掉它整个应用跟着退出。

## 根因

`src-tauri/src/main.rs` 缺少 Windows GUI 子系统声明。Rust 在 Windows
上默认把二进制链接为**控制台子系统**（CONSOLE），Windows 为其分配
可见控制台窗口；`elwright.exe`（桌面壳的后端进程）由此带窗。
「一关全退」本身是正常行为——后端进程退出则 WebView 壳失去支撑——
异常的只是后端进程不该有可见窗口。

修复采用 Tauri 标准写法：
`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
——release 构建 GUI 子系统，debug 构建保留控制台便于本地看日志。

## 修改范围

- `src-tauri/src/main.rs`：文件头部加 1 行内部属性 + 注释。无逻辑改动。
- `ew` CLI（`src-tauri/src/bin/ew.rs`）**刻意不加**该属性：CLI 就该有
  控制台（stdout 管道依赖 CONSOLE 子系统）。

## 风险

- 极低。属性只影响 PE 头 Subsystem 字段；唯一的已知副作用是 release
  桌面二进制不再有 stdout/stderr 控制台输出——桌面应用的日志本就不
  走 stdout，无影响。

## 验证

- 【自动化】PE 头探针（本机 GNU 工具链，解析可选头 Subsystem 字段）：
  - 修复后的 `target/release/elwright.exe` → **GUI**（本次 cargo
    release 实测构建）；
  - debug 构建 → CONSOLE（保留日志）；
  - 对照组：无属性二进制 CONSOLE（复现现状）、`-mwindows` 链接 GUI。
- 【自动化】Rust 门禁：fmt clean、clippy lib+tests 0 警告、ew 构建
  成功；前端四闸例行全绿（86/86、e2e 19/19，本项无前端改动）。
- 【手测】（留给用户，装新 msi 后）启动桌面程序 → 除主窗口外**不应
  再出现任何控制台窗口**；主窗口关闭整个应用退出（预期行为）。
