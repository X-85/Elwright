# verification：后端 elwright.exe 弹可见控制台窗口（P1）

## 修复内容

`main.rs` 增加 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`：
release 二进制切 GUI 子系统（不分配控制台），debug 保留控制台。

## 验证记录

- 【自动化】PE 可选头 Subsystem 字段探针（解析 PE32+ 头 offset 68 的
  2 字节字段，本机 GNU 工具链，2026-09-03）：
  - 修复后 `target/release/elwright.exe`（cargo release 实测构建）→
    **GUI** ✓；
  - debug 构建 → CONSOLE（保留本地日志）✓；
  - 对照：无属性二进制 → CONSOLE（复现点验现象）✓；`-mwindows`
    链接 → GUI（原理验证）✓。
  - 探针脚本：`pe_sub.py`（parse 可选头），三构建物 + 两对照共 5 次断言
    全部符合预期。
- 【自动化】Rust 门禁：fmt clean；clippy --lib / --tests 各 0 警告；
  ew 构建成功。
- 【自动化】前端四闸例行全绿：lint 0 / vitest 86/86 / vite build 成功 /
  e2e 19/19（本项无前端改动，无回归）。
- 【手测】（留给用户，装新 msi 后）启动桌面程序 → 除主窗口外不应再
  出现 elwright.exe 控制台窗口；关闭主窗口整个应用退出（预期）；
  `ew ls` 在终端中仍正常输出（CLI 未受影响）。
