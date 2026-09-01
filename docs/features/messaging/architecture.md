# 架构（Architecture）

```text
App.vue
  └─ PeopleChatView.vue
       ├─ 会话列表
       ├─ 消息时间线
       └─ 输入区（文字 / 图片 / 表情）
            └─ localStorage（第一阶段）

UI 接线（2026-09-01，ADR-003，全部落地）：

PeopleChatView.vue  邀请互加弹窗 / 联系人绑定 / 发送状态机（sending→sent|queued|failed）
                    / 3s 收件轮询合并（cursor 持久化 localStorage）/ 预览模式降级
SettingsCenter.vue  「消息中继」分组（MessagingSettings.vue：URL + 保存 + 测试连接）
lib/bridge.ts       +12 方法（桌面 tauriInvoke；浏览器统一中文降级抛错）

第二阶段传输层核心：

src-tauri/src/core/
  ├─ messaging_transport.rs   Noise_XX 握手 + AEAD 帧 + 帧序列化（协议层，12 单测）
  ├─ identity.rs              ed25519 + X25519 身份 / 邀请签发校验（13 单测）
  ├─ messaging_client.rs      中继连通性探测（probe_relay，2 单测）
  ├─ messaging_queue.rs       离线发件箱（outbox.jsonl，只存密文，5 单测）
  └─ commands.rs              6 个 IPC：identity_* ×3 + messaging 配置/测试 ×3

docs/features/messaging/relay/   自托管中继参考实现（axum，纯密文转发，Docker 一键）

src-tauri/tests/
  ├─ messaging_phase2_ipc.rs      IPC 冒烟（7 例）
  └─ messaging_relay_smoke.rs     真实 relay 进程双端握手 + AEAD 收发 + 零明文日志（2 例）
```

- 第一阶段使用前端本地适配器，消息模型与未来远端消息保持独立。
- 消息类型先定义为 `text`、`image`、`emoji`，保留 `status` 字段供后续增加 `sending/sent/delivered/failed`。
- 图片仅保存用户主动选择的内容；不读取剪贴板、终端或其他本地文件。
- 网络传输、身份认证、邀请和附件服务另建 MessageTransport，不塞入 `Bridge` 的现有能力执行接口。
- 传输层按 [ADR-002](./decisions/ADR-002-messaging-transport.md)（协议/身份/队列）与
  [ADR-003](./decisions/ADR-003-messaging-wiring.md)（接线：邀请 v3 DH 硬绑定、成对房间
  按 ID 定角色、sync_peer 收发、离线队列本地密钥加密）落地：身份即 X25519 静态密钥
  （Noise_XX），会话密钥端到端协商，中继只见密文与房间号；收发路径握手后校验
  `remote_static == 联系人 DH 公钥` 防中间人；消息模型仍以阶段① localStorage 会话
  为展示层，收件箱/发件箱在 Rust 侧加密落盘。
