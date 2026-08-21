# Elwright 架构方案

> 版本：v0.2（含阶段1落地）｜ 日期：2026-08-21 ｜ 状态：阶段 1（Rust 核心 + CLI 壳已跑通）

## 1. 背景与动机

### 1.1 痛点
积累的 skill（工作流能力）被锁死在 agent 运行时里。没有 LLM 或 agent 不可用，连本可独立运行的脚本都够不着。

两个真实场景：
- **场景 A（空窗期）**：AI coding 固定积分，超额需申请（≈1 天审批）。审批期间客户端在但 LLM 断供 → skill 全废。
- **场景 B（全离线）**：出差客户现场关门开发（无网无 agent），或现场实施人员无 agent / 无针对现场的 agent → skill 用不了。

### 1.2 使命（开源普惠）
让接触不到 AI、或没钱开通 LLM 的人，也能享受大模型红利，提高效率，把时间还给自己——多陪家人、提升自己、享受人生。

### 1.3 设计哲学（pi-mono 极简）
- **只放自己用到的东西**（参考 pi-mono 的 read/write/edit/bash 四个）。
- **LLM 是增强插件，不是地基**：工具自身独立存在，离网也能跑。
- **离网覆盖 70% 日常操作流**：操作类/机械类流程离网可用；思考类（评审/优化）离线降级为 SOP。

## 2. 产品定位

- **个人工作流工具箱**，双形态：**CLI + 桌面应用**，LLM-independent core。
- **名称**：`Elwright`
  - `El` = 超人 Kryptonian 姓氏后缀，意为星 / 光 / 家；
  - `wright` = 工匠 / maker；
  - 合意：「超人家的造工具的人」——普通人（无 AI）也能锻造自己的超能力。
- **命令**：CLI `ew`（或 `elwright`）。
- **Tagline**：普通人也能用上的大模型红利 · LLM 是增强不是地基。

## 3. 核心架构：一个核心，两个壳

- **共享核心（Rust lib）**：能力注册表加载、脚本执行引擎、LLM 客户端、离线降级逻辑——只写一遍，CLI 与桌面共用。
- **CLI 壳**：`ew ls` 列表 / `ew run <id>` 跑脚本型 / `ew view <id>` 看知识型 / `ew invoke <id>` 调技能型。现场/SSH/低配机首选，也是开源普惠主入口。
- **桌面壳**：Tauri 2 + Vue 3 + Vite，给要界面的用户。
- **跨平台**：Windows（优先）+ macOS；Linux 后续。

```mermaid
graph TD
    CORE[共享核心 Rust lib\n注册表+执行引擎+LLM客户端+降级] --> CLI[CLI 壳 ew\n终端/现场/低配]
    CORE --> GUI[Tauri+Vue 桌面壳\nWindows/macOS]
    CORE -->|script| S[spawn 进程 离网可用]
    CORE -->|knowledge| K[渲染 md 离网可用]
    CORE -->|skill| L[OpenAI兼容API\n离网降级SOP]
```

## 4. 能力模型（三类）

| 类型 | 离网可用 | 处理方式 |
|---|---|---|
| script | ✅ 直接跑 | 核心 spawn 进程 |
| knowledge | ⚠️ 当文档翻 | 核心渲染 md |
| skill | ❌ 离线降级 SOP | 核心调 LLM；不可达时展示 SOP 文档 |

注册表条目示例：

```json
{
  "id": "doc-keyword-search",
  "name": "文档关键字搜索",
  "type": "script",
  "category": "知识库",
  "entry": "resources/tools/doc-keyword-search/search_doc.py",
  "offline": true
}
```

```json
{
  "id": "tech-grill",
  "type": "skill",
  "name": "八层拷问",
  "prompt": "八层提问模板...",
  "offline": false,
  "degradeDoc": "resources/docs/tech-grill-sop.md"
}
```

**离线降级规则**：技能型被调用时检测 LLM 可达性；不可达（场景 A 空窗 / 场景 B 离线）则展示 `degradeDoc`，不报错。

## 5. LLM 接入（OpenAI Compatible）

- 设置项：`base_url` + `api_key` + `model`，用户自配。
- **默认指向本地模型**（如 `http://localhost:11434/v1`），场景 A 空窗期可切本地模型复活技能型。
- 场景 B 无模型 → 技能型降级 SOP。
- 兼容任意 OpenAI 兼容端点（云端 / 本地 Ollama / llama.cpp 等）。

## 6. 种子能力清单（初版）

> 原则：只列常用做 seed；**具体 skill 文件暂不改**（见第 8 节）。

**脚本型（离网 ✅）**
文档关键字搜索、Excel转md、Word转md、产品包需求→功能清单、修改说明→功能清单、能力标签自动生成、jar反编译、一键部署、知识库查孤儿/树/校验(3个)、桌面快捷方式、VSCode扩展管理。

**知识型（离网可看 ⚠️，挑高频）**
WDS定制知识+踩坑、ModbusTCP学习笔记、JVM崩溃日志排查、HTTP接口鉴权方案对比、（其余 knowledge-doc 笔记后续批量导入）。

**技能型（需 LLM，离线降级）**
八层拷问、提示词优化、知识总结分享、工作总结、接口文档格式化（待确认脚本/技能）、需求选点。

**暂不入册（环境/个人类）**
`pi/`（获取LLM手段）、`login-wechat/`（隐私）、`process-analysis/`（未开发）、toolbox 根老住户。

## 7. 开源与分发

- **License**：MIT 或 Apache-2.0（见 LICENSE）。
- **默认零 LLM 可用**：脚本型 + 知识型 = 免费层（任何人装完即用）；技能型 = 接了模型才解锁的增强层。
- **小白友好 LLM 配置指引**：README 给本地 Ollama / 免费兼容端点填法。
- **分发**：Tauri 打 `msi`(Win) / `dmg`(macOS)；脚本作为 `resources/` 打包进 app，自带不依赖外部路径。
- **README 面向非技术用户**，降低门槛。

## 8. 与现有资产的关系

- `toolbox/` 下现有脚本（doc-keyword-search 等）→ 变为 script 型能力，打包进 resources。
- `D:\knowledge-doc` 笔记 → knowledge 型能力（viewer）。
- 旧的 `toolbox/kit/方案.md` → 保留为 v0（PowerShell）取消纪要，不删。
- `pi/`、`login-wechat/` 等 → 暂不入册。
- **本期不移动/改写任何现有 skill 文件**，仅将引用登记进 `capabilities.json`，待阶段 1 再决定脚本如何导入 resources。

## 9. 技术栈详解（已锁定选型）

> 2026-08-21 拍板：以下选型全部确认，阶段 1 起按此落地。

### 9.1 选型总览（技术 → 解决什么问题 → 为什么选它）

| 技术 | 解决什么问题 | 选型核心理由 | 备选（未选理由） |
|---|---|---|---|
| **Rust**（核心语言） | 一套代码同时喂 CLI 壳 + 桌面壳；跨平台、单二进制、体积小、离网启动快 | Tauri 后端就是 Rust → 核心与桌面**同一语言、零 FFI 胶水** | Go（接 Tauri 要 sidecar/FFI，多一层）；Node/Electron（体积大、违背 pi-mono 极简 & 离网 70%）；Python（桌面壳不如 Tauri 轻） |
| **Tauri 2**（桌面壳，阶段 3） | 要界面但不打包整个 Chromium | 系统 WebView，单二进制几 MB，天然复用 Rust 核心 | Electron（重）；Flutter（Dart，复用不了 Rust 核心）；原生 WinUI/SwiftUI（不跨平台） |
| **Vue 3 + Vite**（桌面 UI，阶段 3） | 桌面壳前端渲染（列表/详情/invoke） | 开发者最熟（AstrBot dashboard 即 Vue3+Vite，可复用范式）；轻量、Tauri 官方模板支持 | React（需重拾习惯）；Svelte（需从零学，Tauri 已足够小，省体积无意义） |
| **OpenAI Compatible HTTP**（LLM 客户端，阶段 2） | 技能型调 LLM，端点各异（云端/Ollama/llama.cpp） | 一套 `/v1/chat/completions` 适配所有兼容端点，用户自填 base_url+key+model，不绑厂商 | 仅绑单厂商 SDK（锁定风险） |
| **capabilities.json**（注册表） | 能力清单如何被核心加载 | 静态 JSON，人可读、进 git 版本可控，阶段 0 已建种子 | SQLite（过度工程）；TOML（JSON 已用，无换必要） |
| **spawn 子进程**（脚本执行） | script 型是现成 .py/.ps1，核心不能 import | `std::process::Command` 原样执行，抓 stdout/exit code，**离网直接跑** | 嵌入解释器（重、环境耦合） |
| **tokio + serde + clap**（Rust 生态） | 异步运行时 / JSON 解析 / CLI 参数 | Rust 事实标准，无争议 | — |

### 9.2 三项关键决策（已定）

1. **LLM 客户端：自写 `reqwest` thin client**（不引 `async-openai` crate）。理由：只用到 `chat/completions` 一个端点，自写可控、零额外结构耦合，符合 pi-mono 极简。后期若需流式/工具调用再评估。
2. **注册表：v1 纯静态 JSON**（不做目录自动扫描）。理由：种子清单已就绪，静态可读可版本化；自动发现留作后续增强，不提前加复杂度。
3. **桌面 UI：Vue 3 + Vite**。理由见 9.1；开发者流利度优先于框架特性。

### 9.3 目录结构（V1）

```
Elwright/
├── src-tauri/           # Rust 核心 + Tauri 壳
│   ├── src/
│   │   ├── core/        # registry.rs / executor.rs / llm.rs / degrade.rs
│   │   └── main.rs      # Tauri 入口
│   └── Cargo.toml
├── src/                 # Vue3 桌面界面（阶段3）
├── capabilities.json    # 能力注册表（种子清单）
├── resources/
│   ├── tools/           # 脚本型能力的 .py/.ps1（阶段1导入）
│   └── docs/            # 知识型/SOP 的 .md
├── README.md
└── LICENSE
```

CLI 壳复用同一 `src-tauri/src/core`，单独编译为 `ew` 二进制（`cargo build --bin ew`）。

## 10. 路线图

- **阶段 0**：建仓 + 本架构方案（本次）。
- **阶段 1**：Rust 核心（registry + executor + 脚本型跑通）+ CLI 壳，验证「离网 70%」。
- **阶段 2**：LLM 客户端 + 技能型 invoke + 降级。
- **阶段 3**：Tauri+Vue 桌面壳。
- **阶段 4**：跨平台打包（msi/dmg）+ 开源发布（MIT + README + 小白指引）。

## 11. 待确认 / 后续
- 具体 skill 文件本期不改（本期只规划，见第 8 节）。
- 名称把手核查：Elwright 在 GitHub / crates.io / npm 均无占用，域名未检索到（待发布时 registrar 确认）。✅
- `api-doc-formatter` 归类（脚本/技能）待定。
- Linux 支持列为后续。
- **技术栈三项决策已锁定**（2026-08-21，见 §9.2）：LLM 客户端自写 reqwest thin client；注册表 v1 纯静态 JSON；桌面 UI 用 Vue 3 + Vite。

## 12. 阶段进度与环境决策记录（2026-08-21）

### 12.1 阶段 1 落地情况（公司机器，已完成）
- Rust 工具链装好：`rustc`/`cargo` 1.98.0，含 `stable-x86_64-pc-windows-msvc` + `stable-x86_64-pc-windows-gnu` 两个 toolchain。
- 公司机器**无 MSVC 链接器**（没装 VS）→ 用 **GNU 工具链 + MinGW-w64（`D:\mingw64`，winlibs gcc-16.2.0）** 编译。
- 编译命令：`cargo +stable-x86_64-pc-windows-gnu build`，产物 `src-tauri/target/debug/ew.exe`（运行时需 `D:\mingw64\bin` 在 PATH）。
- cargo 代理/源在**用户级** `C:\Users\44895\.cargo\config.toml`（rsproxy 源 + 公司代理，不进仓库）。
- 代码修复并 push（commit `c23a123`）：`registry.rs` 解析顶层 `{capabilities:[...]}`（原误当数组）+ `Capability` 补 `doc` 字段（知识型）+ View 优先读 `doc`；加 `elwright-guide` 知识条目。
- 验证：`ew ls` 列 23 项能力；`ew view elwright-guide` 正确读出中文 md 全文（中文路径 OK）。**阶段 1 在公司机器完整跑通**。

### 12.2 工具链决策结论（关键，回家接着看）
- **不装 MSVC Build Tools 的影响边界**：卡住的只有「Tauri 桌面二进制编不出来/跑不起来」这一处（Tauri 后端 `wry` → WebView2 Loader 链接按 MSVC `.lib` 设计，GNU+MinGW 的 `ld` 读不通）。以下**完全不受影响**：CLI 壳 `ew`、阶段 2 LLM 客户端（纯 Rust `reqwest`+`tokio`）、所有共享核心逻辑、Vue 3 前端开发（Node/npm 跑 Vite，浏览器预览）。
- **桌面 app 是交付项 → 最终必须有 MSVC**（或硬刚 GNU 非官方路径，不推荐做交付）。但**不是现在装**：推迟到真正 `tauri build` 出桌面 `.exe` 时再装。
- MSVC 实际落地 ~4.5–5 GB（非 6–7）：MSVC v143 工具 ~1.7G + Win11 SDK ~1.3G + 共享组件 ~1–2G。**安装时指定到 D 盘**（如 `D:\VSBuildTools`），别吃 C 盘系统空间。
- 本机磁盘现状（2026-08-21）：C 剩 14.6 GB / D 剩 10.7 GB，均偏紧。可用 `cargo clean` 清 `target/` 缓存腾空间。WebView2 Runtime 已预装（151.0.4129.101），桌面壳渲染不缺。

### 12.3 接下来（回家可立即做的）
- **阶段 2（无新工具链）**：`cargo add reqwest tokio`，把 `llm.rs` 占位换成真实 OpenAI 兼容 `/v1/chat/completions` 调用；`ew invoke <id>` 调技能型；LLM 不可达时降级 `degradeDoc`。
- 测试阶段 2 可选本地 Ollama（`localhost:11434/v1`），或直接填云端 OpenAI 兼容端点 + key。
- 阶段 3 桌面壳：先开发 Vue 前端（npm/vite，无需 Rust 工具链），Tauri 具体编译等 MSVC 到位再 `tauri build`。

### 12.4 阶段 2 落地情况（家里 macOS，2026-08-21 已完成）
- 家里 Mac 装 rustup 工具链（stable 1.98.0，minimal profile，用户级安装）。
- `llm.rs`：占位替换为真实 OpenAI 兼容客户端。实现取舍：用 **reqwest blocking**（`--features blocking,json`），不显式引 tokio——CLI 是同步程序，免 async 污染调用链；理由与后果记录在 `docs/features/llm-invoke/decisions/ADR-001-blocking-reqwest.md`。60s 超时防挂死。
- `ew invoke`：skill 类型限定；有 `ELWRIGHT_LLM_BASE_URL` 则调 LLM（能力 prompt 作 system、命令行参数作 user），失败/未配置/超时/解析失败均降级 `degradeDoc`，不报错退出。
- 验证（见 `docs/work/active/feature-2026-08-stage2-llm-invoke/verification.md`）：`cargo test` 3 例 URL 拼接全过；降级路径（未配置 / 端点不可达）、成功路径（本地 mock OpenAI 兼容端点）、类型守卫、`ew ls` 回归冒烟全部通过。真实云端端点 / Ollama 待用户复验。
- 新增种子 SOP `resources/docs/tech-grill-sop.md`（此前技能型 degradeDoc 均指向不存在的文件，降级只报"文件不存在"）。
- Agent 开发维护方案（`resources/docs/AI_CODE_AGENT_MAINTENANCE.md`）首次实际应用：本阶段产出 `docs/features/llm-invoke/`（README/behavior/architecture/changelog/ADR-001）+ `docs/work/active/feature-2026-08-stage2-llm-invoke/`（plan/checklist/verification/STATUS）。
- **接下来**：阶段 3 桌面壳（先 Vue 前端，npm/vite；Tauri 编译等 Windows 机器 MSVC 就绪）；其余 5 个技能型的 SOP 文档批量导入。

### 12.5 阶段 3 前端落地情况（家里 macOS，2026-08-21 浏览器预览版完成）
- **前端先行路径执行**（§12.3 既定决策）：`src/` 作为自包含 Vite 项目（package.json 在 src/ 内，保持本方案 §9.3 目录规划），Vue 3 + Vite 8 + TS，依赖仅 `vue` + `marked`（无组件库，pi-mono 极简）。
- **Bridge 抽象层**（`src/lib/bridge.ts`）：UI 只依赖 `Bridge` 接口（list/view/run/invoke）；浏览器适配器已实现，Tauri 适配器挂接点留在 `createBridge()`（探测 `window.__TAURI_INTERNALS__`），阶段 3b 接 IPC 时 UI 零改动。
- **Dev 只读 API**（vite 插件）：`/api/capabilities` 读真实注册表、`/api/file?path=` 读 resources/ 文件（前缀校验防穿越）——预览即真实数据，非 mock。
- **UI**（全中文，三栏）：侧栏类型筛选 + 搜索；能力列表（类型徽标/分类/⚡离网标记）；分型详情——script 传参运行（预览态给等价 CLI 命令）、knowledge 渲染 Markdown、skill 调用（预览态固定走降级 SOP，正好预演降级 UI）。
- 验证（见 `docs/work/active/feature-2026-08-stage3-desktop-ui/verification.md`）：`npm run build` 成功（产物 111KB）；curl 冒烟（24 项能力 / SOP 读取 / 穿越拦截 403）；浏览器 DOM 快照逐项验证（筛选、技能降级渲染、知识文档渲染、脚本面板）。
- 后续由阶段 3b 完成 Tauri 壳接入，见 §12.6。

### 12.6 阶段 3b Tauri 壳落地情况（家里 macOS，2026-08-21 已完成）
- 新增 `src-tauri/src/main.rs` 与 Tauri 2 配置：4 个 IPC 命令 `list_capabilities` / `view_doc` / `run_script` / `invoke_skill` 复用共享 core；脚本和 blocking LLM 请求在后台线程执行。
- `Capability` 以 JSON 字段名序列化给前端；`executor` 提供 stdout/stderr 捕获变体；`core/invoke.rs` 让 CLI 与桌面壳共用 LLM 失败/未配置时的 SOP 降级逻辑。
- `src/lib/bridge.ts` 用 `@tauri-apps/api` 的动态 `invoke()` 实现 Tauri 适配器，探测到 `window.__TAURI_INTERNALS__` 自动切换，现有 Vue 组件零数据访问改造，侧栏显示当前模式。
- 加入 `tauri.conf.json`、官方工具生成的 app 图标、`build.rs` 和 desktop 默认二进制；`cargo test`（6 项）、`npm run build`、`tauri build --debug --bundles app` 全部通过，产物为 `src-tauri/target/debug/bundle/macos/Elwright.app`。
- 阶段 4：将 `capabilities.json` 与 `resources/` 纳入 bundle 后的路径解析，制作/签名正式 msi 与 dmg，再发布；Windows 仍需 §12.2 所述 MSVC 工具链。

### 12.7 阶段 4/5 落地与整体收尾（2026-08-22）

- **阶段 4（macOS 侧完成）**：资源根三段式解析（`ELWRIGHT_ROOT` env > cwd 上溯 > bundle 资源目录/exe 相邻）——core 保持零壳依赖；`tauri.conf.json` bundle.resources 把 capabilities.json 与 resources/ 打进 `Contents/Resources`（实测无 `_up_` 逃逸）；产出 `Elwright_0.1.0_aarch64.dmg`（8.2MB，未签名），挂载核验资源落位 + 卷内启动冒烟通过。坑：dmg 的 AppleScript 美化偶发 Finder 超时（-1712），重试即过。
- **阶段 5 第一批（3 个通用脚本）**：doc-keyword-search / xlsx-to-md / docx-to-md，纯 Python stdlib 零依赖，`ew run` 端到端 + 全错误路径验证（样本为手工构造的最小合法 xlsx/docx）。其余 10 个 entry 等公司机原版导入。
- **整体审计后的补漏**：① executor python 解释器探测（python3 → python → py，OnceLock 缓存）——修 Windows 无 python3 别名问题；② LLM 配置回退链落地（env > 注册表 `$meta.llmDefault`）——本方案 §5「默认指向本地模型」承诺兑现，装 Ollama 的用户零配置解锁技能型；③ CI 新增 macOS dmg 制品 job（上传 artifact）。
- **待外部条件**：Windows msi（公司机 MSVC，§12.2/设计文档 §3）；GitHub Release 发布（版本三处同步 + 产物上传）；公司机原版脚本与知识文档导入。
