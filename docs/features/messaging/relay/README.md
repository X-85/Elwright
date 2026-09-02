# elwright-relay

Elwright 消息中继的**参考实现**——一个自托管的 WebSocket 服务，仅按房间 ID 转发密文，
**不解密、不存储业务数据**。

> 对应协议与设计取舍见
> [`../transport-protocol.md`](../transport-protocol.md) 与
> [`../decisions/ADR-002-messaging-transport.md`](../decisions/ADR-002-messaging-transport.md)。

## 安全前提

- 中继是**纯密文路由器**：客户端之间已用 Noise_XX 协商出 session key，
  AEAD 加密在客户端完成。中继看到的全部内容都是 snow `TransportState` 输出的 ciphertext。
- **中继日志绝不打印载荷字节**——日志只输出房间 ID、连接数、字节统计。
- 中继不持久化——重启即丢未投递消息（生产部署按需扩展）。

## 路由

- URL 形如 `ws://host:port/ws/<room_id>`
- 房间 ID 是公开路由元数据（用于转发决策）——它**不是密钥**，不暴露业务内容。
- 同房间内任意一方发来帧（type 0/1/2 全部转发），全部广播给其他成员。
- 单成员房间暂存 64 帧；第二个客户端连接时全部投递；30s 空房清理。

## 部署

### 直接运行

```bash
BIND_ADDR=0.0.0.0:9000 cargo run --release
```

### Docker

```bash
docker build -t elwright-relay .
docker run -d --name elwright-relay -p 9000:9000 elwright-relay
```

或：

```bash
docker compose up -d
```

### 反向代理 + TLS

生产部署**必须**套一层反向代理（Caddy / Nginx）做 TLS 终止。
客户端用 `wss://` 连接到反向代理。中继本身只听明文 WS。

## 资源

最低 **512 MB VPS**、单核即可支撑 200 并发连接（中继不读写密文，瓶颈在 axum tokio 调度）。
更高并发按比例升 CPU。

## 自检

`src-tauri/tests/messaging_relay_smoke.rs` 会启动本 relay 自起一个实例 + 两端 mock 客户端，
跑完整的 handshake → AEAD round-trip → 关闭流程，并断言：

- 两端 session key 匹配
- 解密后的明文与原始明文一致
- relay 的 stderr **不包含**明文字节

## 不在本参考实现

- TLS / 反向代理配置（部署层）
- 鉴权 / 房间 ACL（默认路由完全开放，生产需在前置代理层加白名单/认证）
- 持久化（重启丢消息）
- 多区域转发、负载均衡
- 监控 / metrics（按需接入 prometheus exporter）

这些扩展点都在 ADR-002「弃选 / 待评估」清单里——如需实施请走新 ADR。