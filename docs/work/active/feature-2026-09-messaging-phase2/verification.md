# Verification（验证记录）

> 详细方案见 [plan.md](./plan.md)；取舍见 [ADR-002](../../../features/messaging/decisions/ADR-002-messaging-transport.md)。

## 验证清单（标记约定见 AI_CODE_AGENT_MAINTENANCE.md §6）

### 【自动化】

（实施完成后回填）

### 【手测】

（真机点验完成后回填）

## 已知遗留

- v0 不支持多端：同 ID 多端会覆盖密钥对导致历史消息在另一端无法解密（文档明示）
- 二维码渲染首版允许引入 `qrcode` 包（~10KB），后续评估是否自写
- 短码 5 分钟有效期是默认，可配置（ADR-002 §未决）