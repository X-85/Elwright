# 变更记录

## 2026-08-22

- 注册表精简：内置脚本能力从 3 个通用工具（doc-keyword-search / xlsx-to-md / docx-to-md，随 v0.1.1 进过包）缩减为 1 条示例 `text-stats`；个人脚本能力全面转向用户叠加层 `~/.elwright/` 导入。executor 执行链路不变。
- 本 Feature 文档随精简改写：README/behavior/architecture 面向当前真实状态（text-stats 示例 + 通用执行行为），旧 3 脚本的行为记录见 git 历史与 `docs/work/archive/`。

## 2026-08-21

- 首批 3 个通用脚本工具（doc-keyword-search / xlsx-to-md / docx-to-md）完成并进阶段 5；当时 executor 已支持 python3 探测回退 python/py。
- 更早：阶段 1 起即有基础执行链路（扩展名选解释器 + CLI `run`），脚本型一直是 CLI 与桌面壳共用的能力类型之一。
