# verification：OpenAI 兼容端点配置后不可用（C4）

## 修复内容

- `chat_url` 对裸根 base_url（无 `/v1` 后缀）自动补
  `/v1/chat/completions`；已带 `/v1` 行为不变。非流式/流式/测试连接
  三个调用点一并生效。

## 验证记录

- 【自动化】独立探针（GNU 工具链，2026-09-03）：6 用例覆盖裸根、
  尾斜杠、带 /v1、网关子路径 /api/v1、自定义版本段 /v1-beta——
  ALL PASS。
- 【自动化】单测 `works_without_version_path`（更新，含点验原始地址
  `http://10.54.39.45:8080`）+ `keeps_custom_version_paths`（新增）；
  `cargo test --lib` 执行留 CI（本机 GNU 限制同 B3/B5/B4）。
- 【自动化】Rust 门禁：fmt clean；clippy --lib / --tests 各 0 警告；
  ew 构建成功。期间本机 target 缓存 rmeta mmap 损坏（STATUS 大规模
  invalid metadata），`cargo clean` 全量重建恢复，与本改动无关。
- 【自动化】前端四闸例行全绿（lint 0 / vitest 86 / build 成功；本项
  无前端改动）。
- 【手测】（留给用户，需真机内网网络，Agent 无法代验）
  1. 装新 msi → 设置中心 → 模型设置：base_url 填
     `http://10.54.39.45:8080`（无需 /v1）、key、glm-5.3-flash →
     「测试连接」→ 预期「连接正常（http://10.54.39.45:8080/v1/chat/completions，model=glm-5.3-flash）」。
  2. 若失败看状态码：404 → 端点不在 /v1/chat/completions（base_url
     写全到版本段再试）；401 → API Key；400 → 模型名/max_tokens
     限制（整段错误发回给 Agent 二次定位）。
  3. 通过后 invoke 一次能力 + AI 对话发一条消息确认完整链路。
