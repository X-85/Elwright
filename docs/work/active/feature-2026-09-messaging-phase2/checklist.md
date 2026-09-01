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

- [x] `src-tauri/src/core/identity.rs`（commit 2342abe；12 单测）
  - [x] `Identity::load_or_create(path)` 生成 ed25519 + X25519
  - [x] `id_base32()` 派生（16 字符；step3 修复为 SHA-256 前 10 字节）
  - [x] `create_invite(ttl)` → `(short_code, qr_payload, expires_at)`
  - [x] `accept_invite(&InboundInvite)` 完整校验（长度/字符集/有效期/ed25519 签名/短码一致）
  - [x] 单测：ID 一致性、邀请校验签名/有效期
- [x] IPC：`identity_get`、`identity_create_invite`、`identity_accept_invite`（commit d4c9251 + 7 例 mock runtime）
- [x] main.rs 注册
- [ ] Bridge：5 方法（前端接线待做）
- [ ] vitest：identity.test.ts / invite.test.ts

## Step 3 — 设置中心「消息中继」

- [x] `UserConfigFile::messaging_relay_url` 字段（commit d4c9251）
- [x] `ew config messaging` 子命令（show/set/clear；test 在 step4 补齐）
- [ ] SettingsCenter.vue 新分组（前端接线待做）
- [x] 「测试连接」——核心探测 `messaging_client::probe_relay` + IPC `test_messaging_relay` + `ew config messaging test`（commit step4）
- [ ] vitest：URL 校验
- [ ] e2e：URL 输入 + 校验 + 保存

## Step 4 — 客户端 + 中继最小回路

- [x] `docs/features/messaging/relay/Cargo.toml`
- [x] `docs/features/messaging/relay/src/main.rs`（axum + tokio-tungstenite；房间路由 + 64 帧暂存 + 30s 空房清理；日志零载荷）
- [x] `docs/features/messaging/relay/Dockerfile`（多阶段 + distroless）
- [x] `docs/features/messaging/relay/docker-compose.yml`
- [x] `docs/features/messaging/relay/README.md`
- [x] 客户端 `connect → handshake → send encrypted` 接通（tests/messaging_relay_smoke.rs：initiator/responder 双端 Noise_XX 经真实 relay 进程双向 AEAD 收发 + 断言 relay stderr 无明文）
- [x] 中继连通性探测 `core::messaging_client`（2 单测）+ IPC `test_messaging_relay` + CLI `test` 子命令

## Step 5 — 离线消息队列

- [x] 存储选定：~~sled~~ → 零依赖 JSONL（偏差记入 ADR-002「实施偏差」段）
- [x] `core::messaging_queue`：Outbox open/enqueue/list/remove/record_attempt（FIFO + 按对端过滤 + 损坏行容忍 + 原子重写）
- [x] 队列消息加密：只存 `Transport::send` AEAD 密文（hex），明文不落盘（单测 `plaintext_never_hits_disk` 强制）
- [ ] 握手成功后从队列 pop 投递——属完整客户端接线（PeopleChatView 适配器替换 + 前端接入），随前端联调落地；队列原语已就绪
- [x] 单测：5 例（FIFO 往返 / 明文不入盘 / 损坏行 / attempts / 空载荷边界）

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