# change-note：OpenAI 兼容端点配置后不可用（C4）

## 问题（真机点验 v0.1.13，Windows）

用户在设置中心配置 OpenAI 兼容端点（base_url `http://10.54.39.45:8080`、
API 格式 Chat Completions、模型 glm-5.3-flash）后不可用；同一配置在
ZCode 中可用。

## 根因

`core::llm::chat_url` 把 base_url 直接拼 `/chat/completions`——约定
base_url 应「到 /v1 为止」（ADR-001 示例 `https://api.openai.com/v1`），
但用户填的是裸根 `http://10.54.39.45:8080`（无 `/v1`）。拼出的
`http://10.54.39.45:8080/chat/completions` 不是 OpenAI 兼容端点的标准
路径（标准为 `/v1/chat/completions`），请求 404，测试连接与对话全部
失败。ZCode 等工具会对裸根自动补 `/v1`，所以同配置在那边可用——
这是配置约定差异，不是用户配错。

## 修改范围

- `src-tauri/src/core/llm.rs`（`chat_url`，非流式/流式/测试连接三个
  调用点全部经过它，一处修复三处生效）：
  - base_url 以 `/v1` 结尾 → 原样拼 `/chat/completions`（原行为）；
  - 否则（裸根）→ 自动补 `/v1/chat/completions`。
  - 边界：非 `/v1` 结尾的自定义版本段（如 `…/v1-beta`）也会被补
    `/v1`——此类端点极少，注释已说明写全路径即可。
- 单测更新：`works_without_version_path`（裸根补 `/v1`，含点验原始
  地址）、新增 `keeps_custom_version_paths`（`/api/v1`、`/v1-beta`）。
- 前端无改动（LlmSettings 表单本就不含「API 格式」字段，无需适配）。

## 风险

- 低。`chat_url` 是唯一出口；已带 `/v1` 的既有配置行为完全不变。
- 兜底：若修复后仍有问题（如公司内网代理拦截、max_tokens=1 对
  reasoning 模型 400），测试连接的错误信息会携带 HTTP 状态码与响应
  片段（既有实现），可据此二次定位——已写入手测步骤。

## 验证

- 【自动化】独立探针（GNU）：6 个 URL 用例（裸根/裸根带尾斜杠/带
  /v1/带尾斜杠/网关子路径/自定义版本段）ALL PASS。
- 【自动化】单测更新 + 新增；`cargo test --lib` 执行留 CI（本机 GNU
  限制同前几轮）。
- 【自动化】Rust 门禁：fmt clean、clippy lib+tests 0 警告、ew 构建
  成功（期间本机 target 缓存出现 rmeta mmap 损坏，clean 重建恢复，
  与本次改动无关，已在台账环境备注记录）。前端四闸例行全绿（86/86）。
- 【手测】（留给用户，必须真机网络）
  1. 装新 msi → 设置中心 → 模型设置，base_url 仍填
     `http://10.54.39.45:8080`（无需加 /v1），key 与模型照旧 →
     「测试连接」→ 预期「连接正常」。
  2. 若仍失败：看错误提示里的状态码——404 = 端点路径不在
     `/v1/chat/completions`（尝试 base_url 写全到版本段）；401 = key；
     400 = 模型名或 max_tokens 限制（把错误全文发给 Agent）。
  3. 测试通过后 invoke 一次能力 + AI 对话发一条消息，确认真实链路。
