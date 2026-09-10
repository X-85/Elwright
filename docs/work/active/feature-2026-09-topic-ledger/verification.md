# Verification

## 自动化

- `git diff --check`：通过。
- 手工结构检查：通过，Skill frontmatter、协议参考、安装脚本、查看页入口和示例 Topic 均存在。
- Skill Creator `quick_validate.py topic-ledger`：通过（`Skill is valid!`）。
- Playwright CLI（WebKit）：通过。打开查看页，选择当前 Topic 文件夹后，页面显示标题、状态、目标、完成标准、当前摘要、未决问题、产出物和交接摘要；展开详情后显示 handoff、decisions、events 三份内容。截图保存在本机 `.playwright-cli/` 临时目录，未纳入仓库。

## 手测

- Windows：从仓库根目录执行 `powershell -ExecutionPolicy Bypass -File .\topic-ledger\install.ps1`，确认旧目录被备份、新 Skill 可被 ZCode 发现。
- Windows：打开 `topic-ledger\viewer\topic-viewer.html`，选择 `session\topics\T-2026-09-topic-ledger-mvp`，确认状态、决策、未决项、产出物和交接区块可见。
