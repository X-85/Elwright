# Verification（验证记录）

> 方案见 [plan.md](./plan.md)；取舍见 [ADR-001](../../features/mindmap/decisions/ADR-001-mindmap-mvp.md)。

## 【自动化】（2026-09-01 完成）

- core::mindmap 5 单测：DFS 序保持（加兄弟/子）、子树级联删除（根拒绝）、整块交换
  （上移到首位后回弹）、缩进/外提（子树随行 + 一级外提拒绝）、持久化 roundtrip +
  损坏文件容忍 + id 消毒防路径穿越
- vitest 5：前端镜像纯函数同一套断言（DFS 序/级联删/块交换/缩进外提/折叠隐藏）
- e2e 1：脑图预览降级守卫（【预览模式】文案）；12/12 全过
- 门禁：cargo 135 lib + 全集成、clippy -D warnings 0、fmt、eslint 0、vitest 86、build

## 【手测】（真机点验待用户）

- 桌面端建图/编辑/折叠/转 Todo/导入 Todo 全链路
- 重启应用后脑图仍在（本机文件持久化）

## 已知边界

- 修复期抓错两个（测试自身断言）：深度计数预期写错（a1bx 实为 2）、第二张图
  node_count 非常数 3；均为测试笔误非实现缺陷
- 无撤销/重做（MVP 未含，按需后置）
