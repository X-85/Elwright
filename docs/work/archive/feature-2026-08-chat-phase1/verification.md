# Verification

2026-08-22，chat 阶段①（对话基础）：

## 自动化

1. **cargo test 全量**：31 passed / 0 failed，含两个新增 mock 端点单测——
   - `chat_messages_round_trips_via_mock_endpoint`：std TcpListener 单连接 mock，验证 POST 路径、Bearer 鉴权、model 字段、多轮 role 按原顺序进 body、响应解析取 `choices[0].message.content`。
   - `chat_is_two_message_wrapper_of_chat_messages`：`chat(system,user)` = `[system,user]` 封装；无 api_key 时请求不带 Authorization（invoke 路径行为不变）。
2. **前端 `npm run build`**：通过（TS/Vue 编译零错误；chunk 体积 warning 为既有状态）。
3. **不可信 Markdown 注入样本实测**（node 直跑 `lib/safeMarkdown.ts`，2026-08-22）：
   - 拦截：`<script>alert(1)</script>`、`<img src=x onerror=...>`、`javascript:` / `JaVaScRiPt:` 协议链接、href 带引号、`data:` 协议——输出中均无对应可执行标记。
   - 正常：标题/粗体/https 链接渲染正确；代码块内 `<div>` 以转义显示（`&lt;div&gt;`，页面呈现 `<div>`）未破坏；复制按钮注入。
4. **CLI 真实二进制端到端**（mock LLM @127.0.0.1:18777）：`ew invoke weekly-report` 返回 `MOCK-OK roles=system->user`（重构后的 chat→chat_messages 链路 + 真实 HTTP）；撤掉 base_url 后照常中文报错并降级离线 SOP。
5. **Tauri 壳编译**：`cargo build`（main.rs + chat_completion IPC 注册）通过。

## 浏览器预览模式（IAB 自动化，2026-08-23）

`npm run dev` + In-app Browser 自动化，全程通过：

1. **视图切换**：侧栏「💬 AI 对话」点击后 `[active]` 标记正确，main 区从能力列表切到对话页；切回「🧰 能力工具箱」后过滤器/搜索/计数/能力列表完整恢复。
2. **预览降级态**：对话页头部渲染「AI 对话」标题 + 状态徽标「预览模式 · 不可对话」+ 降级文案「【预览模式】AI 对话仅在桌面应用可用…」+ 输入框 + 发送按钮（空输入时禁用）。
3. **预览错误路径**：输入「你好」→ 发送按钮启用 → 点击后 `chat()` 抛错被捕获 → 错误气泡「【预览模式】浏览器无法发起 AI 对话…」+ 「↻ 重试」按钮；发送后输入清空、按钮回禁用。

## 桌面壳构建与启动（macOS，2026-08-23）

1. **构建**：`tauri build --debug --bundles app` 通过——产出 `Elwright.app`（`src-tauri/target/debug/bundle/macos/`），`chat_completion` IPC 编译进 bundle（仅 4 条既有 non_snake_case warning，非新代码）。
2. **启动自检**：mock LLM @127.0.0.1:18899 + `ELWRIGHT_LLM_BASE_URL` 环境变量，直接跑 `target/debug/elwright` 二进制 4 秒进程存活、boot log 无 panic——新代码不导致启动崩溃。

## 未自主验证（需人在原生窗口操作）

真实多轮对话的 GUI 交互（发送→回复→追问、停止丢弃、失败重试、未配置引导态、⚙ 设置后状态条刷新、代码块复制）需在原生 Tauri 窗口操作——IAB/CDP 无法驱动 macOS WKWebView 原生窗口。该路径的 **IPC 逻辑层** 已被 cargo 单测（mock TcpListener）与 CLI `ew invoke` mock 端到端覆盖；GUI 交互层为唯一未验证缝。
