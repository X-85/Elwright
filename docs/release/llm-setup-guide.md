# LLM 配置指引（写给没用过大模型的朋友）

> Elwright 的脚本型和知识型能力**开箱即用，不需要任何 LLM**。只有 6 个「技能型」能力（八层拷问、提示词优化等）需要接一个大模型才能由 AI 代跑；没接也不报错，会自动变成一份可以照着做的操作手册（SOP）。
> 本文档是发布材料的初稿，阶段 3b 合入后将挂入 README「快速开始」。

## 我需要配 LLM 吗？

| 你想要 | 需要配吗 |
|---|---|
| 跑脚本（文档搜索、Excel 转 md…） | ❌ 不需要 |
| 看知识笔记 | ❌ 不需要 |
| 让 AI 帮我拷问技术细节、写周报 | ✅ 需要 |

## 第一步：搞一个 OpenAI 兼容端点

技能型只认「OpenAI 兼容接口」，以下任选其一：

### 方案 A：本地跑 Ollama（免费、数据不出机器，推荐）

1. 到 https://ollama.com 下载安装（Windows/macOS 都有）。
2. 装完后在终端执行（只需一次，模型约 4-5 GB，视网络等待）：
   ```bash
   ollama pull qwen3:8b        # 中文能力好的入门款；机器内存 ≥16G 用这个
   # 机器配置低就换小号：ollama pull qwen3:4b
   ```
3. Ollama 装好就会一直在本机 `11434` 端口提供服务，地址就是 `http://localhost:11434/v1`。

### 方案 B：云端 API（要联网、按量付费）

任意「OpenAI 兼容」的服务都行（各云厂商大模型平台基本都兼容）。在它们的控制台拿到三样东西：接口地址（一般以 `/v1` 结尾）、API Key、模型名。

## 第二步：填配置（推荐用命令，一次配好持久保存）

```bash
ew config set base_url http://localhost:11434/v1     # Ollama 本地
ew config set model qwen3:8b
ew config set api_key 你的Key                          # 云端端点才需要；Ollama 跳过
ew config                                             # 查看当前生效配置与来源
```

配置保存在 `~/.elwright/config.json`（用户级，重开终端仍有效）。加 `--local` 则写到项目目录 `config.local.json`（不进 git，适合单项目不同端点）。

<details><summary>旧方式：环境变量（临时/排障用）</summary>

**macOS / Linux（终端里执行，或写进 `~/.zshrc`）：**

```bash
export ELWRIGHT_LLM_BASE_URL="http://localhost:11434/v1"   # 方案 A 填这个
# 方案 B 则填云端地址，如 https://api.xxx.com/v1
export ELWRIGHT_LLM_API_KEY=""        # Ollama 留空；云端填你的 Key
export ELWRIGHT_LLM_MODEL="qwen3:8b"  # 和你 pull / 购买的模型名一致
```

**Windows（PowerShell）：**

```powershell
setx ELWRIGHT_LLM_BASE_URL "http://localhost:11434/v1"
setx ELWRIGHT_LLM_MODEL "qwen3:8b"
setx ELWRIGHT_LLM_API_KEY ""
```

（`setx` 写入后需重开终端生效）

## 第三步：验证

```bash
ew invoke tech-grill Rust所有权
```

- 看到 AI 的回复 → 配置成功。
- 看到「【离线降级】展示 SOP」→ 没配好或端点不通，但这不是错误，照着 SOP 手动做也行。检查环境变量拼写和 Ollama 是否在运行（`ollama list`）。

## 常见问题

- **报「请求失败」**：Ollama 没启动（重新打开 Ollama 程序）；或地址少了/多了 `/v1`。
- **模型名报错**：`ollama list` 看确切名字，照抄。
- **公司网络限制**：本地 Ollama 不走外网，公司内也能用——这正是 Elwright「LLM 是增强不是地基」的设计：断网时一切照常，只是技能型退化为 SOP。
- **以后会有设置界面吗**：桌面版规划中有配置界面，当前 CLI 版先用环境变量。
