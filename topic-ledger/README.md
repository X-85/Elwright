# Topic Ledger MVP

这是一个可复制到 ZCode、Codex 或其他支持 Markdown Skill 的 Agent 中使用的 Topic 台账 Skill。

## Windows 使用

从仓库根目录运行：

```powershell
powershell -ExecutionPolicy Bypass -File .\topic-ledger\install.ps1
```

脚本会优先寻找已有的 `.zcode\skills`、`.codex\skills` 目录；也可以显式指定：

```powershell
powershell -ExecutionPolicy Bypass -File .\topic-ledger\install.ps1 -SkillRoot "$env:USERPROFILE\.zcode\skills" -SkillName topic-ledger
```

默认安装为 `$topic-ledger`，不会覆盖原来的 `$session-ledger`。如果希望保留原命令名并升级它：

```powershell
powershell -ExecutionPolicy Bypass -File .\topic-ledger\install.ps1 -SkillRoot "$env:USERPROFILE\.zcode\skills" -SkillName session-ledger
```

已有目录会先改名为带时间戳的 `.backup-*`，方便回退。使用 `-SkillName session-ledger` 时，脚本也会同步修改 Skill frontmatter，保留原 `$session-ledger` 调用名。

## 查看 Topic

双击 `viewer\topic-viewer.html`，点击“打开 Topic 文件夹”，选择项目中的 `session\topics\T-...` 目录即可。页面只在浏览器内读取 Markdown，不会写回文件。
