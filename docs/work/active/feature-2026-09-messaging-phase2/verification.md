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

### 【手测】

（真机点验完成后回填）

## 已知遗留

- v0 不支持多端：同 ID 多端会覆盖密钥对导致历史消息在另一端无法解密（文档明示）
- 二维码渲染首版允许引入 `qrcode` 包（~10KB），后续评估是否自写
- 短码 5 分钟有效期是默认，可配置（ADR-002 §未决）