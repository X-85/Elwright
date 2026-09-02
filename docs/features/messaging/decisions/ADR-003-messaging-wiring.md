# ADR-003 — 消息会话② UI 接线：端到端可用闭环

- **状态**：已接受（2026-09-01，用户定向「把消息功能做成真正可用的闭环」后实施）
- **Feature**：[messaging](../README.md)
- **接续**：ADR-002（传输层核心已落地）；本 ADR 只管「核心 → 可用」的接线层
- **任务目录**：`docs/work/active/feature-2026-09-messaging-phase2/`（切片 A/B/C 续用）

## 背景

传输层核心（协议/身份/邀请/中继/队列）已有完整测试，但对用户不可见：
PeopleChatView 仍是阶段① localStorage 本地投递。接线层要回答四个问题：

## 决定

### D1 — 邀请 v3：QR 增载对端 X25519 DH 公钥

ADR-002 的 v2 邀请只带签名公钥——接收方拿不到对端 DH 公钥，
而 Noise_XX 握手后才知道对方 DH 公钥，无法在握手前绑定「这是我要的那个人」。

v3 格式（9 段）：

```
elwright-invite:v3:{id}:{edpk}:{dhpk}:{short_code}:{expires_at}:{nonce}:{sig}
```

- 签名内容：`id || dh_pub || expires_at || nonce`（ed25519，64 字节）
- 接收方校验：格式 / 有效期 / 签名 / **`id == derive_id_from_dh_public(dh_pub)`**
  （ID 本就由 DH 公钥派生——这是硬绑定，伪造 DH 公钥即过不了 ID 校验）
- 联系人落盘：`~/.elwright/contacts.json`（`{peer_id, signing_pub_hex, dh_pub_hex, alias?, added_at}`）

v2 从未在 UI 暴露过，直接替换不迁移。

### D2 — 成对房间 + 按 ID 排序的确定性角色

- 房间号：`min(idA,idB) + "-" + max(idA,idB)`（两端算出同一路径，`/ws/<room>`）
- Noise_XX 角色：**ID 较小的一方恒为 initiator**——两端同时上线也不会撞角色
- 中继只多了一个路由约定，无任何逻辑变化

### D3 — sync_peer 阻塞式收发 + listener 线程 + 前端轮询

- `sync_peer(relay, identity, contact)`：连接 → 握手（按 D2 角色）→
  **校验 `remote_static == contact.dh_pub`**（不符即断开，防中间人）→
  flush 发件箱（见 D4）→ 收 Data 帧直到空闲超时 → 落 inbox → 关闭
- 后台 listener 线程：IPC `messaging_start_listener` 启动（幂等），每轮遍历联系人
  依次 `sync_peer`，无中继/无联系人时空转长睡眠；配置每轮重读（改 URL 免重启）
- 收到的消息落 `~/.elwright/messaging/inbox.jsonl`（D4 同款本地加密），
  前端 `messaging_poll_inbox(since_id)` 轮询合并进阶段① localStorage 会话——
  不重写 phase1 存储模型，收发两端各加一条薄适配
- 发送：`messaging_send(peer_id, text)` 即时触发一次 `sync_peer`（快路径）；
  失败进发件箱，状态 `queued`，对端上线后由 listener flush 补投
- 弃选：tauri Channel/事件推送（要动 Bridge 事件管线，个人聊天轮询 2s 足够）；
  每联系人常驻 WS（重连管理复杂度不成比例）

### D4 — 离线队列/收件箱改存「本地静态密钥加密的明文」

ADR-002 §5 原设想「队列存会话密文」有缺陷：会话密钥不跨连接持久，
重连后旧密文无法解密、flush 即垃圾。改为：

- 本地静态密钥：身份目录下 `messaging.localkey`（首次随机生成 32 字节）
- 落盘格式：`chacha20poly1305(local_key, nonce12, aad=peer_id)` 密文（hex）——
  「明文不入盘」单测继续强制
- flush 时解出明文，用**当次握手的新会话** `transport.send()` 重加密——
  队列内容与会话密钥解耦
- 新增依赖：`chacha20poly1305`（RustCrypto 纯 Rust，rustls 同源，windows-gnu 无负担）。
  实施备注：原列 `hkdf`/`hmac` 未引入——本地密钥为单用途专用密钥，直接作为
  ChaCha20Poly1305 key 使用即可，少两个依赖（与项目极简底线一致）
- 弃选：存明文 + 文件权限（不满足清单硬要求）；sealed box（ephemeral X25519
  每条加密，对本地静态场景是杀鸡用牛刀）

## 红线对齐

- 默认不启用：未配置中继 URL 时 listener 空转、send 直接降级本地——无任何网络行为
- 中继只见密文与房间号；身份/联系人/队列/收件箱全部本地文件
- 不做：群聊、多端、已读回执、在线状态（ADR-002 非目标继续有效）

## 后果

- 消息功能成为真闭环：邀请互加 → 配中继 → 互发文字 → 离线补投
- 真机点验新增：双账户互发、篡改 DH 公钥的联系人应握手失败、离线补投
- `messaging_relay_smoke.rs` 后续补「经 relay 的双角色握手 + 发件箱 flush」用例
