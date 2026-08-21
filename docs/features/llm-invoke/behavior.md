# 技能型 invoke —— 业务行为

## 触发条件

- 用户执行 `ew invoke <id> [prompt...]`，且 `id` 在 `capabilities.json` 中注册为 `type: "skill"`。
- 其他类型（script/knowledge）调用 invoke 直接报错退出（退出码 1）。

## LLM 调用（在线路径）

- 配置来源：环境变量 `ELWRIGHT_LLM_BASE_URL`（必填才视为已配置）、`ELWRIGHT_LLM_API_KEY`（可选，非空时作 Bearer 认证）、`ELWRIGHT_LLM_MODEL`。
- 请求：POST `{base_url}/chat/completions`，messages 为两条——能力 `prompt` 字段作 system，命令行剩余参数拼接作 user。
- 用户未给附加输入时，user 内容为「（无附加输入，请按模板直接执行）」。
- 成功：打印 `choices[0].message.content`，退出码 0。

## 降级（离线路径）

以下任一情况触发降级，**打印 SOP 而非报错**：

1. 未设置 `ELWRIGHT_LLM_BASE_URL`。
2. 请求失败（网络不可达、连接被拒等）。
3. 超时（60 秒）。
4. 端点返回非 2xx 或响应不是合法的 chat completion JSON。

降级时 stderr 打印原因提示，stdout 打印 `degradeDoc` 指向的 SOP 文件全文。

## 不允许的情况

- `degradeDoc` 未配置或文件不存在：打印相应提示文案，不报错退出。
- 不支持流式输出、多轮对话、工具调用（见 ADR-001 与架构方案 §9.2，后续按需评估）。

## 对外部系统的影响

- 仅对用户配置的 OpenAI 兼容端点发起一次 HTTPS/HTTP POST，无其他副作用。
