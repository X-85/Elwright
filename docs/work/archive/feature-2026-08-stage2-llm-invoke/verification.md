# 验证记录（2026-08-21，家里 macOS，rustc 1.98.0）

## 构建与静态检查

- `cargo build --bin ew` 成功（新增 reqwest 0.13.4，编译 19.8s）。
- `cargo test`：3 passed, 0 failed（`core::llm::tests` URL 拼接 3 例）。

## 功能验证

| 场景 | 命令 | 结果 |
|---|---|---|
| 未配置 LLM | `ew invoke tech-grill`（无环境变量） | ✅ 打印未配置提示 + tech-grill-sop.md 全文 |
| 端点不可达 | `ELWRIGHT_LLM_BASE_URL=http://localhost:9/v1 ew invoke tech-grill` | ✅ 打印调用失败原因 + 降级 SOP，不挂死不 panic |
| 调用成功（模拟） | `ELWRIGHT_LLM_BASE_URL=http://127.0.0.1:18434/v1 ew invoke tech-grill Rust所有权`（本地 mock 端点） | ✅ 正确收到回复；验证 system=能力 prompt、user=用户输入、model 均正确送达 |
| 空附加输入 | `ew invoke prompt-optimizer`（mock 端点） | ✅ user 回退为「（无附加输入，请按模板直接执行）」 |
| 类型限定 | `ew invoke doc-keyword-search` | ✅ 报错「不是技能型能力」，退出码 1 |
| 回归冒烟 | `ew ls` | ✅ 24 项能力正常列出 |

## 未验证项（说明）

- 真实 LLM（云端端点 / 本地 Ollama）未验证：本机无可用模型。成功路径已用本地 mock OpenAI 兼容端点等价覆盖（协议层一致）。
- API key 的 Bearer 认证头：代码走 reqwest 标准路径，未用需鉴权的真实端点实测。
- Windows 公司机器：需拉取本分支后 `cargo +stable-x86_64-pc-windows-gnu build` 复验（无 MSVC 依赖新增，风险低）。

## 结论

构建、单测、降级路径、成功路径（mock）、类型守卫全部通过。阶段 2 核心链路完整，**可进入用户确认环节**（真实端点复验由用户配置后进行）。
