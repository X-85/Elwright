# Topic 交接

- topic: T-2026-09-topic-ledger-mvp
- generated: 2026-09-11
- status: 处理中

## 一句话状态

Topic 台账独立验证包已落地，下一步是在公司 Windows 的 ZCode 安装并用 Chrome 查看真实 Topic。

## 已完成

- 新增 `topic-ledger` Skill，兼容原有 Q 编号台账。
- 新增 Topic 文件协议：`topic.md`、`events.md`、`decisions.md`、`handoff.md`。
- 新增 `install.ps1`，覆盖前备份旧 Skill。
- 新增零依赖 `viewer/topic-viewer.html`。
- `topic-ledger/` 可单独复制，不要求安装 Elwright、Rust 或 Node.js。
- 安装脚本显式使用 UTF-8，兼容 Windows PowerShell 5.1 的中文 Skill 文件。

## 已确认决定

- Topic 是人的工作单元；Chat/Task 是附属的交流和执行记录。
- 普通 Markdown 是跨 Agent 交接格式。
- 查看页只读，不上传、不执行命令。

## 证据与产出

- `topic-ledger/SKILL.md`
- `topic-ledger/references/protocol.md`
- `topic-ledger/install.ps1`
- `topic-ledger/viewer/topic-viewer.html`

## 未验证与风险

- 公司 ZCode 的 Skill 根目录需要现场确认；安装脚本支持 `-SkillRoot` 显式指定。
- Chrome 文件夹选择器需要选择单个 Topic 目录，而不是整个仓库。

## 下一步

1. 在公司 Windows 只复制 `topic-ledger` 目录（或拉取仓库后定位到该目录）。
2. 运行 `powershell -ExecutionPolicy Bypass -File .\\topic-ledger\\install.ps1 -SkillRoot "$env:USERPROFILE\\.zcode\\skills"`。
3. 在 ZCode 中让 Skill 创建或更新一个 Topic checkpoint，并完成一次 handoff。
4. 用 Google Chrome 打开 `topic-ledger\\viewer\\topic-viewer.html`，选择 `session\\topics\\T-2026-09-topic-ledger-mvp`。

## 给下一个 Agent 的边界

- 先读本文件、`topic.md` 和 `topic-ledger/references/protocol.md`。
- 不要把现有“资源与课题”页面重命名或迁移；它是另一个领域模型。
- 不要在未得到用户确认时关闭 Topic、确认重大决定或覆盖旧 Skill。
