# Verification（验证记录）

> 详细方案见 [plan.md](./plan.md)；取舍见 [ADR-002](../../../features/messaging/decisions/ADR-002-messaging-transport.md)。

## 验证清单（标记约定见 AI_CODE_AGENT_MAINTENANCE.md §6）

### 【自动化】

**Step 1 — 协议层骨架（2026-09-01 完成）：**

- cargo test --lib：96/96 通过（其中 messaging_transport 12 个：握手 round-trip / 双向加解密 / 篡改拒绝 / nonce 重放拒绝 / 显式 rekey / 帧握手-数据-控制 encode-decode / 版本错配 / 截断头 / 未知帧类型 / 密文不出现明文窗口匹配）
- cargo clippy --all-targets -- -D warnings：0 警告
- cargo fmt --all -- --check：干净
- 实施偏差：去 dryoc 与 libsodium-sys 依赖；改用 snow TransportState 自带 ChaCha20-Poly1305 AEAD（已写入 ADR-002「实施偏差」段）
- windows-gnu 工具链编译：待 windows CI runner 验证

**Step 2 — 本地身份 + 邀请（2026-09-01 完成，commit 2342abe）：**

- cargo test --lib identity：13/13 通过（base32 字符集/长度、身份唯一性、持久化重载、load_or_create 幂等、短码长度/字符集、有效期、v2 QR 8 段格式、错误长度拒绝、过期拒绝、邀请往返、篡改短码拒绝、篡改签名拒绝、**16 字符 ID 回归**）
- 修复：derive_id_from_dh_public 改取 SHA-256 前 10 字节（早期版 5 字节只能产 8 字符，IPC 测试暴露）
- 实施偏差：邀请短码取签名头 5 字节 base32 截 6 字符；签名验证走完整 64 字节签名（v2 QR 携带）

**Step 3 — 中继 URL 配置 + 身份 IPC（2026-09-01 完成，commit d4c9251）：**

- 单测 +4（validate_relay_url 合法/非法、relay_url 往返、字段保留），lib 113/113
- tests/messaging_phase2_ipc.rs：7/7（身份稳定、v2 QR、ttl 边界、邀请往返+签名篡改、垃圾 QR 拒绝、配置往返、非法 URL 拒绝）
- CLI 手测：show/set/clear + 非法协议拒绝（http:// 报错退出码 1）

**Step 4 — 中继参考实现 + 客户端最小回路（2026-09-01 完成，commit d3a717b）：**

- tests/messaging_relay_smoke.rs 2/2：**真实 relay 子进程** + 双端（initiator/responder）Noise_XX 三步握手经 WebSocket 完成；双向 AEAD 收发内容一致；**relay stderr 断言不含明文片段**（验证清单「明文不出现于 relay 日志」）
- messaging_client 探测 2 单测 + CLI 手测三条路径：✓ 已连接（1ms）/ 未配置报错 / 连接拒绝报错
- 实施备注：客户端测试遵循「initiator 先发后收、responder 先收后发」错开时序（全双工前的最小同步约定）

**Step 5 — 离线消息队列（2026-09-01 完成，commit b1e59c8）：**

- 单测 5/5：FIFO 往返（含按对端过滤）、**明文不入盘**（验证清单项，读原始文件字节断言）、损坏行跳过+重写清除、attempts 递增、空载荷/缺文件边界
- 实施偏差：~~sled~~ → 零依赖 JSONL（已写入 ADR-002「实施偏差」段）

**Step 6 — 文档回填（2026-09-01 完成）：**

- behavior.md §第二阶段 + 本阶段边界、architecture.md 传输层结构图、changelog.md 2026-09-01 条目
- ROADMAP「人与人消息会话」条目更新为「传输层核心已落地，UI 接线待做」

**最终门禁（Step 6 时点）：**

- cargo test：120 lib + 全部集成套件（含 7 IPC + 2 中继冒烟）全绿
- cargo clippy --all-targets -- -D warnings：0 警告；cargo fmt --check：干净
- cargo build --bin ew：成功

### 【手测】

（真机点验完成后回填）

## 已知遗留

- v0 不支持多端：同 ID 多端会覆盖密钥对导致历史消息在另一端无法解密（文档明示）
- 二维码渲染首版允许引入 `qrcode` 包（~10KB），后续评估是否自写
- 短码 5 分钟有效期是默认，可配置（ADR-002 §未决）