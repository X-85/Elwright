# 人与人消息会话（Messaging）

> 状态：**第一阶段进行中**。这是 Elwright 的轻量工作沟通入口，不是通用社交软件。

消息会话用于和另一位 Elwright 用户快速讨论工作事项；当讨论需要共同修改流程图、代码或工作上下文时，再升级为实时协作空间。

## 路线

1. **消息会话客户端基础**：独立入口、会话列表、文字/图片/表情、本地消息状态。
2. **消息传输服务**：身份、邀请、一对一实时投递、离线消息和多端同步。
3. **实时协作空间**：从消息会话升级，共享流程图、代码和关联上下文。

第一阶段不伪造跨设备投递；当前客户端消息保存在本地，网络传输在第二阶段接入服务后启用。

## 代码位置

- `src/components/PeopleChatView.vue`：消息会话界面和本地消息状态。
- `src/App.vue`：人与人消息入口。

## 相关文档

- [behavior.md](./behavior.md)
- [architecture.md](./architecture.md)
- [changelog.md](./changelog.md)
- [当前任务](../../work/active/feature-2026-08-messaging-phase1/plan.md)
