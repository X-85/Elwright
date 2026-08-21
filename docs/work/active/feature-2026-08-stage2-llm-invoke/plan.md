# 阶段 2：LLM 客户端 + 技能型 invoke + 离线降级

## 目标

- `llm.rs` 从占位换成真实的 OpenAI 兼容 `/v1/chat/completions` 调用（自写 reqwest thin client，架构方案 §9.2 已锁定）。
- `ew invoke <id>`：配置了 LLM 时调用 LLM；LLM 不可达或调用失败时降级展示 `degradeDoc` SOP，不报错退出。
- LLM 配置沿用环境变量 `ELWRIGHT_LLM_BASE_URL` / `ELWRIGHT_LLM_API_KEY` / `ELWRIGHT_LLM_MODEL`。

## 实现步骤

1. `cargo add reqwest --features blocking,json`（CLI 是同步程序，用 blocking 客户端，不显式引入 tokio；见 ADR-001）。
2. `llm.rs`：实现 `chat()`——拼 URL、构造 messages（能力 prompt 作 system，用户输入作 user）、60s 超时、解析 `choices[0].message.content` 与错误响应。
3. `ew.rs` 的 `invoke`：限定技能型；有 LLM 配置则调用，失败降级 SOP；无配置直接降级。
4. 创建 `resources/docs/tech-grill-sop.md` 种子 SOP，使降级路径可端到端验证。
5. 单元测试：URL 拼接逻辑；构建 + 手动验证降级路径。

## 非目标

- 不做流式输出、工具调用（后期再评估，见架构方案 §9.2）。
- 不做 LLM 可达性预检探针（chat 失败即降级，语义等价且更简单）。
- 不导入其余 5 个技能型的 SOP 文件（仅 tech-grill 作种子验证）。
- 不动桌面壳（阶段 3）。

## 风险

- LLM 端点无响应导致 CLI 挂死 → 60 秒超时兜底，超时按失败降级。
- 端点返回非标准 JSON → 解析失败按失败降级，原始错误信息打印给用户。

## 验证方式

- `cargo test`（URL 拼接单元测试）。
- `cargo build --bin ew` 成功。
- 无 LLM 环境变量时 `ew invoke tech-grill` 展示 SOP 降级文案。
- 配置指向不可达端点（如 `http://localhost:9`）时同样降级、不 panic 不挂死。
