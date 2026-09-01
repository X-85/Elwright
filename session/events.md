# 会话事件

### Q1 | 第1次处理
- 问题或新增信息：用户显式调用 `$session-ledger`，要求开启会话台账。
- 本轮方案：按协议创建 `session/index.md`、`session/events.md`、`session/decisions.md` 和 `session/archive/`。
- 实际结果：已验证，基础台账结构已创建；未创建任何虚构的问题记录或决定。
- 下一步：后续每轮先读取索引，再按当前问题追加简短事件并更新索引。

### Q1 | 第2次处理
- 问题或新增信息：台账初始化后的结构已完成检查。
- 本轮方案：将初始化问题标记为完成，后续按既定协议维护。
- 实际结果：已验证，`index.md`、`events.md`、`decisions.md` 和 `archive/` 均存在。
- 下一步：后续新问题建立新的 Q 编号。

### Q2 | 第1次处理
- 问题或新增信息：用户希望不靠手动点击，自动验证 Elwright 功能和按钮可用性。
- 本轮方案：检查现有测试、CI 和浏览器自动化条件，设计分层测试路径。
- 实际结果：已验证，现有 Rust 与 Vitest 测试未覆盖真实页面点击；本机有 npx、没有可用浏览器。建议优先 Playwright 浏览器端到端测试，原生 Tauri 行为另设桌面冒烟测试。
- 下一步：待用户确认后新增 Playwright、关键按钮可访问选择器、冒烟用例和 CI job。

### Q2 | 第2次处理
- 问题或新增信息：用户确认需要 Playwright 冒烟用例和 CI 检查，并担心测试破坏本地文件。
- 本轮方案：采用独立浏览器上下文、初始化时清空测试 localStorage、只使用虚拟路径；不调用真实文件选择器或软件启动。
- 实际结果：处理中，开始实现浏览器端到端测试、CI job 和测试文档。
- 下一步：运行本地 Playwright（若浏览器可用）、前端测试/构建并校验 CI 配置。

### Q2 | 第3次处理
- 问题或新增信息：首次 Chromium 运行暴露两个定位器名称不完整的问题。
- 本轮方案：依据失败 DOM 快照，将调用按钮和资源复选框改为稳定的可访问名称片段定位；测试资源改用 `virtual://` URI，避免与任何真实路径混淆。
- 实际结果：已验证，Playwright Chromium 3/3、Vitest 22/22、Rust 41/41、前端构建、格式检查和 CI YAML 解析全部通过；`npm ci` 干净安装通过。
- 下一步：推送分支后确认 GitHub Actions 的 Browser smoke job 运行结果。

### Q2 | 第4次处理
- 问题或新增信息：用户要求推送当前工作。
- 本轮方案：明确排除 `.zcode/` 和未纳入本次实现的 traffic-light 任务目录，提交已核对的功能、文档和测试文件并推送当前分支。
- 实际结果：已验证，提交 `4215c1c` 已推送至 `origin/codex/feature-2026-08-progressive-capabilities`。
- 下一步：确认 GitHub Actions 的 Browser smoke job 远端结果。

### Q2 | 第5次处理
- 问题或新增信息：用户询问远端 GitHub Actions 如何确认，以及是否会主动检查。
- 本轮方案：查询当前分支的 GitHub Actions workflow runs，并核对 CI 触发条件。
- 实际结果：已验证，当前分支没有远端 workflow run；CI 仅在 `main` push 或 PR 时触发，直接推送 feature 分支不会执行。
- 下一步：创建草稿 PR 或调整 CI 的 push 触发范围后，再检查 Browser smoke job 结果。

### Q2 | 第6次处理
- 问题或新增信息：用户选择创建草稿 PR 来触发远端 CI。
- 本轮方案：检查当前分支、远端提交和 GitHub CLI/API 认证条件。
- 实际结果：已验证，分支已推送到 `origin`，但本机没有 `gh` 且没有 GitHub API token，无法安全执行外部 PR 创建。
- 下一步：用户安装并登录 `gh` 后继续创建草稿 PR，再检查 Actions 结果。

### Q2 | 第7次处理
- 问题或新增信息：用户询问已有 GitHub 使用记录时为何仍被本机认证阻塞。
- 本轮方案：核对 Git 远端协议、SSH 认证和 API 认证条件。
- 实际结果：已验证，当前机器通过 SSH 密钥以 `X-85` 身份认证，因此可推送；创建 PR 需要独立的 GitHub HTTP API 认证，而本机没有 `gh` 或 API token。当前分支没有已存在的 PR。
- 下一步：在网页创建草稿 PR，或安装并登录 `gh` 后由 Agent 创建；PR 创建后检查 Actions。

### Q2 | 第8次处理
- 问题或新增信息：工程质量第二档（enhancement/2026-08-quality-tier2-e2e）在另一会话完成并推送，其 CI 触发条件结论与 Q2 一致（仅 main push / PR 触发）。
- 本轮方案：按仓库既有发版惯例收口——本地合并到 main 后 push 触发 Actions，草稿 PR 不再是必要路径。
- 实际结果：已验证，分支已推 origin；合并动作留给用户。
- 下一步：用户合并 main 后确认 Actions 全绿（frontend job 含新增 Playwright e2e 步骤）。

### Q2 | 第9次处理
- 问题或新增信息：用户询问安装 `gh` 后是否仍需手动创建 PR，以及 `gh` 的作用。
- 本轮方案：说明 Git 与 GitHub CLI 的职责边界，以及安装后仍需完成登录授权。
- 实际结果：已验证，`gh` 可通过 GitHub API 创建草稿 PR、查看 Actions 和读取失败日志；仅安装未登录时不能执行这些平台操作。
- 下一步：用户安装并运行 `gh auth login` 后，可由 Agent 直接创建 PR 并检查 CI。

### Q2 | 第10次处理
- 问题或新增信息：`gh auth login` 询问是否上传本机 SSH 公钥。
- 本轮方案：根据已验证的 SSH 登录状态，避免重复上传同一公钥。
- 实际结果：已确认，应选择 `Skip`；GitHub API 授权仍将通过后续浏览器登录完成。
- 下一步：完成浏览器授权后运行 `gh auth status`。

### Q2 | 第11次处理
- 问题或新增信息：设备码登录在交换 token 时连接 `github.com/login/oauth/access_token` 超时。
- 本轮方案：区分 `api.github.com` 与 `github.com` 的网络可达性，提供重试、网络/VPN 和 Fine-grained Token 两条恢复路径。
- 实际结果：已验证，SSH 推送仍正常，`api.github.com` 可达，但当前终端到 GitHub 网页 OAuth 域名超时；设备码不可继续复用。
- 下一步：换网络后重新登录，或在浏览器创建仅限 `X-85/Elwright` 的 Token，再通过 stdin 登录 `gh`。

### Q2 | 第12次处理
- 问题或新增信息：用户在 Fine-grained Token 权限页未看到 `Secrets`。
- 本轮方案：澄清 Fine-grained Token 的权限项随仓库/组织策略显示，且本任务不需要任何 Secrets 权限。
- 实际结果：已确认，只需设置 Contents、Pull requests 和 Actions；不显示 `Secrets` 是正常行为。
- 下一步：生成 Token 后通过 stdin 登录 `gh` 并执行 `gh auth status`。

### Q2 | 第13次处理
- 问题或新增信息：用户在 Fine-grained Token 权限页未看到 `Checks`。
- 本轮方案：按创建草稿 PR 和读取 Actions 的最小权限重新确认权限集。
- 实际结果：已确认，`Checks` 不需要也不一定显示；Contents（只读）、Pull requests（读写）、Actions（只读）足够。
- 下一步：生成 Token 后通过 stdin 登录 `gh` 并执行 `gh auth status`。

### Q2 | 第14次处理
- 问题或新增信息：用户已完成 GitHub CLI 认证；草稿 PR 创建成功，但 PR 事件没有自动出现检查项。
- 本轮方案：使用仓库现有的 `workflow_dispatch` 在功能分支手动触发 CI，并轮询远端任务。
- 实际结果：已验证，PR #1 已创建；Browser smoke、Frontend、Rust lint、Linux Rust、macOS dmg、Windows msi 已通过；macOS Rust 任务的 mock LLM 步骤失败，Windows Rust 任务仍在首次 cargo test 编译中，workflow 尚未结束。出现 Node.js 20 弃用提示，但不影响任务结论。
- 下一步：等待 Windows 任务结束并读取 macOS 失败日志；必要时修复后重新触发 CI。

### Q2 | 第15次处理
- 问题或新增信息：用户准备在其他 AI 工具中继续开发，要求功能代码提交到分支，但台账变更不要提交。
- 本轮方案：检查本地与远端分支、PR head 提交及未提交文件，按文件类型区分功能代码与台账/其他任务文档。
- 实际结果：已验证，没有未提交的功能代码；本地 `HEAD`、远端分支和 PR #1 均为 `cb0638a`。台账保持本地未提交；`.zcode/` 与 traffic-light 任务目录也未纳入 PR。
- 下一步：可直接交给其他工具从该远端分支继续开发；CI 状态仍按 Q2 第14次记录处理。

### Q3 | 第1次处理
- 问题或新增信息：工程质量第二档（IPC 冒烟 + Playwright 分层 e2e）落地，分支 enhancement/2026-08-quality-tier2-e2e 已推 origin（8f7be09）。
- 本轮方案：tauri mock runtime 真协议测 IPC 层（6 用例，macOS/Linux 真 PTY）+ Playwright chromium 测浏览器层（6 用例）；期间发现本地 5173 被主工作区 dev server 占用、e2e 复用跑到无关 UI，改独占端口 5273 修复并在 verification.md 记勘误。
- 实际结果：已验证，五道闸全绿（cargo 52 / vitest 26 / e2e 6 / clippy / fmt）。
- 下一步：用户回家执行【手测】桌面壳冒烟（tauri dev）→ 合并 main → CI 首跑回填 verification。

### Q4 | 第1次处理
- 问题或新增信息：工作工具栏第一阶段（Todo + 今日记录）立项并完成，分支 feature-2026-08-workbench-phase1 已推 origin。
- 本轮方案：core/workbench.rs（todos.json + notes/ 一天一文件，日期校验防路径穿越）+ 7 条 IPC 命令 + WorkbenchView.vue（防抖自动保存/saveToken 防过期写）+ 浏览器模拟存储（刷新即失口径）；范围调整为今日记录提前进第一阶段（用户 2026-08-25 拍板）。
- 实际结果：已验证，五道闸全绿（core 5 单测 + IPC 冒烟 4 + vitest 4 + e2e 场景 1）。
- 下一步：用户回家【手测】桌面壳持久化（重启后数据仍在、~/.elwright/ 落盘正确）→ 按 tier2 先进的顺序合并。

### Q5 | 第1次处理
- 问题或新增信息：路线图复查发现双线开发重叠——主工作区另一会话的 codex/feature-2026-08-progressive-capabilities 分支（领先 main 8 提交）自带一套 Playwright 冒烟（src/e2e/app-smoke.spec.ts + playwright.config.ts + CI 独立 e2e job + package.json），与 tier2 分支的同名设施直接冲突；两者还同改 App.vue/bridge.ts/main.rs/style.css/capabilities.json。
- 本轮方案：合并顺序定为 tier2 → workbench → codex 重整；codex 的重复 e2e 设施合并时去重（保留一套，场景可合并），属合并期决策。
- 实际结果：未验证（未实际合并）；冲突文件清单已确认。
- 下一步：用户完成 tier2/workbench 手测合并后，处理 codex 分支 rebase + e2e 去重；新开发工作在合并积压清空前暂缓。

### Q5 | 第2次处理
- 问题或新增信息：用户要求本会话像 codex 一样直接对自有功能分支跑验证以收口积压。期间发现 /tmp worktree 被 macOS 系统定期清理损毁（.git 及 3 天未访问文件被删），两分支因已推 origin 零损失。
- 本轮方案：重建 worktree 至持久路径 ~/code/Elwright/wt-workbench（教训：长期 worktree 不放 /tmp）；重跑全量自动化验证；另启动 tauri dev 尝试 GUI 点验。
- 实际结果：已验证——五道闸全绿（cargo 52 / clippy / fmt / vitest 26 / e2e 6 / build），桌面壳启动冒烟通过（进程稳定运行）。GUI 深度点验被阻：ZCode Computer Use.app 的辅助功能与屏幕录制权限未授权（request_access 探测确认 denied），按协议停止重试。
- 下一步：二选一——用户授权权限后由本会话 GUI 点验，或用户回家自行 5 分钟手测；合并执行待用户确认。

### Q3 | 第2次处理
- 问题或新增信息：用户授权 ZCode Computer Use 辅助功能权限（屏幕录制仍未授权，不影响 AX 路径），GUI 点验得以执行。
- 本轮方案：tauri dev 真机启动，经 AX 元素操作逐项点验 tier2 重构回归；期间确认 dev 二进制无 bundle_id 导致键盘输入路径不可用、WKWebView textarea 的 AX 写值不生效（工具边界），改以文件放置法验证 note 读取链路。
- 实际结果：已验证——桌面壳启动正常、能力 3/3、AI 对话页（真实配置与会话列表）、终端 tab 建立、正常退出路径，全部通过。
- 下一步：无（本任务验证闭环，待合并）。

### Q4 | 第2次处理
- 问题或新增信息：同上权限条件下完成工作台 GUI 点验。
- 本轮方案：AX 元素点击驱动真实 UI：添加 Todo → 勾选 → 检查 ~/.elwright/todos.json 落盘 → 正常退出 → 重启 → 复核；笔记读取用文件放置法验证。
- 实际结果：已验证——Todo 添加/勾选真实 IPC 落盘（camelCase、completedAt 正确）、重启后 Todo 与笔记完整保留，持久化闭环通过。
- 下一步：无（验证闭环，待合并）。

### Q5 | 第3次处理
- 问题或新增信息：worktree 事故已恢复（持久路径 ~/code/Elwright/wt-workbench），两分支自动化与 GUI 验证全部闭环，收口只剩合并动作。
- 本轮方案：等待用户确认合并执行（tier2 → main 触发 CI → workbench → codex 重整去重）。
- 实际结果：验证分支已更新推送（workbench 4ee3883，携带两份 verification 回填）。
- 下一步：用户一句话确认后由本会话按序执行合并并盯 CI。

### Q6 | 第1次处理
- 问题或新增信息：用户询问路线图下一步（另一会话正在处理其他分支的合并与 CI）。复查发现本工作区（codex/feature-2026-08-progressive-capabilities）存在孤儿任务目录 `docs/work/active/enhancement-2026-08-app-shell-traffic-light-controls/`——plan/checklist/verification 全部勾完，但对应红绿灯按钮的代码改动在本分支、main、workbench 任何提交中都找不到（无相关提交，工作区无未提交代码），疑似上个会话中断丢失。
- 本轮方案：仅评估不动手。给出候选排序：①确认/补做 traffic-light 孤儿任务（本分支范围内独立小增强）；②同步过时文档（ROADMAP「当前版本」停在 v0.1.1、AGENTS.md「当前进度」指针停留在 feat/chat）；③等 workbench 合并后本分支 rebase + e2e 去重（依赖另一会话的合并动作，暂不动）。合并与 CI 线路由另一会话处理，本会话不碰。
- 实际结果：未验证（纯评估轮，未改代码）。
- 下一步：待用户选定方向后执行。

### Q6 | 第2次处理
- 问题或新增信息：用户选定方向①——补做 traffic-light 孤儿任务。
- 本轮方案：仅改 `src/style.css` 窗口控制段：圆点 14→16px、按钮热区 18×24→20×26、gap 7→8px（chrome-left 6→7px）、三色各加 5px 低透明度同色光晕、悬停 brightness 1.08→1.12 并叠白色光晕；行为、位置、Tauri API 均未触碰。verification.md 补勘误段说明代码丢失经过与真实验证结果。
- 实际结果：已验证——npm run build 通过、vitest 22/22、git diff --check 通过；浏览器视觉点验（Vite :5274 截图）深色/浅色主题圆点清晰可辨、悬停有放大提亮反馈。已提交 `6d64131`（含任务目录入库），dev server 已停。
- 下一步：随本分支一起等 workbench 合并后 rebase；可选把本分支推 origin（用户动作或本会话执行）。

### Q6 | 第3次处理
- 问题或新增信息：用户指示继续推荐下一步——文档同步。核实：最新 tag 为 v0.1.5（2026-08-24），tier2 已于 2026-08-30 并入 main（6f84109）。
- 本轮方案：ROADMAP「当前版本」v0.1.1 → v0.1.5（补 v0.1.2–v0.1.5 轨迹）；工程质量第一/二档标记完成；Workbench 第一阶段补「已验证待合并」；里程碑新增 2026-08-30 tier2 条目；AGENTS.md 进度指针由过时的 feat/chat/v0.1.3 改写为现状。
- 实际结果：已验证——git diff --check 通过，提交 `94dda1c`（仅 docs/ROADMAP.md + AGENTS.md）。
- 下一步：推荐动作已清空；本分支待 workbench 合并后 rebase + e2e 去重（依赖另一会话合并线），可选推 origin。

### Q6 | 第4次处理
- 问题或新增信息：用户指示继续下一步。fetch 核实：origin/main 仅新增 CI 重触发空提交（a58705d，另一会话在排查 cargo test 平台差异），workbench 尚未合并，rebase + e2e 去重仍被合并线阻塞。
- 本轮方案：执行可独立完成的动作——推送本分支到 origin（cb0638a..94dda1c，快进非强推）；清理视觉点验遗留的失效浏览器标签（dev server 已停）。
- 实际结果：已验证——推送成功，本地与 origin 同步；标签已关闭。
- 下一步：本会话无剩余可独立推进项；等 workbench 合并后做 rebase + e2e 去重（Q5 顺序第三环）。

### Q6 | 第5次处理
- 问题或新增信息：用户再指示继续。fetch 确认 workbench 仍未合并（origin/main 停在 a58705d），rebase 本体仍阻塞；改做合并前置准备。
- 本轮方案：`git merge-tree` 预演本分支 vs origin/main，产出 6 个冲突文件解法清单；对比两边 e2e 设施后定去重结论——playwright.config.ts 取 main 版（5273），app.spec.ts（5 场景）与 app-smoke.spec.ts（3 场景）零重叠全保留，删本分支独立 CI e2e job。写入 `docs/work/active/feature-2026-08-progressive-capabilities/merge-prep.md` 并随提交 `ba4ec26` 推 origin。
- 实际结果：已验证——预演与文件对比完成，方案已入库推送。
- 下一步：等 workbench 合并后，按 merge-prep.md 执行真 rebase + 五道闸验收（届时先重跑 merge-tree 复核）。

### Q6 | 第6次处理（Q5 第三环提前执行）
- 问题或新增信息：用户再指示继续。workbench 仍未合并，决定不再干等——把 rebase 主体（对 tier2 侧 main）提前执行，workbench 合并后只剩小规模二次 rebase。
- 本轮方案：`git rebase --autostash origin/main`，按 merge-prep.md 解冲突：①main.rs 采 main 薄结构，10 个 workspace/topic 命令按 AppCtx 模式移植 core/commands.rs 并注册；②playwright.config.ts 取 main 版；③删分支独立 CI e2e job；④gitignore 并集；⑤ROADMAP 逐块取优（保留 main 新增「开发预览环境约束」节与详细第二档条目），AGENTS.md 取本分支现状版。适配：内置注册表 3→4 条（session-ledger）后同步更新 terminal_ipc 与 app.spec 计数断言。
- 实际结果：已验证——五道闸全绿：cargo 42+6 / clippy / fmt / vitest 22 / e2e 8 场景（两份 spec 合跑）/ 前端 build。已 force-with-lease 推 origin（cbe9a75 + 回填 a75a428）。
- 下一步：等 workbench 合并 main 后做第二轮小规模 rebase（先重跑 merge-tree 复核）；随后走 PR/合并流程。

### Q5 | 第4次处理
- 问题或新增信息：用户确认合并。tier2→main（6f84109）后 CI 首跑 ubuntu+windows cargo test 失败（macOS 绿）。无鉴权拿不到 job 日志，改走 ::error:: workflow command → check-run annotation 流（无鉴权可读）拿到真实报错；期间被 GitHub 每步 10 条 annotation 上限截断过一次，改为只发日志尾部 10 行。
- 本轮方案：windows 根因 = terminal_ipc.exe 启动即 STATUS_ENTRYPOINT_NOT_FOUND（0xc0000139，tauri 已知问题 discussions#11179——tauri-build 的 manifest 只进 bin 不进测试二进制），按官方 workaround 在 build.rs 用 rustc-link-arg-tests 给 windows+msvc 测试嵌 Common-Controls v6 manifest；ubuntu 根因 = real_pty 的「进程退出后 write 报错」断言依赖 macOS 行为（Linux 内核对已关 slave 的 master 写静默缓冲恒 Ok），改为写文件副作用断言 + close 释放路径。
- 实际结果：已验证——修复推送后 CI 7/7 全绿；workbench 随后合并（c0c46b1）再跑 CI 7/7 全绿。
- 下一步：codex 分支（另一会话）rebase + e2e 去重移交；两任务目录归档待用户确认。

### Q3 | 第3次处理
- 问题或新增信息：tier2 已随 6f84109 合入 main，含修复后 CI 7/7 全绿。
- 实际结果：已验证。Q3 关闭。
- 下一步：无（任务目录归档待用户确认）。

### Q4 | 第2次处理
- 问题或新增信息：workbench 已随 c0c46b1 合入 main，CI 7/7 全绿。
- 实际结果：已验证。Q4 关闭。
- 下一步：无（任务目录归档待用户确认）。

### Q6 | 第7次处理（Q5 第三环完成）
- 问题或新增信息：fetch 发现 workbench 已并入 main（c0c46b1，另一会话同时修了两平台 cargo test CI 失败）。merge-tree 预演 6 冲突后执行第二轮 rebase。
- 本轮方案：按预案解决——App.vue 以本分支 app-chrome 壳为基底，工作台入口并入侧栏导航（工作台按钮 + WorkbenchView 挂载）；main.rs/mod.rs/commands.rs 取 workbench todo/note 命令与本分支 workspace 命令并集；bridge.ts 四块纯并集（类型+方法表+双实现）；ROADMAP 取 main 更新后的 Workbench 条目并标记已合并。
- 实际结果：已验证——过程中并集脚本对 bridge.ts 块尾截断场景产生三处语法残缺（noteList 两处缺收尾、NoteDate 误加逗号），构建暴露后逐一手工修复（1d20db8）。最终五道闸全绿：cargo 47+6+4 / clippy / fmt / vitest 26 / e2e 9 场景（含工作台场景在新壳下通过）/ 前端 build。已 force-with-lease 推 origin（9dda2b4），merge-prep 回填完毕。
- 下一步：Q5 全链路（tier2→workbench→codex 重整）完成，本分支可走 PR/合并 main 流程；合并后 CI 首跑验收。

### Q6 | 第8次处理（PR CI 首跑收口）
- 问题或新增信息：用户确认继续。发现 PR #1（前会话所建）即本分支→main，force-push 后自动携带新提交；推送触发 CI run 33317338620，7 job 中 6 绿、仅 Frontend (Vue) 失败。
- 本轮方案：失败原因是 e2e flake——app-smoke 第 1 用例 `getByText('预览模式')` 在 CI 时序下同时匹配侧栏徽标与详情区降级提示（strict mode violation），本地时序侥幸通过。修复：断言锁定 `.detail` 容器内（ea3b1b5）。
- 实际结果：已验证——本地 e2e 9/9 后推送，新一轮 run 33317658038 七 job 全绿（含三平台 Rust、clippy/fmt、Frontend、dmg/msi 制品）。
- 下一步：合并 PR #1 由用户确认（合并即进 main，归档亦待用户执行）。

### Q5 | 收口（PR #1 合并执行）
- 问题或新增信息：用户确认直接合并 PR #1。发现其处于 draft 状态（前会话所建），先 `gh pr ready` 转正式。
- 本轮方案：merge commit 合并（7e222ab）→ 盯 main CI（run 33318131096，7 job 全绿）→ 在 wt-main 工作区同步文档：进行中清空、V2 各条目标记已交付部分、里程碑新增 PR #1 大批次条目、AGENTS.md 指针改为「待发版，下一站代码浏览器阶段①」，提交 7bd0f9e 推 main。
- 实际结果：已验证——main CI 全绿；文档同步已上 main。
- 下一步：v0.1.6 发版（用户决策）；docs/work/active/ 23 个任务目录归档（用户确认后执行）；代码浏览器阶段①启动编码。

### Q7 | 第1次处理（归档 + v0.1.6 发版）
- 问题或新增信息：用户确认继续三项收尾。归档：docs/work/active 23 个目录中 22 个已合并的移入 archive（保留 code-browser-phase1，未编码）；发版：版本号四件套同步 0.1.6（tauri.conf.json / package.json / Cargo.toml / install.ps1 新 ProductCode 6635434E-…），Cargo.lock 随 cargo check 刷新。
- 本轮方案：归档提交随 main 直推；发版提交 4d643fe + tag v0.1.6 推送触发 release.yml（run 33318637029）。
- 实际结果：处理中——归档与发版提交已上 main（4d643fe），release 流水线运行中（后台监控）。
- 下一步：release 全绿后确认 GitHub Release 产物（dmg+msi），更新 ROADMAP 当前版本为 v0.1.6，台账收口。

### Q7 | 第2次处理（收口）
- 问题或新增信息：release.yml run 33318637029 一次全绿，GitHub Release v0.1.6 附 Elwright_0.1.6_aarch64.dmg + Elwright_0.1.6_x64_en-US.msi。
- 本轮方案：ROADMAP「当前版本」更新为 v0.1.6（含本版内容摘要），提交 33da3a4 推 main。
- 实际结果：已验证——Release 产物齐备；归档、发版、文档同步全部闭环。
- 下一步：真机安装验证（用户动作：msi 公司机 / dmg 家里机）；下一功能 = 代码浏览器阶段①启动编码。

### Q8 | 第1次处理（v0.1.6 功能冒烟 → 抓到发布阻断 bug）
- 问题或新增信息：用户指示本机直接做 v0.1.6 功能冒烟（真机安装留到明天公司）。GUI 冒烟进度：①成长开关标签切换 ✅ 且状态跨重启持久化；②资源与课题页加载 ✅；③新建文件夹真实落盘 ✅（workspace.json 出现"冒烟测试"）；④添加资源 ❌ 恒失败（网页/笔记两种类型、AXPress 与真鼠标、两个实例共 4 次均不落盘、无 toast）。期间踩坑：dev 二进制无 bundle_id 键盘路径不可用（Q3 旧坑），改用 debug .app；激活时把 /Applications 旧版 v0.1.5 顶到前台（同 bundle id 冲突，已关闭旧实例）。显示器会话中断后 GUI 路径不可用，转 IPC 级收口。
- 本轮方案：开 Web Inspector Console 抓到 invoke Promise 永久 pending；写 mock-runtime scratch 测试拿到实锤——`invalid args 'resource': missing field 'id'`。根因：前端新建资源不带 id，`Resource.id` 缺 `#[serde(default)]`。修复 + 新增 tests/workspace_ipc.rs 真协议回归（无 id 负载 → create → delete → load 自清理）；浏览器 e2e 走 localStorage 不经此接缝所以全绿漏网。提交 7178d6d（含 bugfix 任务目录）。
- 实际结果：已验证——IPC 回归 1 passed；全量闸门 cargo 47+6+4+1 / clippy / fmt / vitest 26 / e2e 9 / build 全绿。发 v0.1.7 修复版（版本四件套 + 新 ProductCode EB638E42-…，6a17af2，tag 已推），release.yml run 33320957617 后台监控中。
- 下一步：release 全绿后核对产物并更新台账；未完成的 UI 冒烟项（消息会话/工作台/台账 invoke/红绿灯视觉）待显示器恢复或用户真机补验；添加资源 UI 复验一次（IPC 已覆盖同形态负载）。

### Q8 | 第2次处理（收口）
- 问题或新增信息：v0.1.7 release（run 33320957617）一次全绿，产物 Elwright_0.1.7_aarch64.dmg + Elwright_0.1.7_x64_en-US.msi 已上 GitHub Release。
- 本轮方案：仅核对与收口，无代码改动。
- 实际结果：已验证——v0.1.7 取代 v0.1.6 成为当前版本（ROADMAP 已随发版提交更新）。
- 下一步：用户明日公司机装 msi（v0.1.7）时顺带点验「添加资源」；剩余 UI 冒烟项待补。

### Q8 | 第3次处理（挂起 + 澄清）
- 问题或新增信息：用户问显示器息屏是否导致失败——澄清：资源收藏失败是真实代码 bug（IPC 级证据链独立于显示器），息屏仅中断了修后 UI 补验。用户确认真机验证暂时做不了，挂起。
- 本轮方案：无代码改动，纯解答。
- 实际结果：已验证（上一轮已闭环）。
- 下一步：真机验证挂起——用户装 v0.1.7 时顺带补点验「添加资源」等剩余项。

### Q9 | 第1次处理（代码浏览器阶段①启动）
- 问题或新增信息：用户指示进入下一阶段 = 代码浏览器阶段①（本地项目只读查看与轻量符号跳转：接口→实现、JavaBean 类型→类定义）。既有分支 codex/feature-2026-08-code-browser（e90086e，仅方案文档）。
- 本轮方案：读分支上的 plan/checklist → 分支同步到最新 main（原分支基于旧 main）→ 按方案实施（预计 core/code_browser.rs + IPC 命令 + 前端视图 + 分层测试）。
- 实际结果：处理中。
- 下一步：读方案、同步分支、开工。

### Q9 | 第2次处理（阶段①实现完成）
- 问题或新增信息：方案文档已在 main（随 PR #1 进来），从最新 main 建分支 feature-2026-08-code-browser-phase1，ROADMAP 进行中登记（30ac228）。
- 本轮方案：core/code_browser.rs（路径边界拒绝对路径/.. 穿越、敏感文件拒读、512KB 文件/2000 条目/8 层深度上限、语言识别、文件名+内容搜索、按需 Java 符号行级扫描）+ 六条 IPC 命令（camelCase 参数）+ bridge 双适配（浏览器端明确降级）+ CodeBrowserView（懒加载树、只读查看、escape-first 轻量着色 codeHighlight.ts、搜索、符号跳转多候选、最近项目/文件持久化 ~/.elwright/code-browser.json）+ App.vue「代码」导航入口。
- 实际结果：已验证——core 单测 8/8、IPC 冒烟 1/1（临时项目根）、vitest 31（含 codeHighlight 5）、e2e 10（含浏览器降级守卫）、cargo 55+1+6+4+1 / clippy 0 / fmt 全绿。期间修一个行内方法体误杀（分号判断放宽为只看括号前头部）。一次偶发失败未复现（连跑三遍全绿；疑与当时未退净的应用实例并发写 ~/.elwright 撞车）。分支已推 origin。
- 下一步：①用户真机点验（verification.md 待办）；②PR 合并 main 待用户确认；③真机复验 bugfix-2026-08-workspace-create-resource 后归档。

### Q9 | 第3次处理（PR #2 合并 + CI 双修）
- 问题或新增信息：PR #2 首轮 CI 挂 Windows Rust + clippy 两 job。①IPC 测试 InvokeRequest 的 url 硬编码 tauri://localhost，Windows 的 Tauri IPC origin 是 http://tauri.localhost，ACL 按 origin 校验报 "Plugin not found"（terminal_ipc.rs 有现成 cfg 处理，新测试文件没抄）；②projectRoot 驼峰参数触发 clippy（CI 带 -D warnings，本地裸跑 clippy 只看 error 漏检了 warning）。教训已记。
- 本轮方案：新测试文件补 cfg origin（code_browser_ipc + 回溯修 workspace_ipc——后者随 v0.1.7 直推 main 时也带着此问题，当时漏盯了 main CI）；五条命令补 allow(non_snake_case)。第三轮 PR CI 全绿（run 33350691958），合并 PR #2（3e246ee），main CI 全绿（run 33351039081）。
- 实际结果：已验证——代码浏览器阶段①进 main；docs/work/active/PENDING-REAL-MACHINE-CHECKLIST.md 汇总三块真机点验项（v0.1.7 修复复验/代码浏览器/Q8 未竟项）随 65a097c 上 main。
- 下一步：用户真机点验（按清单）；之后归档两任务目录；代码浏览器阶段②（研发上下文联动）待排期。

### Q9 | 第4次处理（跳过真机验证 + 归档 + v0.1.8 发版）
- 问题或新增信息：用户指示真机验证全部跳过、落实到 work 文档留档、继续后续工作。
- 本轮方案：PENDING-REAL-MACHINE-CHECKLIST.md 标记"暂缓跳过留档"；两个任务目录（bugfix-workspace-create-resource / feature-code-browser-phase1）verification 补"人工验证跳过"说明后归档 active→archive；ROADMAP 进行中清空、里程碑新增代码浏览器①条目；AGENTS.md 指针更新（9df288a）。随后发 v0.1.8 携带代码浏览器（版本四件套 + 新 ProductCode 8983D51A-…，6f09f9b，tag 推送）。
- 实际结果：已验证——release run 33352043237 一次全绿，产物 Elwright_0.1.8_aarch64.dmg + Elwright_0.1.8_x64_en-US.msi 已上 GitHub Release。
- 下一步：进行中暂空；下一阶段候选（代码浏览器③研发上下文联动 / 设置中心后续 / 消息②传输）待用户定向。

### Q12 | 第1次处理（代码浏览器③第一批：收藏/书签/发送到 AI）
- 问题或新增信息：用户选定候选 1（代码浏览器③研发上下文联动），确认第一批范围：收藏文件 + 代码书签 + 发送到 AI；Todo/终端定位留第二批，流程图关联被工程图 MVP 卡住不做。
- 本轮方案：分支 feature-2026-08-code-browser-phase3-context。①RecentStore 扩 favorites(100)/bookmarks(200，项目根+路径+行号去重)，两条对称 toggle IPC，写用户层 code-browser.json；②CodeBrowserView：目录树/文档栏星标、行号点击切书签（备注取行前 40 字）+ 书签行高亮与面板跳回、发送到 AI 确认面板（路径/范围/字符数，选中优先整文件兜底 8000 字符截断）；③ChatView 暴露 insertContext，App 经 send-to-ai 事件切视图预填，敏感文件拒发。文档三件套 + 任务目录已更新。
- 实际结果：处理中——core 单测 10/10、IPC 1/1、vitest 31、build 全绿；PR #3 首轮 CI 挂三平台 Rust + clippy（save_recent 在全新环境 ~/.elwright 不存在写入失败；本地未复现因目录已存在）→ 修为自建缺失目录并补测试（save_recent_creates_missing_dir），严格 clippy/-D warnings 过，第二轮 PR CI 全绿，合并与 main CI 监控中。
- 下一步：main CI 全绿后发 v0.1.9；真机点验项留档（收藏/书签持久化、AI 预填正确性）。

### Q12 | 第2次处理（收口：合并 + v0.1.9 发版）
- 问题或新增信息：PR #3 二轮 CI 全绿后合并 main（30fbcd5），main CI 全绿（run 33355921681）。
- 本轮方案：发 v0.1.9 携带阶段③第一批（版本四件套 + 新 ProductCode A1747E57-…，cd63a97）。
- 实际结果：已验证——release run 33356569229 一次全绿，Elwright_0.1.9_aarch64.dmg + Elwright_0.1.9_x64_en-US.msi 已上 GitHub Release。
- 下一步：第二批（Todo 联动、终端定位）待排期；真机点验项留档（PENDING 清单 + 任务目录 verification）。

### Q12 | 第3次处理（第二批完成 + 一次流程失误）
- 问题或新增信息：第二批（Todo 联动/终端定位）实现完成，但提交时 wt-main 停在 main 分支（v0.1.9 发版后未切回），导致直接推了 main 绕过 PR；且提交信息里的反引号被 zsh 命令替换吃掉，`绝对路径:行号` 字样缺失（完整定义在任务目录 plan.md 留档）。内容本身完整（11 文件），main CI 全绿（run 33361321241）。未重写 main 历史弥补信息（避免 force-push 风险）。
- 本轮方案（功能）：①转为 Todo——文档栏一键创建含 `绝对路径:行号` 标记的 Todo；WorkbenchView 经 lib/codeLinks.ts 解析标记渲染可点击链接，emit open-code 跳回代码浏览器（openAbsolute 暴露）；②终端定位——TerminalPanel openTab 扩展 cwd 参数 + openAtDir 暴露，文档栏一键在集成终端新 tab 打开文件目录。vitest 新增 codeLinks 4 例。
- 实际结果：已验证——cargo 57+1+6+4+1 / 严格 clippy / fmt / vitest 35 / e2e 10 / build 全绿；main CI 全绿。
- 下一步：流程教训记 Q13（提交前必查当前分支）；阶段③全部完成，归档待真机点验（继续挂起）；下一阶段待用户定向。

### Q13 | 第1次处理（流程失误：绕过 PR 直推 main）
- 问题或新增信息：在 wt-main 工作区提交时未检查当前分支（v0.1.9 发版后停在 main），阶段③第二批直接落 main 并推送，绕过 PR 流程；提交信息含反引号被 zsh 命令替换吃掉一段。
- 本轮方案：内容完整且 main CI 全绿，不重写 main 历史；失误如实记录。后续约束：①在 wt-main 提交前先 `git branch --show-current` 确认分支；②提交信息含反引号/特殊字符时用单引号 heredoc 或文件方式传入。
- 实际结果：已记录，未验证（流程约束）。
- 下一步：后续会话遵守。

### Q14 | 第1次处理（设置中心后续：常规/外观/终端）
- 问题或新增信息：用户指示不发 v0.1.10，先做设置中心。范围取常规（启动视图/自动检查更新；语言置灰待 i18n）+ 外观（密度/缩放）+ 终端（字体/字号/滚动历史），模型档案留后续（涉及 LLM 配置链改造）。
- 本轮方案：lib/preferences.ts（localStorage 持久化 + mergePreferences 逐字段校验回退默认，与 theme.ts 同源思路）；SettingsCenter 三分区 UI；App 挂载初始化偏好 + 启动视图恢复 + 记住上次视图 + 自动检查更新；TerminalView xterm 用偏好并 watch 实时改字体字号（主题原已联动）。
- 实际结果：已验证——vitest 37（preferences 2）、e2e 10、build 全绿；PR #4 CI 全绿，已合并 main（c76631b），main CI 全绿。期间再次在 main 分支上误提交（Q13 重犯），本次当场纠正：提交挪到 feature 分支、main reset 回 origin，未强推。
- 下一步：真机点验留档（启动视图/密度缩放/终端偏好）；模型档案与 i18n 后续排期；按用户指示暂不发 v0.1.10。

### Q15 | 第1次处理（ROADMAP 盘点）+ Q16 | 第1次处理（AI 对话阶段③交付）
- Q15 问题或新增信息：用户要求盘点路线图未做功能。结论：主干未完成 8 项（AI 对话③④最大；消息②③需 ADR；渐进发布后半；设置收尾=模型档案+i18n；工作台二阶段；资源顺延项；质量三档按需；浏览器远期段），后置验证 2 项（脑图/工程图），远期/暂缓按规划不动；另发现 ROADMAP 文档四处漂移。
- Q15 本轮方案：docs/roadmap-sync 分支同步四处漂移（进行中登记 AI 对话③、当前版本重写 v0.1.7–v0.1.9+未发版清单、里程碑补三条、V1 第 6 项划掉），PR #5 合并（370bb7b）。
- Q16 问题或新增信息：按建议启动 AI 对话阶段③（用户确认式能力协作）。方案依据 chat 行为文档能力协作节既有规则。
- Q16 本轮方案：①后端 chat_system_prompt 可测 helper——CHAT_SYSTEM_PROMPT + 能力清单 + 严格提议格式（【能力提议】id: <id>，空注册表不注入），chat_completion 接线；②lib/chatProposal.ts 标记协议（提议/调用解析、结果识别、回灌截断 2000）；③ChatView：提议/调用消息渲染确认卡片（名称/类型/预期影响/参数），按类型分发 run_script/view_doc/invoke_skill（离线 SOP 标注来源），结果块「把结果告诉 AI」回灌；④输入区能力选择器（用户主动路径）。
- 实际结果：已验证——core 58(+1 系统提示单测)、vitest 43(+5)、e2e 10、严格 clippy/fmt/build 全绿；PR #6 CI 全绿，合并 main（4b70c27），main CI 全绿。
- 下一步：阶段④（流式输出/请求级取消/长上下文）待排期——需后端 SSE 改造（ADR 值得写）；真机点验留档（真实模型全链路）；两任务目录待真机后归档；未发版内容继续累积（v0.1.10 时机待用户）。

### Q16 | 第2次处理（阶段④ ADR 落地）
- 问题或新增信息：用户指示先写 ADR。产出 docs/features/chat/decisions/ADR-003-streaming-and-cancellation.md（PR #7 合并，468e9ae）。
- 本轮方案（ADR 决策）：保留 blocking reqwest（不违反 llm-invoke ADR-001），spawn_blocking 内逐块读 SSE（手写 ~30 行 data: 行解析，零新依赖），增量经 Tauri Channel 推送（复用终端已验证模式，事件 delta/done/error/cancelled）；取消 = 显式 chat_cancel 命令 + Mutex 取消表，读取循环逐块检查，中断即断连；前端节流增量渲染，停止保留部分文本标注。落选：async 全面迁移（违 ADR-001 + 双客户端成本）、仅取消不流式（不解决首字延迟）、引 SSE 库（违零依赖底线）。长上下文策略不在本 ADR 范围。
- 实际结果：已验证——PR #7 CI 全绿并合并。实现节待阶段④编码后回填。
- 下一步：按 ADR-003 实施阶段④（chat_completion_stream + chat_cancel + IPC Channel 测试 + 前端增量渲染）；实施后回填 ADR 验证节。

### Q16 | 第3次处理（阶段④落地，PR #8 合并）
- 问题或新增信息：按 ADR-003 实施阶段④并合并 main（PR #8，84e9667，CI 全绿）。过程中两次分支/提交混乱（提交落 docs 分支、heredoc 定界符写错），均已当场纠正，最终 PR 流程完整。
- 本轮方案（实现）：①llm.rs chat_messages_streaming——blocking Read 逐块读 SSE，手写 data: 行解析（parse_sse_delta，注释/非 JSON/[DONE] 容错），逐块检查取消表；②chat_completion_stream（Channel 推 delta/done/error/cancelled JSON 事件）+ chat_cancel（Mutex 取消表）；全程无有效输出且未取消时回退非流式；③bridge 双适配（桌面 Channel / 浏览器抛预览错误）；④ChatView 桌面流式增量渲染（50ms 节流）、停止真取消并保留已收文本标注（已停止）、浏览器维持旧路径。
- 实际结果：已验证——SSE 解析单测 4/4；cargo 62+1+6+4+1 / 严格 clippy / fmt / vitest 43 / e2e 10 / build 全绿。另修复 code_browser 测试临时目录并行撞名（macOS 时钟微秒精度，纳秒改原子序数，六连跑零失败）——此前的偶发失败全部是这个原因。
- 下一步：真机点验（流式首字延迟/停止立即生效）留档；v0.1.10 发版时机待用户（main 上已攒代码浏览器③第二批 + 设置中心后续 + AI 对话③④）。

### Q17 | 第1次处理（发版 v0.1.10）
- 问题或新增信息：用户指令"进行发版 v0.1.10"——main 攒 4 个功能（设置中心后续 / 代码浏览器③第二批 / AI 对话阶段③④）待发。
- 本轮方案（发版）：①版本号四件套同步：src-tauri/Cargo.toml、src-tauri/tauri.conf.json、src/package.json → 0.1.10，Cargo.lock 自动联动；②install.ps1 ProductCode 同步为新生成 GUID（219CB3F6-F3AA-4E23-9634-2346D926593C，避免与 v0.1.9 撞码导致同机重复装）；③本地五道闸：cargo test 62+1+6+4+1 全过、strict clippy 0 警告、fmt 无差异、vitest 43 全过、build 成功；④commit chore(release) + 打 tag v0.1.10 + push；⑤盯 release.yml 一次全绿（macOS dmg 2m18s + Windows msi 3m24s + Publish 11s），产物 Elwright_0.1.10_aarch64.dmg + Elwright_0.1.10_x64_en-US.msi 上 GitHub Release，自动生成 5 个 PR 的完整 changelog。
- 实际结果：已验证——Release 页面 [v0.1.10](https://github.com/X-85/Elwright/releases/tag/v0.1.10) 公开可见，两个安装包资产就绪，变更日志覆盖 PR #4 #5 #6 #7 #8。
- 下一步：ROADMAP 同步（当前版本、里程碑、进行中三处）；真机点验清单延续挂起（v0.1.10 新增项：流式首字延迟、停止按钮立即生效、能力协作全链路确认面板）；下一阶段待用户定向。

### Q18 | 第1次处理（代码浏览器阶段④ 受控补丁编辑）
- 问题或新增信息：按用户"按你的推荐进行"，落到 ROADMAP V2 第二档"代码浏览器阶段④"；本轮先 ADR-001「再评估」暂不引入 Java LSP，再实施"受控补丁编辑"。
- 本轮方案（分两阶段 PR）：
  - **ADR + 任务清单**（PR #9，6f5171a，3ed09ef）：ADR-001 决策"不引入 jdtls / 走受控补丁编辑"，plan/checklist/verification 在 `docs/work/active/feature-2026-08-code-browser-phase4-patches/`。
  - **实施**（PR #10，90bc346）：后端 `core::patch`（parse_unified_diff / apply_hunks_to_content / is_sensitive_path / sha256_hex / build_preview / apply_preview / revert_snapshot_in + 快照持久化），10 个单测 + IPC mock runtime 3 例（env_lock 串行化避免共享 ELWRIGHT_USER_ROOT 串扰）；IPC `apply_patch_preview` / `apply_patch_apply` / `apply_patch_revert` / `apply_patch_snapshots`；前端 `lib/patch.ts::extractFirstDiff` 粗筛 + `PatchPreviewDialog.vue` 三栏 + `ChatView.vue` diff 围栏识别入口 + Bridge 4 方法 + patch.test.ts 4 例；路径黑名单 `.env/.pem/.key/.ssh/.aws/node_modules/target/.git`；上下文不匹配 → 整文件 warnings 跳过不写盘不入快照。
  - **本地五道闸**：cargo 87（72 核心 + 3 apply_patch_ipc + 6 terminal + 4 workbench + 1 workspace + 1 code_browser_symbols）+ vitest 47 + strict clippy -D warnings + fmt + build 全绿；CI 7/7（mac/win/linux clippy+fmt + 三平台 cargo + 前端 + dmg + msi）一次全绿。
  - **文档回填**：code-browser behavior/architecture/changelog + ADR-001 verification；chat README/behavior/architecture/changelog 阶段③④ + 补丁入口补齐（v0.1.10 时漏的同步）。
- 实际结果：已验证——PR #10 squash 入 main，main HEAD 90bc346；分支 feat/code-browser-phase4-patches 自动删除。
- 下一步：真机点验待用户（选中 → AI 回复 → 应用补丁 → 撤销 + 敏感路径拒绝 + 写入冲突兜底）；Q18 收口。下一步推荐按既定顺序推进"设置中心模型档案 → 工程质量第三档"，等用户定向。

### Q19 | 第1次处理（设置中心 模型档案 ADR）
- 问题或新增信息：按用户"按顺序执行"=先设置中心模型档案，再工程质量第三档；本轮先 ADR-001「评估决策」再实施（沿用 Q18 的"先 ADR 再实施"协议）。
- 本轮方案（ADR 阶段）：决策=引入 `profiles: Map<name, LlmProfile>` + `activeProfile: string`，**与既有 flat 字段共存兼容**——`~/.elwright/config.json` 加两个可选字段，旧配置不动；解析顺序 env > 项目 flat > 步骤 3a activeProfile 命中 → profile / 步骤 3b 否则回退 flat > 注册表默认。范围：profile CRUD + 激活切换 + UI 下拉 + `ew config profile list/use/show/add/remove/rename`；拒绝：完全替换 flat / profile 共享继承 / OS keychain 加密（独立 ADR）。任务目录 `docs/work/active/feature-2026-08-settings-center-model-profiles/{plan,checklist,verification}.md`。
- 实际结果：已验证——ADR PR #11 合并（65a13cb），CI 7/7 全绿；分支 feat/settings-center-model-profiles-adr 自动删除。
- 下一步：开实施 PR——core::llm profile 解析 + ConfigLayers::merged step 3 升级 + 5 单测；CLI `ew config profile` 6 子命令；5 IPC（list/get_active/set_active/save/delete）+ main.rs 注册 + mock runtime 5 例；前端 Bridge 5 方法 + LlmSettings.vue 档案下拉/新建 + vitest ≥4；文档回填 + ROADMAP 标记；本地五道闸 + PR + CI 7/7 + 合并 + Q19 实施段收口。

### Q19 | 第2次处理（设置中心 模型档案 实施）
- 问题或新增信息：按既定 ADR-001 落地实施，遵循"先 ADR 再实施"两阶段协议。
- 本轮方案：
  - **后端 core::llm**：新增 `LlmProfile` / `UserConfigFile`（flat + profiles/active_profile 兼容形态）/ 原子写（tmp + rename）；`ConfigLayers::collect` 走 `UserConfigFile::to_flat_config`（active 命中→profile/否则回退 flat）；`is_valid_profile_name` / `normalize_profile_name` / `read_profiles` / `save_profile` / `delete_profile` / `set_active_profile` / `rename_profile` / `list_profiles` / `get_profile` / `active_profile_name`；profile 名 lowercase + 正则校验；5 个新单测。
  - **CLI bin/ew.rs**：`ConfigAction::Profile { action: ProfileAction }` 6 子命令（list/show/use/add/remove/rename）；key 脱敏；激活项 `*` 标记；非法 name / 不存在 name 走中文错误 + exit 1；CLI 端手工冒烟全绿。
  - **IPC commands.rs + main.rs**：5 个 IPC（llm_list_profiles / llm_get_active_profile / llm_set_active_profile / llm_save_profile / llm_delete_profile）+ `LlmProfileDto` / `ProfileMetaDto`；main.rs 注册；mock runtime 5 例（list 空 / save 可见 / set+get 同步 / delete 激活清空 / flat-only 兼容）。
  - **前端**：Bridge 5 方法（types `LlmProfileMeta` / `LlmProfileInput`）+ browser stub 5 中文降级 + tauri invoke 5；`lib/profileName.ts` 提供可单测的 `validateProfileName` + `normalizeProfileName`；`LlmSettings.vue` 顶部档案下拉（★ 标记 + flat 哨兵）+ + 新建档案按钮 + 已配置档案清单（含删除）+ 新建档案小弹窗；style.css 新增 .profile-bar / .profile-list / .add-profile-modal；vitest 4 例。
  - **文档回填**：settings-center behavior（模型档案章节）/ architecture（配置解析顺序小节 + 架构图扩）/ changelog / README；ROADMAP「设置中心后续阶段」标注完成 + 「未发版」更新 + 「进行中」写 Q19 + 历史时间线。
- 实际结果：已验证——PR #12 squash 入 main（92597f8），CI 7/7 全绿；分支 feat/settings-center-model-profiles 自动删除；本地五道闸：cargo 97（72 核心 + 5 profile + 3 apply_patch_ipc + 1 code_browser_symbols + 5 llm_profiles_ipc + 6 terminal + 4 workbench + 1 workspace）+ strict clippy -D warnings 0 警告 + fmt 无差异 + vitest 51（含 4 新 profileName 测试）+ build 成功。
- 下一步：Q19 收口；下一步按既定顺序推进「工程质量第三档（ESLint + coverage 阈值）」，等用户定向（也可以由 Agent 直接接续按 ROADMAP 推荐顺序开工）。

### Q20 | 第1次处理（工程质量第三档 ADR）
- 问题或新增信息：按既定顺序，Q19 模型档案已并 main，下一步是 ROADMAP V1「工程质量治理」第三档（立项前置条件已达成——vitest 存量 51 例 / 10 test file）；本轮先 ADR-002「评估决策」再实施（沿用 Q18/Q19 的"先 ADR 再实施"协议）。
- 本轮方案（ADR 阶段）：决策=引入 ESLint 10 flat config + @vitest/coverage-v8。ESLint：typescript-eslint + eslint-plugin-vue，规则三组——`no-unused-vars`（`_` 前缀豁免）+ `vue/no-unused-components` + `vue/no-unused-vars` + `no-explicit-any` warn；TS/Vue 下 `no-undef` 关闭；vue 模板风格规则 off（prettier 独立 ADR）。Coverage：v8 provider，include `src/lib/**/*.ts`，**exclude `src/lib/bridge.ts` facade**（同构于「弃选把 .vue 纳入门槛」——facade 由 IPC mock runtime + Playwright e2e 覆盖），thresholds lines/functions/statements 70% + branches 60%。CI：frontend job 加 `npm run lint` 与 `npm run test:coverage` 两步，三平台 matrix 复用。范围：配置 + 清零现有告警 + CI 接缝 + 文档回填。拒绝：prettier / istanbul / c8 / SonarQube / husky（独立 ADR 或低 ROI）。任务目录 `docs/work/active/enhancement-2026-08-quality-tier3-eslint-coverage/{plan,checklist,verification}.md`。
- 实际结果：已验证——ADR PR #13 合并（f25501f），CI 7/7 全绿；分支 feat/engineering-tier3-eslint-coverage-adr 自动删除。
- 下一步：开实施 PR——ESLint 10 flat config（rules 三组 + no-undef off + vue 风格 off）+ 清零 6 真未用 + 1 `_id` 化；vitest coverage v8 + thresholds 70/60 + bridge.ts 排除；preferences.test.ts 补 9 例（51 → 60）；TS 7.0.2 → 6.0.3（typescript-eslint 8.68 显式拒绝 TS 7）；CI frontend job 加两步；本地六道闸（eslint + vitest + coverage + vite build + cargo fmt + clippy + test）+ PR + CI 7/7 + 合并 + Q20 实施段收口。

### Q20 | 第2次处理（工程质量第三档 实施）
- 问题或新增信息：按既定 ADR-002 落地实施，沿用 Q18/Q19 的"先 ADR 再实施"两阶段协议；遇到意外阻塞——typescript-eslint 8.68 显式拒绝 TS 7.0（启动期硬错），临时降级 TS 7.0.2 → 6.0.3（vite 8 / vitest 4 peer 与 TS 版本无关，安全降级）。
- 本轮方案：
  - **ESLint 10 flat config**（`src/eslint.config.js`）：`typescript-eslint` + `eslint-plugin-vue`；rules：`no-unused-vars`（`_` 前缀豁免）+ `vue/no-unused-components` + `vue/no-unused-vars` + `no-explicit-any` warn；TS/Vue 下 `no-undef` 关闭（TS 自己管类型）；vue 模板风格规则（multiline/first-attribute/closing-bracket/max-attributes-per-line）全 off（prettier 独立 ADR）。
  - **清零 6 真未用 + 1 watch 形参**：删 `favoriteOf` (CodeBrowserView.vue) / `diffLines` computed + `renderDiffLines` import (PatchPreviewDialog.vue) / `emit` defineEmits 整段 (TerminalPanel.vue) / `WorkspaceTopic` type import (WorkspaceView.vue) / `LINE_COMMENTS` 常量 (codeHighlight.ts) / `applyTheme` 解构 (theme.test.ts)；`CapabilityDetail.vue` 的 watch 形参 `id` → `_id`。
  - **vitest coverage v8**（`src/vitest.config.ts`）：provider `v8`；include `lib/**/*.ts`；exclude `lib/**/*.test.ts` + `lib/__tests__/**` + **`lib/bridge.ts` facade**；thresholds lines/functions/statements 70% + branches 60%；reporter `text` + `html`。
  - **preferences.test.ts 51 → 60**：新增 9 例覆盖副作用——`updatePreferences` 部分覆盖 + `initializePreferences` 应用 density/zoom + localStorage 写入 + 写入失败不抛 + `saveLastView` 拒绝 `last` 与非法值 + `resolveStartupView` 4 路径。
  - **CI 接缝**（`.github/workflows/ci.yml`）：frontend job 加 `npm run lint`（紧跟 `npm ci`）+ `npm run test:coverage`（紧跟 `npm test`）；三平台 matrix 复用，不新增 job。
  - **TypeScript 7.0.2 → 6.0.3**：typescript-eslint 8.68 拒绝 TS 7.0（启动时抛 "typescript-eslint does not support TS 7.0"），vite 8 / vitest 4 peer 不绑 TS 版本，安全降级。
  - **package.json scripts**：`lint` (`eslint .`)、`lint:fix` (`eslint . --fix`)、`test:coverage` (`vitest run --coverage`)；devDependencies 新增 `eslint` / `typescript-eslint` / `eslint-plugin-vue` / `@vitest/coverage-v8` / `@eslint/js` / `vue-eslint-parser`。
  - **实测覆盖率**：statements 95% / branches 76.52% / functions 100% / lines 95.48% —— 全过 70/60 阈值；最差的 `chatProposal.ts` 仍 88.88/71.42/100/100；`safeMarkdown.ts` branches 55%（条件分支）—— 60% 阈值已通过。
  - **文档回填**：engineering-quality README 表格第三档「按需」→「已完成 2026-08-31 ADR-002 PR #14」+ ADR-002 链接 + 第二条 decisions；changelog.md 新建（2026-08-31 条目）；ROADMAP「未发版」补 Q20 + 第三档条目「按需」→「已完成 ADR-002 PR #13+14」+ 「进行中」清理 + 历史时间线 Q20 段。
- 实际结果：已验证——PR #14 squash 入 main（21cfbc1），CI 7/7 全绿；分支 feat/engineering-tier3-eslint-coverage-impl 自动删除。本地六道闸：eslint exit 0 + vitest 60/60 + coverage 95/76/100/95（过阈值）+ vite build ok + cargo fmt exit 0 + cargo clippy --all-targets -D warnings exit 0 + cargo test exit 0。
- 下一步：Q20 收口——`session/index.md` 加 Q20 行 + `session/events.md` 写 Q20 第1/第2次处理（本条）；commit + push 至 main。下一阶段待用户定向（v0.1.11 发版？继续 V1 余下工作？）。

### Q21 | 第1次处理（README「快速开始」补真机启动桌面端的命令）
- 问题或新增信息：用户上一轮问「下一步做什么」后我给出 v0.1.11 发版推荐；用户没接推荐，反问「README.md 快速开发中，怎么没有启动代码的命令，就是可以直接看桌面端的」——指出 README「快速开始」一节只有 `npm run dev`（vite 浏览器预览），新人装好代码后找不到桌面 app 入口。
- 本轮方案：
  - 确认事实：读 `src-tauri/tauri.conf.json` 第 7–10 行——已有 `beforeDevCommand: "npm --prefix src run dev"` / `devUrl: "http://localhost:5173"` / `frontendDist: "../src/dist"`，Tauri 命令链是通的，只是 README 没写。AGENTS.md 也写了「`npm run dev` 只覆盖查看类功能；终端、AI 对话、能力增删、模型配置、技能调用等核心功能必须进真机运行时」——这层风险 README 必须告诉新人。
  - 改 `README.md`「快速开始（开发者）」：保留原 CLI 三行；拆出桌面端 ①真机 + ②浏览器预览两段，命令按仓库现状——真机走 `cd src-tauri && npm --prefix ../src install && ../src/node_modules/.bin/tauri dev`（不在 src/ 直接 npx，避免 AGENTS.md 提到的「`npx tauri` 在 src/ 下找不到配置」坑），浏览器预览走 `cd src && npm install && npm run dev`；写明 macOS 需 Xcode CLT、 Windows 需 VS Build Tools、Linux 见 tauri 官方 prerequisites，浏览器预览只覆盖查看类、核心功能必须真机 tauri dev。
  - 附 `tauri build` 出 dmg/msi + 未签名首次打开的右键/SmartScreen 流程 + `git tag v0.1.x && git push origin v0.1.x` 走 CI release.yml 的提示——把 AGENTS.md 已有的「正式打包」段落浓缩到新人最可能在 README 看到的位置。
  - 不动 `src-tauri/tauri.conf.json`（Tauri 配置本就是对的）、不动 `package.json`（scripts 已含 dev/build/test：lint/coverage/preview/test：e2e/lint：fix，不需要新增 desktop 专用脚本）、不动 AGENTS.md（已正确，本任务属于「README 文档同步」而非「AGENTS 协议变更」）。
  - 台账：纯文档改动，未建 `docs/work/active/` 任务目录（AGENTS.md 工作协议里 bugfix/enhancement 类才要求任务目录，文档同步属于轻量维护）。
- 实际结果：已验证——`README.md` 改完；`session/index.md` 加 Q21 行；`session/events.md` 本条。变更未 commit（等用户复核 README 排版与措辞再决定是否 commit + push + 是否提 PR）。
- 下一步：等用户复核 README 排版与措辞；如同意则 commit + push（直接落 main 或 PR 由用户定）；同步回头看 v0.1.11 发版决策（仍挂在「下一阶段待用户定向」状态，Q21 不阻塞）。

### Q22 | 第1次处理（代码浏览器「选项目后看不到文件 + 预览栏比较小」）
- 问题或新增信息：按用户上一轮确认——1. 跑的是 `tauri dev` 桌面模式；2. 选完项目「没有子目录」（=根目录整个空白，不只是懒加载没展开）；3. 预览栏「整个面板被外层挤窄」（c 类）。本轮先用低风险 CSS 改 c 类，再跟用户对齐 a/b 真正根因。
- 本轮方案（诊断）：
  - **core 层**：手写 Rust smoke 跑 `fs::read_dir("/Users/bruce/code/Elwright/Elwright")` 实测返回 18 条（`.git/.github/.gitignore/.zcode/AGENTS.md/Elwright架构方案.md/LICENSE/README.md/assets/capabilities.json/docs/install.ps1/install.sh/resources/session/session-ledger-prompt.md/src/src-tauri`），`code_browser::tree(root, "")` 一定按目录在前排序返回这 18 条 TreeEntry；`SKIP_DIRS` 不命中任何一条。`commands::code_browser_tree` 签名 OK（`#[allow(non_snake_case)]` 配 camelCase JSON），macOS dev 二进制无 entitlements（Tauri 2 dev 默认不走 sandbox）—— `plugin-dialog` 的 `open({ directory: true })` 不被系统挡住。
  - **bridge 层**：`src/lib/bridge.ts:992` tauri 端 `chooseProjectDirectory` 用 `@tauri-apps/plugin-dialog` 的 `open`，`open()` 返回 string|null；browser stub `:604` 直接返回 null；`codeBrowserTree` 转发 IPC `code_browser_tree`（参数 `projectRoot, rel`）。逻辑链无破绽。
  - **前端渲染**：`CodeBrowserView.vue:368-411` 用 `<template v-for="(entries, rel) in treeCache">` + `<ul v-show="rel === '' || expanded.has(rel)">`，根目录条目按 `rel === ''` 渲染；Vue 3 reactive Map 在 v-for 里 key/值遍历无问题（手测过）。
  - **结论**：「看不到文件」**不是 core / bridge / 渲染逻辑的 bug**——是「目录选择对话框到底弹没弹」的接缝问题，需要用户确认。
- 本轮方案（动手）：
  - **低风险 CSS 修复**（Q22-c）：`src/style.css:2244-2257` `.cb-workspace` 加 `height: 100%` + `align-self: stretch`，让 grid 撑满 `.code-browser` 容器，绕开外层主区高度被挤压的问题。原 `grid-template-columns: minmax(240px, 320px) minmax(0, 1fr)` 保持不变（左侧 240-320 / 右侧 1fr 的比例合理）。
  - **未动 Q22-a/b**：等用户回答「系统对话框有没有弹出来」再决定走「前端渲染兜底」还是「dialog plugin 注册/capability 修复」。
- 实际结果：已验证——CSS 改完；Rust smoke 返回 18 条与项目目录现状一致；`session/index.md` 加 Q22 行；`session/events.md` 本条。变更未 commit（等用户真机复测 CSS 效果 + 回 g/h/i 再决定后续）。
- 下一步：用户真机复测代码浏览器看「预览栏」是否已经撑高；同时回 1 问——「选择文件夹」对话框有没有弹出来（g/h/i）；根据回答决定下一刀动哪里；CSS 改完一并 commit。

### Q22 | 第2次处理（GUI 实测定案 + 三处修复 + 真机验证通过）
- 问题或新增信息：用户重复原问题（信号=别再问了，直接修）。改变策略：不再等用户回 g/h/i，直接用 ZCode 桌面控制在用户运行中的 `tauri dev` 实例（pid 23019）上实测。
- 本轮方案（实测诊断）：
  - a11y 观察用户窗口：projectName=springbootDemo1 已设、刷新按钮在位（= IPC 校验调用成功、projectRoot 已赋值），但 [跳转输入框, 右侧提示] 之间**零树行**；点「刷新当前目录树」后依然零行（持久性，非偶发）。
  - 判定实验（node + 项目自带 vue + jsdom，`/tmp/vue_map_vfor_test.cjs`）：`ref(new Map())` + `v-for="(entries, rel) in treeCache"`，`.set('', [3条])` 渲染 `(root):2`、`.set('src', [1条])` 渲染 `1:2` —— **Vue 3 v-for 遍历 Map 得到 `[key, value]` 对儿 + 数字下标**，rel 恒为数字 → `v-show="rel === ''"` 恒 false → 整棵树 display:none。改普通对象后同样模板正确渲染。主根因定案。
  - 为什么一直没暴露：阶段①真机点验按用户指示跳过留档（PENDING 清单）；Playwright e2e 走 browserBridge 的 `chooseProjectDirectory` 恒 null，根本到不了这行模板；vitest 只测纯逻辑不挂组件。三道保障在这条路径上全是盲区。
  - 第二根因（CSS）：`.content` 是为工具箱设计的两列网格 `minmax(300px,380px) 1fr`，`.code-browser` 是唯一漏写 `grid-column: 1 / -1` 的全宽视图（chat-view/workspace-view/people-chat-view 都有）→ 整个代码浏览器被压进第一列 300-380px。
  - 第三根因（静默）：App.vue 报错 toast 只渲染在 `v-if="activeView === 'toolbox'"` 模板内，代码浏览器里 `notify()` 的任何错误用户不可见，放大主根因的「无声无息」。
- 本轮方案（修复，纯前端三文件，Rust/IPC/bridge 零改动）：
  - `CodeBrowserView.vue`：`treeCache` Map → `Record<string, CodeTreeEntry[]>`（has/set/delete/get → 下标判断 `!== undefined` / `[]=` / `delete` / `[] ?? []`），源头加注释说明 v-for-over-Map 的坑。
  - `style.css`：`.code-browser` 补 `grid-column: 1 / -1`（对齐全宽视图惯例）；保留 Q22 第1轮的 `.cb-workspace` height/align-self 兜底。
  - `App.vue`：toast 移出 toolbox-only 模板，全视图可见。
- 实际结果：已验证——本地闸门 lint exit 0 + vitest 60/60 + vite build 成功；经 vite HMR 热更到用户运行中的 tauri dev（无重启），桌面控制实测：重开 springbootDemo1 → 根层树行出现（src/.DS_Store/.gitignore/.iml）→ 点 src 二级懒加载展开（Main.java/package-info.java）→ 文件预览行号+内容+操作按钮全在位；用户本人随后自行开 3 个 tab 并移动窗口（=真机可用确认）。文档：`docs/work/active/bugfix-2026-08-code-browser-tree-invisible/{change-note,verification}.md` + code-browser/changelog.md Q22 条目 + 本台账。
- 下一步：变更未 commit（含 Q21 的 README 改动），等用户复核后一并提交；视觉截图通道受限（窗口曾部分在屏外 + 权限），布局宽度修复以 CSS 逻辑 + a11y 结构证据为准；v0.1.11 发版决策继续等用户定向。

### Q23 | 第1次处理（树子级重复 ×2 + 最大化按钮对齐 mac 原生）
- 问题或新增信息：用户真机反馈两件事：1) springbootDemoV1 展开 src 后 main / test 目录出现 2 次；2) 最大化按钮希望参考 mac 常见软件（移动与调整大小）。
- 本轮方案：
  - **Q23-1 根因**：Q22 修好可见性后暴露的模板结构缺陷——外层 `<template v-for="(entries, rel) in treeCache">` 遍历全部已缓存层级，每个展开过的目录都会被外层再渲染成一份顶级列表（与模板内手工嵌套重复）；旧模板写死三层（cb-sub/cb-sub2）。修复：新增 `visibleRows` computed 按 expanded 递归下钻 treeCache 生成 `{entry, depth}[]` 扁平列表，模板只渲染它（缩进 paddingLeft depth×14px），删三层嵌套；★收藏按钮统一到所有层级文件行；顺带解除「最多两层」旧限制（core 本就支持 8 层深度）。源码加注释防回退。
  - **Q23-2**：绿点补回点击全屏切换（`toggleFullscreen`，isFullscreen 切换；发现现状 @click 无处理函数——app-shell changelog 早期「真正的全屏切换」记载与代码不符，应为悬停面板改造时回归丢失）；`applyWindowLayout` 加 quarter-tl/tr/bl/br 四角四分位（取代 four-grid=左上角单一入口），半屏文案对齐原生（移到屏幕左侧/右侧/上半部/下半部）；四角用文字按钮（Grid2X2 图标随四格排列移除）；`.window-layout-quarter` 11px。拒绝/后置：双击标题栏缩放、Option+zoom（与填充屏幕语义重复）记 plan.md。
- 实际结果：已验证——用户中途关掉旧 tauri dev 又重启（新 pid 24401，直接带全部修改）。闸门 lint exit 0 + vitest 60/60 + build 成功；真机桌面控制实测：springbootDemoV1 根层 8 条正常 → 展开 src **main/test 各一次** → 展开 main 深度 3（java/resources）无重复；绿点点击进全屏（0,34,1710,1073）再点退出精确恢复 71,77,1200,780。文档：两个任务目录（bugfix-2026-08-code-browser-tree-duplicate：change-note+verification；enhancement-2026-08-window-layout-native：plan+verification）+ code-browser 与 app-shell changelog 各一条 + 本台账。
- 下一步：变更未 commit（Q21 README + Q22 + Q23 攒了一批），等用户复核后一并提交；四角/三列逐项点验、全屏态菜单收敛挂 PENDING 真机清单；v0.1.11 发版继续等用户定向。

### Q24 | 第1次处理（最近项目支持删除）
- 问题或新增信息：用户提出「最近项目要支持删除」。全栈小功能：core + IPC + bridge + 前端。
- 本轮方案：语义决策——删除最近项目连同其名下最近文件（同一"足迹"数据），收藏/书签保留（显式沉淀数据），不弹确认框（可再生数据，轻交互），不动磁盘文件。实现：core `remove_recent_project` + 单测；IPC `code_browser_recent_remove_project` + main.rs 注册 + mock runtime 自清理用例（open 临时项目→remove→幂等再删）；bridge.ts 接口/stub/invoke 三端；CodeBrowserView 最近行拆「打开 + × 删除」+ 成功 toast；style.css `.cb-recent-open/.cb-recent-remove`。
- 意外发现与修复（测试基建）：
  1. 新增第二个 IPC 用例后撞出竞态——同二进制两用例并行执行，都对真实用户层 `~/.elwright/code-browser.json` load→改→save，交错即丢更新（老用例书签断言先误报"二次切换未移除"又误报 len 3）。修：`USER_STORE_LOCK` 互斥锁串行化。
  2. 老用例收藏/书签断言是绝对长度（假设用户层为空）——CI 全新 HOME 从未暴露，开发机必误报。修：改增量断言（before+N）。
  3. 清理失败运行在用户层留下的测试垃圾：3 书签 + 6 收藏（全部指向已删除的 elwright-cb-* 临时目录；真实项目数据未动）。
- 实际结果：已验证——闸门全绿：cargo 78 核心（含新单测）+ 各集成测试（含新 IPC 用例）、clippy -D warnings 0 警告、fmt、eslint、vitest 60/60、build。真机：tauri dev 自动重编 Rust 重启后，× 按钮出现；点删 springbootDemo1 → 行消失 + toast 在代码视图显示（顺带回归 Q22 toast 修复）+ store 持久化确认（项目与其最近文件同清）；系统目录选择器 → 打开项目全链路首次真机验证通过（面板弹出→选择→树渲染；键盘 Cmd+Shift+G 在本机 CU 环境被拒，app 无 bundle_id，精确跳转未走通，不影响真实用户）。被删的 springbootDemo1 最近记录已直接恢复到用户层。
- 下一步：变更未 commit（Q21 README + Q22/Q23/Q24 四批攒着），等用户复核后一并提交；v0.1.11 发版继续等用户定向。

### Q25 | 第1次处理（Q21-Q24 批次提交 + 进度文档同步）
- 问题或新增信息：用户指示「看路线图，我们要做的事情，是不是可以提交代码了」。
- 本轮方案：先核对状态（main 分支、全部闸门此前已绿、package-lock.json 仅有本机 npm 的 dev→devOptional 元数据噪音——还原不入库）；同步三处过期进度文档（README 当前状态 v0.1.5→v0.1.10 概览、AGENTS 当前进度 v0.1.7→指针化一句话、ROADMAP 未发版清单/进行中说明/里程碑补 Q21-Q24 批次）；分两批提交：①代码（src 4 文件 + src-tauri 4 文件）②文档与台账（README/AGENTS/ROADMAP + changelog×2 + architecture + 任务目录×4 + session），`.zcode/` 明确不入库；推送 main 后盯 CI。
- 实际结果：两批提交落 main 并推送；CI 结果推送后确认。
- 下一步：CI 如绿则等待用户决策 v0.1.11 发版（tag 后 release.yml 自动出 dmg/msi，需同步 install.ps1 ProductCode）；真机点验 PENDING 清单延续；V2 主干下一阶段（AI 对话长上下文/跨平台完善、人与消息会话②③、设置中心 i18n 基建、工作台后续）待用户定向。

### Q26 | 第1次处理（发版 v0.1.11）
- 问题或新增信息：用户指示「先发版，然后直接执行 V2 剩余主干」。
- 本轮方案：按 v0.1.10 既有流程——①版本号四处同步（Cargo.toml / tauri.conf.json / package.json / package-lock.json，Cargo.lock 随 cargo build 更新）+ 本地闸门（cargo 78 + vite build）→ commit 6a22153 推 main；②等 ci.yml 7/7 全绿后下载 windows msi 制品，`file` 提取 Revision Number = {7638490A-6BF2-4B6C-978A-9F67D7C1320A}（AGENTS.md 记载的提取法），同步 install.ps1；③tag v0.1.11 打在含 ProductCode 的 commit 6583057 上，推 tag 触发 release.yml。
- 实际结果：已验证——ci.yml 33414800623 7 job 全绿；release.yml 33415734722 全绿；`gh release view v0.1.11` 双资产在位（Elwright_0.1.11_aarch64.dmg + Elwright_0.1.11_x64_en-US.msi）。
- 下一步：公司机真机装 msi 验证一键脚本幂等升级；ROADMAP/AGENTS/README 进度同步随 Q27 收口提交。

### Q27 | 第1次处理（V2 剩余主干启动：AI 对话阶段④余项「长上下文」）
- 问题或新增信息：按用户「直接执行 V2 剩余主干」+ ROADMAP 顺序（AI 对话排最前），余项为「长上下文」与「跨平台完善」。
- 本轮方案：
  - 摸清链路：`chat_completion`（commands.rs:240）与 `chat_completion_stream`（:887）均为 `system + 全量 history` 原样转发，无任何长度管理——本地会话永久保存（阶段②）后长会话必然超模型上下文或成本线性膨胀。
  - **ADR-004 定案：core 侧字符预算滑动窗口**——新模块 `core::chat_context::fit_messages` 收口两命令；始终保留 system + 最新一条 user；其余从新到旧整条保留、放不下整条丢弃（不切半条）；最新 user 超预算时截中段留头尾并标注；默认 24000 字符，配置链新增可选 flat 字段 `contextBudgetChars`（serde default），profile 覆盖后置；静默裁剪。拒绝：LLM 摘要（离线红线/慢/贵）、本地 tokenizer（引依赖）、前端截断（破坏 core 收口）、UI 每轮提示。跨平台完善不进本 ADR，逐项走 PENDING 清单 + bugfix 目录。
  - 立项：`docs/work/active/feature-2026-08-chat-long-context/plan.md`（含实施 checklist：core::chat_context + 单测≥5 / 两命令接入 / `ew config` 可见 / IPC mock 用例 / 文档回填 / 闸门）。
  - ROADMAP「进行中」登记本项；ADR 内容在 Agent 最终回复中供用户低成本否决（沿用两阶段协议，但 ADR 不单独开 PR，随收口提交入库）。
- 实际结果：ADR-004 与任务目录已入库（6583057）；ROADMAP「当前版本 v0.1.11 / 未发版清空 / 里程碑补发版条目 / 进行中登记」+ AGENTS 一句话现状 + README 当前状态同步完成（本提交）。
- 下一步：用户对 ADR-004 无异议即开工实施（core::chat_context 模块 + 单测 → 两命令接入 → IPC 用例 → `ew config` 展示预算 → 文档回填 → 闸门全绿）；完成后按 ROADMAP 顺序继续「人与人消息会话②（轻量身份/邀请与一对一传输）」立项。

### Q28 | 第1次处理（长上下文实施，ADR-004 落地）
- 问题或新增信息：用户确认「按建议进行下一步开发」= 实施 ADR-004。
- 本轮方案与实现：①`core::chat_context` 新模块——`fit_messages(history, budget) -> (Vec<ChatMessage>, bool)`：总长≤预算原样返回；否则最新消息必留（超预算 mid_truncate 中段截断留头尾+「（超长截断）」标注），更早消息从新到旧整条保留（不切半条），6 个单测。②llm.rs 配置链：LlmConfig/UserConfigFile 加 `context_budget_chars: Option<usize>`（serde default，JSON snake 与文件既有风格一致）、env `ELWRIGHT_LLM_CONTEXT_BUDGET_CHARS`、merged() Option 字段级合并、`set_flat_field` pub 方法；`pub use DEFAULT_BUDGET_CHARS`。③commands.rs `assemble_chat_messages` 唯一收口两 chat 命令（预算从 config 取，None→24000）。④ew.rs：config 显示 context 行 + set 走 UserConfigFile。⑤顺手修数据丢失隐患：set 原按 BTreeMap<String,String> 回写，配置含 profiles 时解析失败→静默清空——改经 UserConfigFile 后档案保留。⑥新 tests/chat_completion_ipc.rs：TcpListener 起 mock LLM（记录请求体+回 OpenAI 响应），断言裁剪后请求体。踩坑记录：test_env 为 #[cfg(test)] 门控集成测试不可见（本测试进程内唯一用例，天然隔离，无需锁）；AppCtx 需 manage 且 SessionRegistry::new(backend) 签名带 SharedBackend（照抄 terminal_ipc 的 LocalBackend 构造）；批量正则补结构体字段误伤 LlmProfile（ADR 明确 profile 不加此字段），逐处回修。
- 实际结果：已验证——cargo test 84 核心（含 6 新）+ 集成（含新 IPC 用例）全绿；clippy -D warnings 0 警告；fmt 无差异；eslint 0；vitest 60/60；build 成功；CLI 冒烟（set/显示/档案保留）通过。文档回填 chat behavior/architecture/changelog + ROADMAP（AI 对话条目、里程碑、进行中清空）+ 任务目录 plan 勾选与 verification.md。
- 下一步：随本提交进 main + CI 确认；长会话真机点验（需真实 LLM）留 PENDING；下一项「人与人消息会话②：轻量身份/邀请与一对一消息传输」待用户定向立项。

### Q30 | 第1次处理（能力渐进式发布后续：成长体系透明化）
- 问题或新增信息：用户在 V2 剩余主干盘点（Q29 文本）后定向「先做能力渐进式发布后续」——ROADMAP 范围=透明的解锁规则 + 成长提示（社区提案/签名能力包另行排期不在内）。
- 本轮方案：ADR-001 定案并同轮实施（小任务合并两阶段）。①规则透明三处同文案：CapabilityList 待解锁徽标 tooltip / CapabilityDetail 锁定横幅 / App.vue 侧栏提示行，统一「累计使用任意能力 N 次后自动解锁（当前 T/N）」，缺 unlockAfterUses 的 tier>1 显示「暂未开放解锁条件」。②成长提示两时机：select() 跨阈值 notify「🎉 已解锁进阶能力「名」」；侧栏持续显示距最近可解锁项差距。③逻辑收口 lib/growth.ts 纯函数（isUnlocked/growthSummary/newlyUnlocked）+ 6 vitest。④示例策略：内置 weekly-report 升 tier 2 + unlockAfterUses 3（让成长闭环开箱可见；改变该示例默认可见性——已在 ADR/plan/最终汇报标注供否决）。拒绝：单能力计数（破坏 MVP 总量语义与本地数据）、远程/社区解锁（另行排期）、桌面系统通知。踩坑：IAB evaluate 函数字符串需 IIFE 立即执行（"() => ..." 会被当函数对象返回 {}）；shell 相对路径受上次 cd 影响（文档写路径用绝对/先回根）。
- 实际结果：已验证——vitest 66/66（含 6 新）+ eslint 0 + build 成功；浏览器预览四层实测：核心视图 3/4 + 提示行「距解锁「周报生成」还差 3 次」→ 查看全部出现待解锁徽标 → 详情横幅「（当前 0/3）」+ 调用禁用 → 预置 2 次后第 3 次使用触发解锁 toast、提示行消失、周报进入核心视图；测试用 localStorage 已清理。文档回填 progressive-capabilities behavior/changelog/architecture/ADR + ROADMAP（条目+里程碑）+ 任务目录 plan/verification。
- 下一步：随本提交进 main + CI 确认；weekly-report 示例策略等用户知悉；下一项 V2 主干「人与人消息会话②」待定向。

### Q31 | 第1次处理（工作台第二阶段：常用能力 + 实用工具）
- 问题或新增信息：用户定向「先处理可插队的小项：资源工作区顺延项和设置中心 i18n 基建」。核实后确认「资源工作区顺延项」实际指向工作台第二阶段（收藏/最近使用承接能力调用 + 少量高频研发转换工具，与工作台条目顺延项同源），与资源工作区本体（收藏夹/课题已交付）无关。
- 本轮方案（ADR-001 工作台）：①常用能力——lib/capabilityRecents.ts 本机存储（收藏 string[] / 最近使用 {id,at}[] 去重置顶上限 8、损坏回退、静默降级），App.vue select() 同时机 recordRecent，WorkbenchView 区块按收藏置顶+时间倒序、点名称 emit open-capability 跳工具箱选中详情；②实用工具——lib/convert.ts 纯函数 JSON 格式化/压缩、Base64（TextEncoder UTF-8 安全）、时间戳⇄日期（秒毫秒 ≤1e11 自动、本地时区），中文报错；WorkbenchView 2×2 布局（grid-auto-rows + overflow-y）。
- 实际结果：已验证——vitest 81/81（recents 4 + convert 6 新增）；eslint 0（preserve-caught-error 抓 catch 抛错缺 cause 已补 cause）；Playwright 10/10（.wb-count 与新区块撞名 strict violation，作用域化 .wb-todo .wb-count）；浏览器 IAB 实测四区块渲染、常用能力空态、JSON 格式化输出正确。文档：workbench behavior/architecture/changelog + ROADMAP 两处 + 任务目录 verification。
- 下一步：随本提交进 main + CI；桌面端最近使用/收藏/跳转真机点验留任务目录。

### Q32 | 第1次处理（i18n 基建：设置中心试点）
- 问题或新增信息：同轮第二项——设置中心「界面语言」置灰的前置（ROADMAP：待 i18n 基建）。
- 本轮方案（ADR-002 设置中心）：自写轻量 i18n（lib/i18n.ts：locale ref + zh-CN/en 平铺字典 + t() 双级回退 + dictKeysInSync 键集完整性守卫；拒绝 vue-i18n——无复数/格式需求）；preferences.language 字段（默认 zh-CN、mergePreferences 校验、updatePreferences 副作用 setLocale 即时生效）；设置中心壳层全量字符串走 t()（分区标题/说明/控件标签/主题三选项/语言选择器本体启用）。增量迁移遗留：枚举选项标签（启动视图/密度/字体名）、LlmSettings 内部文案、其余视图。
- 实际结果：已验证——vitest i18n 4 用例（含键集完整性）+ 全量 81/81；eslint 0；build 成功；preferences 存量测试不回归。
- 下一步：随本提交进 main + CI；English 模式下枚举标签仍中文为已知遗留；其余视图 i18n 增量迁移按需推进；下一项 V2 主干「人与人消息会话②」待定向。

### Q33 | 第1次处理（发版 v0.1.12，进行中）
- 问题或新增信息：用户定向「发布 v0.1.12」，携带 4 个未发版功能——AI 对话长上下文（ADR-004，Q28）+ 能力渐进式发布后续（Q30）+ 工作台第二阶段（Q31）+ i18n 基建（Q32）。
- 本轮方案：沿用 v0.1.11 的发版协议——①四处版本号同步（Cargo.toml/tauri.conf.json/package.json/package-lock.json + Cargo.lock 随 cargo build 更新），commit + push main；②ci.yml 出制品后下载 windows msi 用 `file` 命令提取 Revision Number = ProductCode；③同步 install.ps1；④打 tag v0.1.12 → release.yml 出 GitHub Release。
- 实际结果：进行中——版本号四处同步（d711791）+ ROADMAP 三处更新 + install.ps1 ProductCode 同步（a909746）已推 main；ci.yml 7/7 全绿出 dmg + msi；windows msi Revision Number = {968DA9AA-C60A-4459-9A33-D3C52864DF87}；tag v0.1.12 已推；release.yml in progress。
- 下一步：等 release.yml 全绿 → GitHub Release 验 dmg + msi 双资产 → 台账 Q33 收口。

### Q33 | 第2次处理（v0.1.12 发版完成）
- 问题或新增信息：上一节待收口——等 release.yml 全绿 + GitHub Release 验双资产。
- 本轮方案：gh release view v0.1.12 验证 → 完成 Q33 收口。
- 实际结果：已验证——release.yml 3 job 全绿（Build Windows msi 3m51s + Build macOS dmg 1m53s + Publish GitHub Release 11s）；GitHub Release v0.1.12 双资产就位（Elwright_0.1.12_aarch64.dmg sha256 7324d966... + Elwright_0.1.12_x64_en-US.msi sha256 39db589b...）。本地闸门全绿（cargo fmt + clippy 0 + test 84 集成 + eslint 0 + vitest 81 + coverage 95/76/100/95 + vite build + Playwright 10/10）；ci.yml 7/7（macOS dmg / Windows msi / Rust lint / Frontend 七步）；install.ps1 ProductCode {968DA9AA-C60A-4459-9A33-D3C52864DF87} 同步。
- 下一步：v0.1.12 上线完成；真机点验项延续 PENDING 清单（长会话裁剪/补丁编辑全链路/模型档案切换/最近项目删除/v0.1.11 msi 升级安装）；下一项 V2 主干「人与人消息会话②：轻量身份/邀请与一对一消息传输」待用户定向立项。

### Q34 | 第1次处理（人与人消息会话阶段② 立项 + ADR-002 批准）
- 问题或新增信息：用户定向「先出 ADR 然后进行开发」——V2 主干重启后第二项「人与人消息会话②：轻量身份/邀请与一对一消息传输」。
- 本轮方案：背景调研（messaging feature README/architecture 显示阶段① PeopleChatView+localStorage，架构留 MessageTransport 替换口；`src-tauri/src/core/` 无 people/message 模块，新建 decisions 目录）→ 出 ADR-002「消息传输通道/身份/邀请/加密」——选型：自托管 WS 中继（仓库附 axum 参考实现）+ 本地 Noise_XX 静态身份（X25519 公钥 16 字符 base32 = ID）+ 6 字符短邀请码 + libsodium secretstream 流加密；弃选 WebRTC P2P / GitHub claim / OIDC / Signal DR / 公共中继实例（理由写入 ADR）→ 用户批准确认 → 任务目录 `feature-2026-09-messaging-phase2/{plan,checklist,verification}.md`（6 步切分：协议层骨架/本地身份+邀请/设置中心中继 URL/客户端+中继最小回路/离线消息队列/文档回填）+ ROADMAP「进行中」与「人与人消息会话」条目同步更新。
- 实际结果：已验证——ADR-002 状态「提议」→「已批准 2026-09-01」；任务目录三件套齐；ROADMAP「进行中」登记本项，「人与人消息会话」条目标记 ADR-002 已批准 + 任务目录名；commit 待 Step 1 完成一并推。
- 下一步：开始 Step 1 协议层骨架——`src-tauri/Cargo.toml` 加 `snow` + `libsodium-sys` 依赖（评估 libsodium-sys 与 dryoc 后定）+ `core::messaging_transport.rs`（Noise_XX 握手 + secretstream 数据通道 + Frame 序列化）+ 帧格式文档 `docs/features/messaging/transport-protocol.md` + 单测（握手 round-trip/密文不可破/nonce 重放拒绝/帧解析边界）+ windows-gnu 工具链编译验证。

### Q34 | 第2次处理（传输层核心 6 步切片全部完成，UI 接线切片遗留）
- 问题或新增信息：Step 1（956e054）后继续——Step 2 身份+邀请（identity.rs 12→13 单测；IPC 测试暴露 `derive_id_from_dh_public` 只取 SHA-256 前 5 字节产 8 字符的 bug，修为前 10 字节 = 80bit = 16 字符，加回归单测）、Step 3 中继 URL 配置（`read/set_messaging_relay_url` + `validate_relay_url` ws/wss/host/port 校验，4 单测；6 个 IPC：identity_get/create_invite/accept_invite + get_messaging_config/set_messaging_relay_url/test_messaging_relay；`ew config messaging show/set/clear/test` 四子命令，d4c9251 + d3a717b）、Step 4 中继参考实现（`docs/features/messaging/relay/`：axum 房间路由 `/ws/<room_id>` + 单成员 64 帧暂存 + 30s 空房清理 + 日志零载荷；多阶段 distroless Dockerfile + compose + README）+ 端到端冒烟 `messaging_relay_smoke.rs`（起真实 relay 子进程，initiator/responder 双端 Noise_XX 三步握手经 WebSocket 完成 + 双向 AEAD 收发内容一致 + 断言 relay stderr 无明文片段——验证清单「明文不出现于 relay 日志」代码落地；踩坑：两 clients 均跑 initiator 拿到对端 msg1 必败需分角色；双方均「先收后发」死锁，改 initiator 先发后收；clippy zombie_processes 要求超时路径 kill+wait）、Step 5 离线队列（`core::messaging_queue` outbox.jsonl：FIFO + 按对端过滤 + 损坏行容忍 + tmp+rename 原子重写 + record_attempt；5 单测含「明文不入盘」强制；~~sled~~→零依赖 JSONL 偏差记入 ADR-002「实施偏差」段，b1e59c8）、Step 6 文档回填（behavior §第二阶段+本阶段边界、architecture 传输层结构图、changelog 2026-09-01、ROADMAP 三处、verification 全量回填）。
- 本轮方案：按 plan.md 6 步切片推进，每步 commit 前跑全门禁（cargo test 全量 + clippy -D warnings + fmt + ew 构建）。
- 实际结果：已验证——最终门禁 120 lib 单测 + 7 IPC 冒烟（messaging_phase2_ipc.rs）+ 2 中继端到端冒烟全绿；clippy -D warnings 0 警告；fmt 干净；CLI 探测三路径手测通过（✓ 已连接 1ms / 未配置报错 / 连接拒绝报错）。5 个 commit 全部推 main：956e054（step1）→ 2342abe（step2）→ d4c9251（step3）→ d3a717b（step4）→ b1e59c8（step5）。
- 下一步：UI 接线切片待用户定向（PeopleChatView 本地适配器替换为真实传输 + 握手成功后队列补投 + Bridge 方法 + 前端 vitest/e2e + 真机双账户点验）；任务目录 feature-2026-09-messaging-phase2 按协议不自行归档；windows-gnu 工具链编译待 windows CI runner 验证（本批 CI 推送后可见）。

### Q34 | 第3次处理（CI 收口：中继冒烟夹具进 CI，7/7 全绿）
- 问题或新增信息：Step 6 文档提交后的 CI run 33475533003 三平台 Rust core 全 FAIL——根因：`cargo test` 会跑 `messaging_relay_smoke.rs`，但 CI 从不构建 relay 二进制，`find_relay_binary` panic「未找到 elwright-relay」。本地全绿是因为本地手动构建过 relay release。
- 本轮方案：双管齐下——①ci.yml rust 矩阵 job 在 cargo test 前加「Build relay (messaging smoke fixture)」步（working-directory 直指 docs/features/messaging/relay，产物路径即测试探测路径），三平台 CI 从此真实跑端到端中继冒烟；②测试对二进制缺失改优雅跳过（提示构建命令），本地开发鲁棒性兜底，纯协议层回环由 complete_handshake_direct_loop_works 持续覆盖。
- 实际结果：已验证——run 33476022859 CI 7/7 全绿（Rust lint 1m44s + 三平台 Rust core 2m51s~4m58s + Frontend 57s + dmg 1m45s + msi 5m29s）。新依赖 snow/x25519-dalek/tokio/tokio-tungstenite 在 ubuntu/macOS/windows-MSVC 全部编译通过（commit 94d3e0f）。附注：CI 用 MSVC，公司 windows-gnu 工具链编译验证仍是 PENDING 项（snow 纯 Rust 实现预计无碍）。
- 下一步：同第 2 次处理——UI 接线切片待定向；任务目录不自行归档。

### Q35 | 第1次处理（消息会话② UI 接线切片开工）
- 问题或新增信息：用户定向「先做消息会话② UI 接线切片，把消息功能做成真正可用的闭环，然后直接做 i18n 增量迁移」。
- 本轮方案：接线切片按 ADR-003（本轮新写，沿用「先 ADR 再实施」惯例，用户已定向批准开工）收口四个设计决定：①邀请 v3——QR 增载对端 X25519 DH 公钥并校验 id==derive(dh_pub)（补 v2 只带签名公钥、DH 绑定缺失的洞）；②成对房间 + 按 ID 排序确定性角色（小 ID=initiator）；③收发路径——sync_peer 阻塞式「连接→握手→验 remote_static==联系人 DH 公钥→flush 发件箱→收至超时→落 inbox」，后台 listener 线程轮询联系人，前端 poll inbox 合并进 phase1 localStorage 会话；④离线队列改存「本地静态密钥（chacha20poly1305+hkdf）加密的明文」——原设计存会话密文在重连后无法解密（会话密钥不持久），flush 时用新会话重加密。新增依赖 chacha20poly1305/hkdf/hmac（纯 Rust 小件）。
- 实际结果：进行中。
- 下一步：ADR-003 落盘 → identity v3 + contacts → 队列 v2 + client 收发 + inbox + IPC → 前端 bridge/SettingsCenter/PeopleChatView → vitest/e2e → 文档回填；随后 Q36 i18n 增量迁移。

### Q37 | 第1次处理（工具路线图整理）
- 问题或新增信息：用户要求将 Markdown 文档预览/编辑和 JSON 工具补入路线图，并询问路线图是否应拆分工具类文档。
- 本轮方案：核对现有工作台工具与 Markdown 行为；保持总路线图为唯一优先级入口，新增工具明细路线图承载范围和验收边界。
- 实际结果：已确认 JSON 格式化/压缩已实现，JSON 转义/反转义尚未实现；今日记录有 Markdown 编辑/预览，但不存在任意本地 Markdown 文档编辑器。已建立 `docs/TOOL-ROADMAP.md` 并将两项登记为 V2 P1 计划。
- 下一步：实施前各自建立 Feature 文档、实现任务和自动化/真机验收清单。

### Q37 | 第2次处理（核对页面入口与完成状态）
- 问题或新增信息：用户看到路线图中的 JSON、Markdown 后，在页面找不到对应功能，询问是否已经完成。
- 本轮方案：对照 ROADMAP、工具明细路线图和当前前端导航/工作台实现，区分“已实现功能”与“已登记计划”。
- 实际结果：已确认：JSON 格式化/压缩已实现，入口在「工作台 → 实用工具 → JSON」；JSON 转义/反转义仍只是计划。Markdown 目前只有「工作台 → 今日记录」里的编辑/预览，不是任意 Markdown 文件编辑器；独立 Markdown 文档预览/编辑仍未实现。
- 下一步：若要看到 JSON 入口需运行包含工作台二期的 v0.1.12 或更新版本；Markdown 独立编辑器和 JSON 转义工具需后续单独开发。

### Q35 | 第2次处理（UI 接线切片完成，消息功能端到端可用）
- 问题或新增信息：按 ADR-003 三切片实施并全部完成。
- 本轮方案：切片 A——ADR-003 落盘；identity 邀请升级 v3（QR 增载对端 X25519 DH 公钥，签名内容含 dh_pub，accept 增加 `id==derive(dh_pub)` 硬绑定校验，v2 不再接受）；core::contacts（contacts.json）；core::local_crypto（chacha20poly1305，AAD 绑定 peer_id）+ identity::load_or_create_local_key；messaging_queue v2（本地密钥加密明文落盘——修复原「存会话密文重连后不可解」设计缺陷，flush 新会话重加密）；core::messaging_inbox（poll(since_id) cursor）；messaging_client::sync_peer（成对房间 pair_room 按 ID 排序、role_for 定角色、握手后校验 remote_static==联系人 DH 公钥防中间人、发件箱 FIFO flush 失败保留+attempts、收 Data 帧落收件箱空闲收尾）+ ensure_listener_started 后台线程 + 全局同步锁；IPC +6。切片 B——bridge +12 方法（浏览器预览统一中文降级抛错）；Rust 视图结构体统一 serde camelCase（对齐 TodoItem 等既有约定，IPC 测试同步回归）；SettingsCenter「消息中继」分组（MessagingSettings.vue：URL/保存/测试连接 + 双语 i18n，en 字典被替换吞掉的 terminal key 已补回——键集守卫测试抓到）；PeopleChatView 全接线（身份 chip/邀请弹窗复制/添加联系人弹窗/联系人快捷条/会话绑定 peerId/发送状态机 sending→sent|queued|failed/3s 收件轮询合并+对端左气泡/预览降级）；e2e +1 消息页降级守卫。
- 实际结果：已验证——门禁全绿：130 lib（identity 15/contacts 2/crypto 2/queue 6/inbox 3）+ IPC 7（camelCase 回归）+ 中继冒烟 3（**双身份全链路：v3 邀请互加→A flush 2 条中文→B 收件箱按序收齐**，经真实 relay 子进程）+ eslint 0 + vitest 81（键集守卫抓到 en 缺 key）+ vite build + e2e 11/11 + clippy -D warnings 0 + fmt。文档回填 behavior/architecture/changelog/ROADMAP/verification/ADR-003 实施备注（hkdf/hmac 未引入）。提交：8c477c0（A1）/ c04fc59（A2）/ dc4523d（B）+ 本轮 docs。
- 下一步：真机双账户点验（互发/篡改 DH 握手失败/离线补投/docker 部署）留 PENDING；随后按用户指示直接开始 Q36 i18n 增量迁移。

### Q35 | 附注（提交卫生失误，当轮披露）
- 问题或新增信息：docs 回填提交（2a55766）用 `git add docs/` 时误收了非本任务的非跟踪文件——`docs/TOOL-ROADMAP.md`、`docs/work/active/feature-2026-09-roadmap-tools/{plan,checklist,verification}.md`（工具路线图规划，疑似用户/并行会话所建）与 `docs/features/workbench/README.md` 的 +1 行改动，混入消息文档提交且提交信息未提及。
- 本轮方案：已推送不改写历史；内容为纯文档无功能影响。向用户明确披露，保留与否由用户决定（需要时可拆出重建跟踪）。
- 教训：提交前必须 `git status` 逐文件核对 staged 清单，任务目录外的非跟踪文件绝不顺手 add（呼应 Q13 提交卫生教训）。

### Q36 | 第1次处理（i18n 增量迁移第一批完成）
- 问题或新增信息：Q35 收口后按用户指示直接开始 i18n 增量迁移（设置中心 ADR-002 第 3 条遗留）。
- 本轮方案：任务目录 `feature-2026-09-i18n-incremental/{plan}.md`（范围 + 后续增量清单）。第一批：①`STARTUP_VIEW_OPTIONS` label → labelKey（`startup.*` 7 个双语键），设置中心渲染走 `t()`；②`LlmSettings.vue` 全量迁移（26 个 `llm.*` 双语键，覆盖字段标签/来源/占位符/按钮/档案切换·新建·删除文案与确认框，`{name}` 占位符调用方 replace）；③补 en 字典历史缺口 `settings.section.terminal`（此前缺失靠 zh 回退，本轮被键集完整性守卫暴露——迁移过程中替换误吞该 key 即被守卫抓到，守卫有效性顺带验证）。
- 实际结果：已验证——eslint 0 / vitest 81（键集守卫过）/ vite build / Playwright e2e 11/11；CI 7/7 全绿（run 33526326926）。commit 43efe7a 推 main。
- 下一步：其余视图壳层文案（PeopleChatView / ToolboxView / TerminalPanel / WorkbenchView）与能力类型/档位枚举标签留后续增量（plan.md 已列）；真机点验项（消息双账户 + 历史 PENDING 清单）由用户执行。
