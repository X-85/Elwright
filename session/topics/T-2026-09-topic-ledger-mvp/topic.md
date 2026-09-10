# Topic: Topic 台账跨 Agent MVP

- id: T-2026-09-topic-ledger-mvp
- status: 处理中
- created: 2026-09-10
- updated: 2026-09-10
- owner: human

## 目标
把已有 session-ledger 升级为可在 ZCode、Codex 和其他 Agent 间携带的 Topic 工作单元，并提供一个人能快速查看的页面。

## 范围
- Topic 文件协议与台账 Skill 融合。
- ZCode Windows 可复制安装。
- 零依赖本地查看页。
- 保留现有 Q 编号和旧台账兼容性。

## 完成标准
- 明天在公司 Windows 拉取仓库后，可以安装 Skill 并在 ZCode 中维护一个 Topic。
- 选择 Topic 文件夹后，查看页能显示目标、当前状态、决定、未决项、产出物和交接摘要。

## 当前摘要
已确定 Topic 是人的工作单元，Chat 是交流容器，Task 是执行记录。MVP 采用普通 Markdown 文件协议，不绑定 Elwright 或某个 Agent。Skill、查看页、Windows 安装脚本和使用文档正在落地。

## 已验证
- [已验证] 现有 `session-ledger` 使用 `index.md`、`events.md`、`decisions.md` 和 `archive/`。
- [已验证] Elwright 现有“课题”页面是资源研究模型，不直接复用为工作 Topic。
- [未验证] 公司 Windows 的 ZCode Skill 目录实际路径和安装后自动发现行为。

## 未决问题
- [ ] 公司 ZCode 的实际 Skill 根目录是否为 `%USERPROFILE%\\.zcode\\skills`。
- [ ] 明天真机使用后，是否需要把默认 Skill 名从 `topic-ledger` 改成 `session-ledger`。

## 产出物
- `topic-ledger/SKILL.md`：跨 Agent Topic 台账 Skill。
- `topic-ledger/references/protocol.md`：字段、模板和兼容规则。
- `topic-ledger/viewer/topic-viewer.html`：本地只读查看页。
- `topic-ledger/install.ps1`：Windows 安装和旧 Skill 备份。

## 关联会话
- 当前 Codex 对话：Topic 台账 MVP 设计与实现。

## 关联问题
- Q40
