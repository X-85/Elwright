# Topic Ledger MVP

这是一个可单独交给 ZCode、Codex 或其他支持 Markdown Skill 的 Agent 使用的 Topic 台账源 Skill。`topic-ledger` 目录本身就是分发边界，不依赖 Elwright、Rust、Node.js、在线 LLM 或本地服务器。

## 使用源 Skill

明天只需要把 `topic-ledger` 目录带到公司电脑，然后让 ZCode 安装或加载这个源 Skill：

```text
请安装并启用这个源 Skill：topic-ledger/SKILL.md
同时保留并按需读取 topic-ledger/references/protocol.md。
```

安装后重新打开或刷新 ZCode，在具体工作目录中直接说“为当前工作创建或更新 Topic checkpoint”。Skill 会把文件写入该工作目录的 `session/topics/<topic-id>`。

## Chrome 查看

双击 `viewer\topic-viewer.html`，或用 Google Chrome 打开它，点击“打开 Topic 文件夹”，选择项目中的 `session\topics\T-...` 目录。Chrome 支持文件夹选择器；也可以把单个 Topic 文件夹拖到页面左侧。查看页只在浏览器内读取 Markdown，不写回文件、不上传、不执行命令。

如果浏览器不支持文件夹选择器，可以改用“选择 Markdown 文件”，一次选择同一 Topic 的四个 Markdown 文件。

## 包内容

- `SKILL.md`：Agent 的 Topic 维护规则。
- `references/protocol.md`：文件字段、模板和旧台账兼容规则。
- `viewer/topic-viewer.html`：零依赖的人类只读查看页。
