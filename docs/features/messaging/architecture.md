# 架构（Architecture）

```text
App.vue
  └─ PeopleChatView.vue
       ├─ 会话列表
       ├─ 消息时间线
       └─ 输入区（文字 / 图片 / 表情）
            └─ localStorage（第一阶段）

第二阶段：MessageTransport（WebSocket / 中继服务）替换本地投递适配器
```

- 第一阶段使用前端本地适配器，消息模型与未来远端消息保持独立。
- 消息类型先定义为 `text`、`image`、`emoji`，保留 `status` 字段供后续增加 `sending/sent/delivered/failed`。
- 图片仅保存用户主动选择的内容；不读取剪贴板、终端或其他本地文件。
- 网络传输、身份认证、邀请和附件服务另建 MessageTransport，不塞入 `Bridge` 的现有能力执行接口。
