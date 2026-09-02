# Transport Protocol — Elwright Messaging（阶段②）

> 对应 [ADR-002](./decisions/ADR-002-messaging-transport.md)。
> 实施定义：见 `src-tauri/src/core/messaging_transport.rs`。

## 总览

消息会话客户端与中继（自托管 WebSocket 服务）之间的二进制协议。

- 握手：Noise_XX_25519_ChaChaPoly_SHA256（snow 0.9 实现，纯 Rust，跨平台）
- 数据通道：snow `TransportState` 自带的 ChaCha20-Poly1305 AEAD（16 字节 tag，nonce 由 snow 内部计数器维护，自动拒绝重放）
- 控制帧：明文（仅用于协议信号：ping/pong/close/error，不携带业务数据）

> 实施期与 ADR §D4 的偏差：原计划用独立 libsodium `secretstream` 流加密；调研发现 snow TransportState 自带的 ChaCha20-Poly1305 AEAD + nonce 单调管理 + `rekey_*` 接口已覆盖 ADR §D4 的全部安全需求（AEAD 完整性 / 重放拒绝 / 顺序保证 / 显式 rekey）。实施期去掉独立 secretstream 封装，简化协议层，少两个依赖（dryoc / libsodium-sys）。变更已写入 ADR-002「实施偏差」段。

## 帧格式

所有帧以一字节 `version` 开头（当前 `0`）：

```
[version: u8][frame_type: u8][...payload...]
```

| frame_type | 名称     | 载荷                                  |
| ---------- | -------- | ------------------------------------- |
| 0          | Handshake | Noise_XX 协议原生消息（snow `write_message` 输出）|
| 1          | Data      | snow TransportState 输出（AEAD ciphertext + 16B tag）|
| 2          | Control   | `[code: u8][...payload...]`           |

### Handshake 帧（type 0）

载荷为 snow `HandshakeState::write_message` 的原始输出：

- Noise_XX 模式三步：
  - 发起方（client）写 msg 1（载荷空）
  - 接收方（server）读 msg 1，写 msg 2（载荷空）
  - 发起方读 msg 2，写 msg 3（载荷空）
  - 接收方读 msg 3（仅此步，雪雪 0.9 的 responder 在写完 msg 2 之后即可调用 `into_transport_mode`，但响应 `is_handshake_finished()` 的安全判定在读 msg 3 后）

实现见 `core::messaging_transport::Handshake::step`（普通步骤）与 `read_final`（仅读最后一步专用，避免 `step` 内强制 write 失败）。

### Data 帧（type 1）

载荷 = `Transport::send(plaintext)` 输出：

- 长度 = `len(plaintext) + 16`（16B AEAD tag）
- 内含 snow 自身 nonce（计数器），对端按顺序逐 chunk 处理
- 任何对 ciphertext 的篡改 → 接收端解密返回 `Noise("AEAD decrypt failed")`
- 重放同一条 ciphertext → 接收端第二次 `read_message` 因 nonce 不匹配而失败

### Control 帧（type 2）

| code | 名称 | payload | 备注 |
| --- | --- | --- | --- |
| 0 | Ping | 任意 | 心跳；接收方应回 Pong（载荷回显）|
| 1 | Pong | 任意 | Ping 回应 |
| 2 | Close | 任意 | 通知对端本方向关闭；接收方不应再 push |
| 3 | Error | UTF-8 字符串 | 协议级错误，UI 可直接展示 |

控制帧明文传输——不携带业务数据，仅协议信号。

## 解析规则

1. 任何帧首字节必须等于 `PROTOCOL_VERSION`（0），否则返回 `VersionMismatch`。
2. 第二字节必须在 `[0, 1, 2]` 内，否则 `UnknownFrameType`。
3. 控制帧第三字节必须在 `[0, 1, 2, 3]` 内，否则 `UnknownControlCode`。
4. 长度 < 2 → `Truncated`。

## 重放与顺序保证

- Data 帧 nonce 由 snow 内部单调计数器维护，外部无法干预。
- 重放旧 ciphertext → 第二次解密失败（snow `StateError::DecryptError`）。
- snow TransportState 不强制顺序（按 nonce 取），但 UI 应按 `recv_nonce` 单调展示。

## rekey 策略（v0）

- 暂用 snow 默认 rekey 阈值（256 条消息或 1 GiB 数据后自动 rekey）。
- 客户端可主动调用 `Transport::rekey_send()` / `rekey_recv()` 强制滚动。
- 未来可加入每 N 分钟 / 每 N MB 显式 rekey 调度（v0 不实现）。

## 多端 / 群消息 / 离线队列

- v0 **不支持多端**（同 ID 多端会覆盖密钥对导致历史消息无法在另一端解密）
- v0 **不支持群消息**（仅一对一房间）
- 离线消息队列由客户端在本地存储加密消息（用同一 `Transport` 实例的 secretstream key——这里直接复用 Transport 实例本身，消息在本地存储前重新 `send()` 到一个本地 secretstream 包装；实施期由 Step 5 落实）

## 实现参考

- 客户端：`src-tauri/src/core/messaging_transport.rs`
  - `Handshake` / `Handshake::step` / `Handshake::read_final` / `Handshake::into_transport`
  - `Transport::send` / `Transport::recv` / `Transport::rekey_send` / `Transport::rekey_recv`
  - `Frame::encode_handshake` / `Frame::encode_data` / `Frame::encode_control`
  - `Frame::parse_header` / `Frame::parse_control`
  - `complete_handshake(alice_static, bob_static) -> (Transport, Transport)` 测试辅助
- 中继（Step 4 待写）：`docs/features/messaging/relay/`
- 测试：`messaging_transport::tests`（12 个用例）