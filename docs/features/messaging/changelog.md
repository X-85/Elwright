# 变更日志（Changelog）

## 2026-09-01 · 第二阶段传输层核心（ADR-002）

- 本地身份：`~/.elwright/identity/` 密钥对（ed25519 + X25519）与 16 字符 base32 ID 派生。
- 邀请互加：6 字符短码 + v2 二维码原文（签名 + 有效期），`accept_invite` 全量校验。
- 传输协议：Noise_XX_25519_ChaChaPoly_SHA256（snow），AEAD 数据帧 + 控制帧，
  帧格式文档化（transport-protocol.md）；重放拒绝、篡改即失败。
- 中继参考实现：`docs/features/messaging/relay/`（axum 纯密文转发 + Docker 部署），
  自动化断言中继日志零明文。
- 离线队列：`outbox.jsonl` 只存密文，FIFO + 损坏行容忍 + 原子重写。
- 配置与探测：`messaging_relay_url` 字段（设置中心 / `ew config messaging` 共用）、
  `test_messaging_relay` IPC 与 CLI 连通性探测。
- 测试：120 lib 单测 + 7 例 IPC 冒烟 + 2 例真实中继端到端冒烟。
- 默认不启用：未配置中继时无任何网络行为。
- 邀请 v3：QR 增载对端 X25519 DH 公钥并校验「ID==派生(DH 公钥)」硬绑定（ADR-003 §D1）。
- UI 接线（ADR-003，同日完成）：设置中心「消息中继」分组（URL/保存/测试连接）；
  PeopleChatView 邀请互加 + 联系人绑定 + 发送状态机（已送达/离线暂存补投/失败）+
  3 秒收件轮询合并；预览模式统一降级。端到端闭环：自建中继即可真实互发。

## 2026-08-23 · 第一阶段客户端基础

- 新增人与人消息会话入口。
- 支持本地文字、图片和表情消息，会话刷新后保留。
- 明确标记网络消息服务尚未接入，为第二阶段传输服务预留边界。
- 将系统原生输入弹窗替换为应用内新建会话面板，确保浏览器和桌面 WebView 都有可见、可操作的创建流程。
