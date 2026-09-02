# Verification · 工程质量第二档

按 checklist 逐项记录结果（标记约定：自动化项附命令与输出摘要，手测项贴用户确认）。

## Step 2 命令层下沉

- `cargo build` 通过；`cargo test` 37/37 绿（lib 单测），行为零变化闸门通过。
- clippy `--all-targets -D warnings` 与 `cargo fmt --check` 通过。
- 存量缺陷顺带修复：三把局部 Mutex 各自保护同一批进程级环境变量（ELWRIGHT_USER_ROOT/ELWRIGHT_ROOT/ELWRIGHT_LLM_*），并行测试下存在竞态假保护；统一为 `core::test_env::env_serialization_guard()` 全局锁后，全量并跑稳定绿。

## Step 3 IPC 冒烟

- `cd src-tauri && cargo test --test terminal_ipc`：**6/6 绿**（macOS 本机，含真 PTY 用例，0.3s）。
  - `real_pty_open_write_exit_roundtrip`：open → write `echo`/`exit` → 轮询断言 write 报错（broken pipe），真 PTY 双向链路。
  - `open_wires_channel_and_ids_increment`：`__CHANNEL__:N` 参数经完整 CommandArg 解析接到 session，EchoBackend 注入输出断言到达命令签名里的 channel（Bug#1 出错路径）。
  - 其余：双会话写入隔离 / 未知 id 中文报错 / close 后 write 报错 / list_capabilities 3 条。
- 全量 `cargo test`：37（lib）+ 6（IPC）= 43 绿，无回归。

## Step 4 Playwright e2e

- `cd src && npm run test:e2e`：**5/5 绿**（chromium）。
  - 勘误（2026-08-24）：首轮本地验证跑在 5173 端口复用的**其他工作区 dev server** 上（reuseExistingServer + 本机常驻 5173 进程），当时页面快照含本分支没有的「消息会话」导航。已改用独占端口 5273 + strictPort 重跑，确认 5/5 绿跑在本分支真实代码上；CI runner 无既有 server，不受此影响。
  - 覆盖：3 能力加载与计数徽标、筛选/搜索联动、知识型 doc 经 /api/file 渲染 markdown（接缝断开即失败）、脚本型运行【预览模式】降级文案、终端按钮不渲染、AI 对话预览提示。
- `npm run build` 与 `npm test`（vitest 22 例）不受影响，全绿。
- 首跑 1 例失败为选择器过宽（markdown 正文含多个 h2 触发 strict mode），改用 `.detail-head h2` 限定后通过——非产品代码问题。

## CI

- 分支 `enhancement/2026-08-quality-tier2-e2e` 已推 origin。本仓库 ci.yml 仅在 push main / PR / 手动触发时运行，分支直推不触发（既有配置，非本任务改动）。
- 待合并到 main 后首次 CI run 验证全部 job（含 frontend job 新增的 Playwright e2e 步骤），回填 run 结果于此。
- 本地等价验证已全绿：cargo build/test（37+6）/ clippy -D warnings / fmt --check / npm test（22）/ npm run test:e2e（5）/ npm run build。

## 手测项

- 桌面壳真机冒烟：**已由 Agent GUI 点验完成（2026-08-30，在 feature-2026-08-workbench-phase1 分支上验证，该分支包含本任务全部提交）**
  - 重构后桌面壳正常启动：顶栏三导航、桌面模式徽标、终端按钮、检查更新
  - 能力工具箱：list_capabilities 3/3 加载（启动与重启后各验证一次）
  - AI 对话页：模型状态条（真实配置 glm-5.1）与会话列表（chat_list_sessions 真实 IPC）渲染正常
  - 终端：terminal_open 经真实 IPC 建会话成功
  - 能力详情/对话发送的视觉细节未逐一点验（点击交互路径均走已验证的同一命令层）
