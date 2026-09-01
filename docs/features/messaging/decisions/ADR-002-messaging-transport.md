# ADR-002：消息传输通道 / 身份 / 邀请 / 加密（人与人消息会话阶段②）

- **状态**：已批准（2026-09-01）
- **日期**：2026-09-01
- **Feature**：[messaging](../README.md)
- **接续**：ADR-001 — 客户端本地消息会话（PR #1 已并入 main，行为见 [behavior.md §第一阶段](../behavior.md)）
- **阶段**：三阶段规划之②——「轻量身份/邀请与一对一消息传输」
- **红线对齐**：[ROADMAP §主干红线与准入](../../../ROADMAP.md) — 数据本地优先、用户明确授权、不默认采集屏幕/终端/剪贴板
- **实施偏差（Step 1 后追加）**：原计划用独立 libsodium `secretstream` 流加密（ADR §D4）；实施期调研发现 snow 0.9 TransportState 自带 ChaCha20-Poly1305 AEAD + nonce 单调管理 + `rekey_*` 接口，已覆盖 ADR §D4 的全部安全需求。去掉独立 secretstream 封装，少引 `dryoc` 与 `libsodium-sys` 两个依赖。详细帧协议见 [transport-protocol.md](../transport-protocol.md)。

---

## 背景

阶段①已交付 `PeopleChatView.vue` 与本地消息状态（文字 / 图片 / 表情 / 消息状态字段预留 `sending/sent/delivered/failed`），架构留口：

> 第二阶段：MessageTransport（WebSocket / 中继服务）替换本地投递适配器（[architecture.md §架构图](../architecture.md)）

阶段②要回答四个互相耦合的产品/技术问题——这是 ADR 的全部价值：

1. **传输通道**：P2P 还是中继？自托管还是借服务？
2. **身份**：谁是 Elwright 用户？密钥存在哪？人类怎么识别彼此？
4. **消息加密**：握手用什么？会话用什么？是否需要前向保密？

四个问题的答案同时决定实施工作量（编译产物大小、首版交付时长）、对 NAT/网络的依赖、与红线的张力。

---

## 决定（推荐方案）

### D1 · 传输通道：自托管 WebSocket 中继

- 在仓库附 `docs/features/messaging/relay/` 目录，给一份最小可用中继：axum + tokio-tungstenite，约 200 行 Rust，纯 TCP/WS 转发 + 房间路由元数据，不解包消息内容。
- 客户端通过 **Tauri 设置中心的「消息中继」URL 字段** 接入；URL 默认为空（不启用传输，保留阶段①本地模式）。CLI `ew messaging` 子命令读同一个 URL。
- 中继只看见：
  - 客户端公网 IP / 端口 / 客户端自签 TLS 指纹（连接层元数据）
  - 房间号（一对一房间 = 双方公钥的 SHA-256 派生，**不存任何人类可读 ID**）
  - 密文字节数与时间戳
- 中继不解密、不记录明文、不解析房间语义；房间成员变动由客户端用 Noise 握手协商，中继只做信封路由。

### D2 · 身份：本地 Noise_XX 静态身份 + 一次性短邀请码

- 首次启动在 `~/.elwright/identity/` 生成：
  - `signing.ed25519`（ed25519 长私钥，签名挑战用）
  - `dh.x25519`（X25519 长私钥，Noise_XX 协商用）
  - `id.base32`（X25519 公钥 → SHA-256 → 头 80 bit → 16 字符 base32 Crockford，UI 给人类看）
- **公钥即 ID**。人类互相认识靠「ID + 显示名（本地自由输入）」，不依赖任何第三方服务。设置中心展示「我的 Elwright ID」+ 二维码 + 「复制」按钮。
- 邀请流程（一次性短码，5 分钟有效）：
  1. A 在「添加联系人」点「生成邀请」，本地用 `signing.ed25519` 对 `{from_id, nonce, expires_at}` 签名得到 6 字符 base32 邀请码（含时间戳与签名摘要）。
  2. 邀请码展示为 6 字符短码 + 二维码 + 5 分钟倒计时。
  3. B 在「添加联系人」粘贴短码或扫码，本地校验签名与有效期 → 在中继上对 A 的 `id.base32` 房间发起 Noise_XX 握手。
  4. 双方 Noise 握手成功后，本地把对方公钥加入 `contacts.json`，显示名手动输入。
- 离线邀请：短码本身**就是**带签名的邀请，不依赖中继在线；只是 Noise 握手要等中继可达时完成（手动重试或自动重试）。

### D3 · 中继部署：随仓库附参考实现，不替用户托管

- `docs/features/messaging/relay/` 包含：
  - `Cargo.toml` + `src/main.rs`（axum + tokio-tungstenite）
  - `Dockerfile`（多阶段编译，debian-slim 运行）
  - `docker-compose.yml`（仅服务 + 端口暴露，反代由用户自选 Caddy / Nginx）
  - `README.md`：最低 512MB VPS 即可单实例服务约 200 并发连接
- 仓库**不提供**公共中继实例、不附带托管链接、不在默认设置里填任何 URL。
- 用户的默认状态是「离线、阶段①本地模式」，与 `LLM 配置缺失` 的处理一致——明确降级，不假装能通信。

### D4 · 加密：Noise_XX 握手 + libsodium secretstream 流加密

- 握手：Noise_XX 模式，X25519 + ChaCha20-Poly1305；双方各自签名（ed25519 over handshake transcript）防 MITM。
- 数据通道：握手后派生双向 `send_key` / `recv_key`，用 **libsodium `crypto_secretstream_xchacha20poly1305_ietf`** 流加密：
  - 自带 nonce、AEAD、重放保护、顺序保证
  - 每条消息带 `[tag, ciphertext, nonce]` 帧头
  - 单方向密钥顺序滚动（每 256 条消息显式 rekey，libsodium 默认阈值）
- 离线消息（对方不在线）由客户端在本地 SQLite 队列里**用同一 secretstream key** 加密暂存，对端下次握手成功后从队列里 pop 投递。
- 多设备：v0 不支持（同一身份只有一个本地密钥对）；未来扩展点 = 主密钥加密 + 设备子密钥（DR 不在此 ADR 范围）。
- 依赖：Rust 端 `libsodium-sys`（`aead` / `secretstream` / `sign` / `scalarmult`）+ `snow`（Noise_XX 协议实现）。`libsodium-sys` 通过 `vcpkg` / 系统 libsodium 链接；CI 镜像已包含。

---

## 取舍与弃选

### 通道：WebRTC P2P ❌

- 优点：无中继、最贴近「无服务器」哲学。
- 弃选：家庭 NAT 多数不通，企业网关更不通——至少一方需公网入口；CLI 端 `libdatachannel` 编译体积不小（~1MB 二进制增量 + WebRTC 栈复杂度），首版交付负担过重。中继 + Noise 加密已能把「中继不可信」从威胁模型里移除，P2P 的边际收益不抵部署成本。
- 留后路：阶段③实时协作（共同编辑高频低延迟）若 NAT 仍是大问题，再评估 P2P。

### 身份：GitHub 用户名 claim ❌

- 弃选：绑定第三方账号 + 信任 GitHub 不被封号；与「数据本地优先」红线张力大。Noise 静态公钥即 ID 已经满足「人类互相识别」需求，不引入额外单点。

### 身份：OIDC / 邮箱注册到中继 ❌

- 弃选：违背数据本地优先与用户明确授权；中继不该知道谁是谁——这是 Noise_XX 设计的关键收益。

### 加密：Signal Double Ratchet ❌

- 弃选：libsignal-protocol-rust 体积大、双 Ratchet 状态机复杂、对一对一过度工程。前向保密由 secretstream 的显式 rekey 已能覆盖（且单设备单密钥对下不存在历史密钥泄露面）。群消息、未来多设备再评估 DR。

### 中继：仓库提供公共实例 ❌

- 弃选：公共实例带来合规、滥用防控、长期运维负担；与「个人工作流工具箱」定位不符——用户完全可以在自己 VPS 跑附带的 docker-compose。

---

## 与主干红线的对齐

- **数据本地优先**：密钥永远不出本地（仅公钥在握手时广播给对方）；中继不解密。
- **用户明确授权**：默认无中继 URL（等同无 LLM 配置）；启用需用户在设置中心显式填 URL 并「测试连接」通过。
- **不采集屏幕/终端/剪贴板**：图片仍只接受用户主动选择（同阶段①）；消息内容不进核心 IPC、不进 LLM 上下文。
- **CLI 与桌面共享核心**：中继 URL 与密钥路径通过 `~/.elwright/config.json` 与 `~/.elwright/identity/` 在两壳共享。

---

## 实施切分（本 ADR 通过后启动）

按惯例分两步：

1. **协议层骨架**（在 `core::messaging_transport` 新模块）：
   - Noise_XX 握手 + secretstream 帧协议
   - 帧格式文档化（`docs/features/messaging/transport-protocol.md`）
   - 客户端单元测试：握手 round-trip、密文不可破、nonce 重放拒绝
2. **本地身份 + 邀请 + UI**（在 `core::identity` + IPC + 前端）：
   - 身份生成 / 显示 / 复制
   - 邀请生成 / 短码校验 / 二维码渲染
   - 设置中心「消息中继」URL 字段 + 「测试连接」按钮
   - 「添加联系人」流程接入 PeopleChatView
3. **客户端 + 中继最小回路**：
   - 客户端 `connect → handshake → send encrypted → 中继 → 对端解密`
   - 中继附 `docs/features/messaging/relay/` 实现 + Docker 化
4. **离线消息队列**：本地 SQLite secretstream 加密暂存 + 重连投递
5. **任务目录**：`docs/work/active/feature-2026-09-messaging-phase2/{plan.md, checklist.md, verification.md}`
6. **测试**：protocol 层单测（纯逻辑） + IPC 层 mock runtime（自起中继） + Playwright 浏览器层（设置中心 URL 校验）
7. **文档回填**：behavior §第二阶段 / architecture §传输通道 / changelog / ROADMAP 进行中登记

---

## 风险与未决

- **libsodium-sys 跨平台**：macOS / Linux 走系统库，Windows CI 镜像已含；公司机 Windows GNU 工具链需确认 libsodium 可用——验证步骤加入 verification.md。
- **中继只读用户密钥派生房间号**：若用户对「中继能看见房间哈希」也不接受，则唯一路径是 P2P（见弃选）。此 ADR 默认中继可见密文与房间哈希，密钥与明文永不出端到端。
- **二维码渲染**：阶段①已有图片接收路径；新增 SVG 二维码生成（前端 `qrcode` 包 ~10KB，零额外依赖后可换自写——首版允许 `qrcode` 包）。
- **短码有效期 5 分钟**是个折中——太短不便扫码，太长泄露面大。可配置，默认 300s。
- **多端**：v0 显式不支持，文档明示——同一 ID 多端会覆盖前一个密钥对导致历史消息无法在另一端解密。这是范围裁剪，不是缺陷。

---

## 批准后动作

请用户回复「批准」或具体修改意见；批准后：

- 本 ADR 状态改为「已批准」
- 在 `docs/work/active/feature-2026-09-messaging-phase2/` 建任务目录（plan / checklist / verification）
- ROADMAP「进行中」登记本项
- 开始 D1~D4 的实施切分第 1 步（协议层骨架）