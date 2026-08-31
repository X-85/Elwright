# verification：最近项目支持删除

日期：2026-08-31

## 【自动化】

- cargo test 全绿：核心 78（含 `remove_recent_project_drops_project_and_its_files_only`）
  + code_browser_ipc 2（含 `recent_project_remove_over_ipc`：open→remove→幂等）。
- cargo clippy --all-targets -D warnings 0 警告；cargo fmt 无差异。
- eslint exit 0；vitest 60/60；vite build 成功。

## 【手测】真机（tauri dev 自动重编 Rust 后新实例 + ZCode 桌面控制）

- 最近项目每行出现「删除最近项目 ×」按钮（a11y 可见）。
- 点 × springbootDemo1：行即时消失 + toast「已从最近项目移除（不影响磁盘上的项目文件）」
  在代码浏览器视图正常显示（顺带回归验证 Q22 toast 全视图化）。
- 持久化验证：`~/.elwright/code-browser.json` 中该项目及名下最近文件均已清除，仅剩 V1。
- 系统目录选择器 → 打开项目全链路首次真机验证通过（面板弹出、选择、openProject、树渲染）。
- 测试残留清理：早前失败测试运行在用户层留下的 3 书签 + 6 收藏（全部指向已删除的
  elwright-cb-* 临时目录）已清除；被删的 springbootDemo1 最近记录已恢复。

## 附带修复（本任务内）

- `tests/code_browser_ipc.rs` 加 `USER_STORE_LOCK` 互斥锁：两个用例并行执行时对真实
  用户层 load→改→save 会竞态丢更新（本地实测撞出）。
- 收藏/书签断言由绝对长度改增量式：老断言假设用户层为空，在非全新 HOME 的开发机上
  必然误报（CI 全新 HOME 从未暴露）。
