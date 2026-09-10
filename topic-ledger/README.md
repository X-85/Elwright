# Topic Ledger MVP

这是一个可单独复制到 ZCode、Codex 或其他支持 Markdown Skill 的 Agent 中使用的 Topic 台账验证包。`topic-ledger` 目录本身就是分发边界，不依赖 Elwright、Rust、Node.js、在线 LLM 或本地服务器；只需要 Agent 能读取和编辑 Markdown，以及 Windows 上有 PowerShell。

## Windows 安装

可以从 Git 仓库复制本目录，也可以把本目录压缩后带到公司电脑。假设它位于 `C:\Tools\topic-ledger`：

```powershell
Set-Location C:\Tools\topic-ledger
powershell -ExecutionPolicy Bypass -File .\install.ps1 -SkillRoot "$env:USERPROFILE\.zcode\skills"
```

如果 ZCode 的 Skill 目录不是这个路径，把 `-SkillRoot` 改成 ZCode 实际使用的目录。省略 `-SkillRoot` 时，脚本会依次尝试 `.zcode\skills`、`.codex\skills` 和 `.config\zcode\skills` 中已存在的目录。

默认安装名为 `topic-ledger`，不会覆盖原来的 `session-ledger`。如果要在原命令名下试用，显式传入 `-SkillName session-ledger`；覆盖前脚本会先生成带时间戳的 `.backup-*` 目录。

安装后重新打开 ZCode，在一个具体工作目录中让它维护 Topic，例如：“为当前工作创建或更新 Topic checkpoint”。Skill 会把文件写入该工作目录的 `session/topics/<topic-id>/`，不需要 Elwright 参与。

## Chrome 查看

双击 `viewer\topic-viewer.html`，或用 Google Chrome 打开它，点击“打开 Topic 文件夹”，选择项目中的 `session\topics\T-...` 目录。Chrome 支持文件夹选择器；也可以把单个 Topic 文件夹拖到页面左侧。查看页只在浏览器内读取 Markdown，不写回文件、不上传、不执行命令。

如果浏览器不支持文件夹选择器，可以改用“选择 Markdown 文件”，一次选择同一 Topic 的四个 Markdown 文件。

## 包内容

- `SKILL.md`：Agent 的 Topic 维护规则。
- `references/protocol.md`：文件字段、模板和旧台账兼容规则。
- `viewer/topic-viewer.html`：零依赖的人类只读查看页。
- `install.ps1`：Windows 安装脚本。
