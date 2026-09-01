# Checklist（实施进度）

> 详细方案见 [plan.md](./plan.md)；取舍见 [ADR-002](../../../features/messaging/decisions/ADR-002-messaging-transport.md)。

## Step 1 — 协议层骨架

- [x] `src-tauri/Cargo.toml` 加依赖：`snow = "0.9"` + `ed25519-dalek` + `thiserror`
- [x] `src-tauri/src/core/messaging_transport.rs` 新建
  - [x] `Handshake::new_initiator / new_responder / step / read_final / into_transport`
  - [x] `Transport::send / recv / rekey_send / rekey_recv / send_nonce / recv_nonce / remote_static`
  - [x] `Frame` 序列化（handshake / data / control 三类 + 帧头解析 + 控制码解析）
  - [x] `complete_handshake(alice, bob)` 测试辅助
- [x] 帧格式文档：`docs/features/messaging/transport-protocol.md`
- [x] 单测（`#[cfg(test)]`，12 个）：
  - [x] 握手 round-trip（远端 static 公钥双向不同）
  - [x] 加解密 round-trip（双向）
  - [x] 密文不可破（手工改字节）
  - [x] nonce 重放拒绝
  - [x] 显式 rekey 后仍能加解密
  - [x] 帧握手/数据/控制三类的 encode-decode round-trip
  - [x] 版本错配 / 截断头 / 未知帧类型 三种边界
  - [x] 密文不出现明文（窗口匹配测试）
- [x] windows-gnu 工具链下 snow 编译通过（CI 验证；公司机手测延后到 PENDING）

## Step 2 — 本地身份 + 邀请

- [ ] `src-tauri/src/core/identity.rs`
  - [ ] `Identity::load_or_create(path)` 生成 ed25519 + X25519
  - [ ] `id_base32()` 派生
  - [ ] `Invite::create(identity, ttl)` → `(short_code, qr_payload, expires_at)`
  - [ ] `Invite::accept(short_code, my_identity) -> PublicKey`
  - [ ] 单测：ID 一致性、邀请校验签名/有效期
- [ ] IPC：`identity_get`、`identity_create_invite`、`identity_accept_invite`
- [ ] main.rs 注册 + mock runtime 用例
- [ ] Bridge：5 方法
- [ ] vitest：identity.test.ts / invite.test.ts

## Step 3 — 设置中心「消息中继」

- [ ] `UserConfigFile::messaging_relay_url` 字段
- [ ] `ew config messaging` 子命令
- [ ] SettingsCenter.vue 新分组
- [ ] 「测试连接」按钮（实测握手一次）
- [ ] vitest：URL 校验
- [ ] e2e：URL 输入 + 校验 + 保存

## Step 4 — 客户端 + 中继最小回路

- [ ] `docs/features/messaging/relay/Cargo.toml`
- [ ] `docs/features/messaging/relay/src/main.rs`（axum + tokio-tungstenite）
- [ ] `docs/features/messaging/relay/Dockerfile`
- [ ] `docs/features/messaging/relay/docker-compose.yml`
- [ ] `docs/features/messaging/relay/README.md`
- [ ] 客户端 `connect → handshake → send encrypted` 接通
- [ ] IPC mock runtime：自起 relay + 两端 mock

## Step 5 — 离线消息队列

- [ ] sled / rusqlite 选定
- [ ] 队列消息 secretstream 加密
- [ ] 握手成功后从队列 pop
- [ ] 单测：队列明文不入磁盘

## Step 6 — 文档回填

- [ ] behavior.md §第二阶段
- [ ] architecture.md MessageTransport 段替换
- [ ] changelog.md v0.x.y
- [ ] ROADMAP 进行中清空 + 人与人消息会话条目标记完成
- [ ] ROADMAP 已完成里程碑加本阶段条目

## 收口

- [ ] 本地闸门全绿（cargo fmt/clippy/test + eslint + vitest + coverage + vite build + e2e）
- [ ] CI 全绿（ci.yml + 视情况 release.yml）
- [ ] 台账 Q34 收口
- [ ] 任务目录归档（人执行）