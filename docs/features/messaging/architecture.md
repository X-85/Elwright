# 架构（Architecture）

```text
App.vue
  └─ PeopleChatView.vue
       ├─ 会话列表
       ├─ 消息时间线
       └─ 输入区（文字 / 图片 / 表情）
            └─ localStorage（第一阶段）

第二阶段传输层核心（已落地，UI 接线待做）：

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
- 第二阶段传输层原语已按 [ADR-002](./decisions/ADR-002-messaging-transport.md) 落地：
  身份即 X25519 静态密钥（Noise_XX），会话密钥端到端协商，中继只见密文与房间哈希；
  离线队列只存密文。PeopleChatView 从本地适配器切到真实传输时，仅需在
  `Bridge` 增加传输方法并接线（见 behavior.md §本阶段边界）。
