# 会话决定

## D2：Topic 台账跨 Agent MVP（2026-09-10）

- 已确认：Topic 是人的工作单元，Chat 是交流容器，Task 是执行记录；Topic 使用普通 Markdown 文件协议，跨 ZCode、Codex 和其他 Agent 复用。
- 已确认：新 Topic 放在 `session/topics/<topic-id>/`，由 `topic.md`、`events.md`、`decisions.md`、`handoff.md` 组成；旧 Q 编号台账继续保留并可被 Topic 引用。
- 已确认：MVP 查看页只读本地文件，不上传、不执行命令、不要求在线 LLM。
- 未验证：公司 ZCode 的实际 Skill 根目录和安装后的自动发现行为；需要公司机点验后再决定是否原地替换旧 `session-ledger`。

## D1：自动化交互测试分层

- 已确认：Elwright 的按钮与页面交互优先由 Playwright 浏览器端到端测试验证；Rust 单元测试和 Vitest 继续覆盖核心、Bridge 与纯逻辑。
- 未验证：Tauri 真正的原生文件选择、软件启动和终端 PTY 行为需要独立桌面壳冒烟测试，不能由浏览器预览替代。
- 已验证：浏览器冒烟测试使用独立上下文和清空后的 localStorage，资源仅保存 `virtual://` URI 字符串，不访问真实文件或 `~/.elwright/`。
