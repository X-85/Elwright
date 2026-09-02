# feature-2026-09-messaging-phase2 — 计划

> 任务对应 [messaging/phase 2：轻量身份/邀请与一对一消息传输](../../../features/messaging/decisions/ADR-002-messaging-transport.md)（ADR-002，已批准 2026-09-01）。
> 接续 [phase 1：本地消息会话](../../../features/messaging/README.md)（PR #1 已并入 main）。

## 目标

把 PeopleChatView 的本地消息投递适配器替换为基于 WebSocket 中继的真实传输，端到端加密、可离线降级、默认不启用。详细取舍见 ADR-002。

## 不在本阶段

- 群消息、多设备、Signal Double Ratchet（前向保密由 secretstream rekey 覆盖）
- 中继的公共实例 / 仓库托管
- 端到端文件传输（图片仍走阶段①的资源模型，不上协议层）
- 实时协作空间——phase 3

## 实施切分

### Step 1 — 协议层骨架（本阶段第一刀）

新增模块 `src-tauri/src/core/messaging_transport.rs`：

- Noise_XX 握手：使用 `snow` crate（Noise Protocol Framework Rust 实现，已被多家生产项目使用）
- 数据通道：`libsodium-sys` 的 `crypto_secretstream_xchacha20poly1305_*`
- 帧格式：
  - 握手帧：`[u8 version][u8 msg_type][16B nonce][payload...]`
    - msg_type: 0=Noise handshake, 1=application data, 2=control (ping/pong/close)
  - 数据帧：`[tag(1B)][nonce(24B)][ciphertext+mac]`，tag 由 secretstream 维护
- 帧格式文档化：`docs/features/messaging/transport-protocol.md`
- 单元测试：
  - 握手 round-trip（initiator/responder 双端，断言两端派生相同 session key）
  - 密文不可破（手工改一个字节解密必须失败）
  - nonce 重放拒绝（重发同一条 secretstream chunk 必须失败）
  - 帧解析边界（短帧 / 长帧 / 截断 / 版本号错配）

### Step 2 — 本地身份 + 邀请

新增模块 `src-tauri/src/core/identity.rs`：

- 首次启动在 `~/.elwright/identity/` 生成密钥对：
  - `signing.ed25519`（ed25519 长私钥）
  - `dh.x25519`（X25519 长私钥）
- 派生 `id.base32`：X25519 公钥 → SHA-256 → 头 80 bit → 16 字符 base32 Crockford
- 邀请生成/校验：`Inviter::create_invite()` → `(short_code, qr_payload, expires_at)`；`Invitee::accept_invite(short_code)` → 校验签名与有效期
- IPC：`identity_get`、`identity_create_invite`、`identity_accept_invite`
- Bridge：5 方法 + vitest 覆盖

### Step 3 — 设置中心「消息中继」

- `config.local.json` 与 `~/.elwright/config.json` 新增 `messagingRelayUrl` 字段（默认空）
- CLI `ew config messaging` 子命令查看/设置
- 设置中心「消息会话」分组——输入 URL + 「测试连接」按钮（实测握手 round-trip 一次）
- 浏览器预览：URL 校验（合法 ws:// / wss://）+ 保存降级文案

### Step 4 — 客户端 + 中继最小回路

- 客户端 `connect → handshake → send encrypted → 中继 → 对端解密`
- 中继参考实现：`docs/features/messaging/relay/`（axum + tokio-tungstenite，~200 行）
  - `Cargo.toml` + `src/main.rs`
  - `Dockerfile`（多阶段编译）
  - `docker-compose.yml`
  - `README.md`：最低 512MB VPS，200 并发连接
- IPC mock runtime 测试：自起 relay + 两端 mock 客户端，断言明文不出现于 relay 日志

### Step 5 — 离线消息队列

- 本地 SQLite（用现有 `rusqlite` 还是 `sled`，待评估——本阶段先用 `sled` kv 简单持久化）
- 队列里的消息**用同一 secretstream key** 加密暂存
- 对端握手成功后从队列 pop 投递

### Step 6 — 文档回填

- `docs/features/messaging/behavior.md` 加 §第二阶段
- `docs/features/messaging/architecture.md` 替换「MessageTransport」段
- `docs/features/messaging/changelog.md` 加 v0.x.y
- `docs/ROADMAP.md`「进行中」改回 + 「已完成里程碑」加条目
- `docs/ROADMAP.md`「人与人消息会话」条目标记完成（本阶段）

## 验证清单

### 【自动化】

- protocol 层单测（`messaging_transport.rs`）
- IPC mock runtime 测试（identity + relay）
- Playwright e2e：设置中心 URL 校验、邀请流程
- vitest：identity / invite / base32 派生 / QR 解析
- 跨平台 CI：libsodium-sys 在 windows-gnu 工具链下编译通过（verification 必测）

### 【手测】（真机点验）

- 同一台机器双账户：A、B 各自本地身份 → A 生成邀请 → B 扫码 → 握手 → A 发文字 + 图片 → B 收到且 UI 状态 `delivered`
- 离线降级：中继 URL 为空 → 「消息会话」按钮灰显或显示「未配置中继」
- 篡改测试：编辑中继转发的密文一位 → 接收端解密失败且 UI 标记 `failed`
- 离线队列：A 发 → B 不在线 → B 上线 → 消息补投成功
- 中继最小回路：在本地 docker-compose 起来 relay，A/B 通过 relay 互发消息，验证密文不出现于 relay 日志

## 任务目录归档时机

按 AGENTS.md：上线确认后由人执行归档（active → archive），Agent 不自行归档。