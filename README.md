# Elwright

> 让更多人用上 AI，改变工作方式，把时间还给自己。

Elwright 是一个面向个人的 AI 增强型工作流工具。它把日常工作中反复使用的脚本、知识和 AI 技能，整理到一个统一入口，帮助你把经验沉淀下来，把重复工作交给工具完成，从而更高效地处理工作。

Elwright 这个名字，来自超人带来的一个联想：每个人都可能拥有尚未被充分发挥的能力。我们希望把 AI 的力量和实用工具结合起来，把那些原本遥远、复杂的能力，变成普通人也能在日常工作中使用的东西。

## 我们为什么做 Elwright

AI 带来的效率提升不应该只属于懂技术、用得起复杂工具的人。Elwright 希望降低使用门槛，让更多人都能逐步体验 AI 对工作方式的改变：从一个小脚本开始，逐渐积累自己的知识和工作方法，再在需要时接入大模型获得更强的帮助。

Elwright 的核心原则是：**LLM 是增强，不是地基。** 没有大模型时，脚本和知识仍然可以正常使用；需要 AI 的技能则自动降级为可照着执行的 SOP，不让工具因为网络、配置或服务不可用而完全失效。

## 产品边界

Elwright 的主线是帮助个人，尤其是研发与技术工作者，完成一个可复用的工作闭环：

```text
沉淀能力 → 调用能力 → 记录工作上下文 → AI 辅助理解与组织 → 用户确认执行 → 产出可复用的知识、流程或工具
```

是否进入主干的判断标准只有一个：功能必须明确缩短这条闭环中的某一步，或让更多人能用上它。不能形成闭环的通用效率功能、娱乐化功能和平台替代型功能，不进入面向所有用户的主干路线图。

- **坚持做**：能力管理与导入、CLI/终端、确认式 AI 协作、与能力相关的工作记录、可复用的知识与工程表达。
- **克制做**：Todo、书签、脑图和工程图只服务于工作能力的组织、执行和复用，不发展成通用笔记、项目管理、白板或绘图平台。
- **不进主干**：全自动 Agent、隐式执行、默认采集用户数据、游戏化桌宠、通用效率软件替代、完整 SSH/服务器管理客户端等。

个人实验可以在独立分支中验证，但不得降低主干的离线可用、可控、隐私和简单性标准；验证出明确且通用的工作流价值后，再单独立项评估是否合入。

## 它能做什么

一个**个人工作流工具箱**，把你的定制化能力（脚本 / 知识 / 技能）统一管理、统一使用。内置注册表只保留少量真实示例，方便你了解能力格式；你的个人能力通过导入加入用户层，不会改动软件自带内容。

- **脚本型**：离网直接跑，例如内置的文本统计示例
- **知识型**：离网可看，例如能力类型说明和个人知识笔记
- **技能型**：接入 LLM 后由 AI 协助完成；没有 LLM 时降级成 SOP 文档，不报错
- **导入与分享**：把个人能力导入用户层，也可以用 `ew export <id>` 打包成单文件，交给别人用 `ew import` 导入

工具自身**不依赖任何 LLM 或 agent 运行时**。LLM 是可选的增强能力。

## 双形态

- **CLI**：`ew`（终端 / 现场 / 低配机 / 开源普惠主入口）
- **桌面应用**：Tauri + Vue（给想要界面的用户）

> 跨平台：Windows（优先）+ macOS；Linux 后续。

## 安装（普通用户）

**一键安装（推荐）**：

```bash
# macOS（Apple 芯片 / Intel 都行）
curl -fsSL https://raw.githubusercontent.com/X-85/Elwright/main/install.sh | bash
```

```powershell
# Windows（PowerShell）
irm https://raw.githubusercontent.com/X-85/Elwright/main/install.ps1 | iex
```

脚本会从 GitHub Release 拉最新版本的安装包直接装好（mac 走 `ditto` 进 `/Applications`，Windows 走 `msiexec /quiet`）。默认装最新版；想指定版本：

```bash
# macOS
curl -fsSL .../install.sh | ELWRIGHT_VERSION=v0.1.3 bash
# Windows
$env:ELWRIGHT_VERSION='v0.1.3'; irm .../install.ps1 | iex
```

完整说明（环境变量、故障排查、跟手动安装的关系）见 [docs/install/one-line-install.md](docs/install/one-line-install.md)。

**手动安装**：到 **[Releases](https://github.com/X-85/Elwright/releases)** 页下载：

- **macOS**：下载 `.dmg`，打开后把 Elwright 拖入「应用程序」。首次打开若被拦截：右键应用图标 →「打开」→ 再点「打开」（安装包未签名，属正常提示，只需过一次）。
- **Windows**：下载 `.msi` 双击安装。SmartScreen 蓝色提示时点「更多信息」→「仍要运行」。

装完即用：可以先打开桌面应用或运行内置示例。**脚本型与知识型能力离网可用**（脚本型需机器上有 Python 3）；个人能力可以通过桌面端导入或 `ew import` 加入用户层。想让技能型接入大模型（可选），见 [LLM 配置指引](docs/release/llm-setup-guide.md)。支持本地 Ollama（免费、数据不出机器）或任意 OpenAI 兼容云端端点；不配置时技能型自动降级为可照做的 SOP 文档。

更新 = 下载新版安装包覆盖安装；LLM 配置存在用户目录（`~/.elwright/config.json`），不会丢。

## 当前状态

✅ **v0.1.10 已发布**（进度详情看 [docs/ROADMAP.md](docs/ROADMAP.md)，此处只留概览）：桌面壳已带能力工具箱、AI 对话（多轮/会话/能力协作/流式与取消）、代码浏览器（只读浏览/符号跳转/收藏书签/受控补丁编辑）、工作台（Todo/今日记录）、资源与课题工作区、人与人消息会话一期、设置中心（常规/外观/终端/模型档案）、集成终端、CLI `ew`。主干红线与下一步排期以 ROADMAP 为准；内置注册表保留 3 个真实示例（文本统计、能力类型说明、周报生成），个人能力通过导入加入 `~/.elwright/`。

## 快速开始（开发者）

```bash
git clone https://github.com/X-85/Elwright.git && cd Elwright

# CLI：列出/运行/查看/调用能力（脚本型与知识型无需 LLM）
cd src-tauri && cargo build --bin ew
./target/debug/ew ls

# 运行内置文本统计示例
./target/debug/ew run text-stats README.md
```

桌面端有两种开发方式，按需要的能力选：

```bash
# ① 真机运行（推荐）：需要本机有桌面应用构建工具链。
#   macOS：装 Xcode Command Line Tools（`xcode-select --install`）
#   Windows：装 Visual Studio Build Tools（C++ 工作负载 + Windows SDK）
#   Linux：参考 https://tauri.app/start/prerequisites/
cd src-tauri
npm --prefix ../src install          # 第一次需要：装前端依赖
../src/node_modules/.bin/tauri dev   # 启动 Tauri 桌面 app，前端热更新，自动连本地 ew 核心
```

```bash
# ② 浏览器预览（仅「查看类」功能可用：listCapabilities / viewDoc / exportCapability / checkUpdate）。
#   终端、AI 对话、能力增删、模型配置、技能调用等核心功能在浏览器里会降级或不可用——
#   想验这些必须走上面的真机 tauri dev。
cd src
npm install
npm run dev                          # http://localhost:5173
```

要打桌面端安装包（dmg / msi）：在 `src-tauri/` 下跑 `../src/node_modules/.bin/tauri build`（mac 产物在 `src-tauri/target/release/bundle/macos/`，Windows 在 `bundle/msi/`）。未签名——首次打开按 dmg / SmartScreen 的「仍要运行」流程过一次即可。想把真机构建交给 CI：直接 `git tag v0.1.x && git push origin v0.1.x`，`.github/workflows/release.yml` 会自动出 dmg + msi 上 GitHub Release。

想导入自己的能力，可以在桌面端使用「导入能力」，或执行 `ew import <文件>`；想让「技能型」能力接入大模型，再看 **[LLM 配置指引](docs/release/llm-setup-guide.md)**。

## License

MIT — 自由使用、修改、分发，让更多人受益。
