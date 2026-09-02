# 验证记录

## 自动化（2026-08-31）

- core 单测 10/10：收藏/书签切换对称性、按行去重、持久化回读、上限、
  save_recent 自建缺失目录（CI 首跑暴露：全新机器 ~/.elwright 不存在导致写入失败）。
- IPC 冒烟 1/1：favorites/bookmarks toggle 走真实协议（对称切换还原用户层）。
- vitest 31、e2e 10（无新增场景；发送到 AI 需桌面文件访问，浏览器端无路径）。
- 全量：cargo / clippy(-D warnings) / fmt / vitest / e2e / build 全绿。

## 自动化补充（第二批，2026-08-31）

- vitest 35（新增 codeLinks 4：标记解析/无标记原样/行号数字校验/marker 生成）。
- 全量：cargo 57+1+6+4+1 / clippy(-D warnings) / fmt / vitest 35 / e2e 10 / build 全绿。
- Todo 跳回与终端定位为纯前端接线（复用已测的 todo_add 与 terminal_open cwd），
  无新 IPC；真机点验见下。

## 人工验证

- 【待用户】真机点验：收藏与书签重开保留、AI 预填内容与选区一致。
- 【待用户】真机点验：转为 Todo 后在工作台点击位置标记跳回代码浏览器；终端定位开在文件目录。
