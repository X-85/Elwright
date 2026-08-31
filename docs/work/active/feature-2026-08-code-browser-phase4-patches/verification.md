# 验证记录

> 模板——实施后回填。

## 自动化（待回填）

- patch.rs 单测：parse / apply / sensitive / snapshot round-trip 全过（X 例）
- IPC mock runtime：apply_patch_preview / apply / revert 三例
- 前端 vitest：patch.ts 解析 + ChatView diff 识别全过
- 本地五道闸：cargo / strict clippy / fmt / vitest / build 全绿
- CI 7/7 全绿

## 人工验证（待真机点验）

- 【待用户】选中代码片段 → AI 回复含 diff → 点击预览 → 三栏对照 → 逐 hunk 接受/拒绝 → 应用 → 文件落盘正确 → 点击撤销 → 文件恢复原状
- 【待用户】敏感文件（.env / .ssh / node_modules / target）应用 → 拒收并提示
- 【待用户】写入冲突（先用外部编辑器改同文件）→ 拒绝应用并提示先刷新
- 【待用户】跨平台路径（Windows 反斜杠 diff）→ 写入正常