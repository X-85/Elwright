# `ew` 在管道下游提前退出时 panic

## 问题

`ew ls | grep -q xxx`、`ew ls | head` 等管道用法中，下游进程匹配/读取后提前退出，`ew` 继续写 stdout 收到 EPIPE，`println!` 直接 panic（`failed printing to stdout: Broken pipe`），`ew` 以 101 退出。

## 原因

CLI 程序的标准行为是管道下游退出后静默结束（cat/grep 等皆如此）；`println!` 遇写错误默认 panic，未做处理。

## 发现路径

CI run#2 macos 冒烟步骤失败 → 本地以 GitHub bash 等价的 `bash -eo pipefail` 复现 → 确认为产品 bug（影响所有用户管道用法），非 CI 环境问题。

## 修改范围

- `src-tauri/src/bin/ew.rs`：新增 `outln!` / `errln!` 容错输出宏（忽略写错误），替换文件内全部 `println!` / `eprintln!`。仅 CLI 壳；core 模块（IPC 侧返回 String，不直接打印）无需改动。

## 风险与影响

- 输出写失败（除 EPIPE 外理论上还有磁盘满等）从 panic 变为静默跳过——CLI 场景可接受，退出码仍由正常流程决定。
- CI 的三条冒烟断言在 `bash -eo pipefail` 下复测通过；`cargo test` 6/6 通过。
