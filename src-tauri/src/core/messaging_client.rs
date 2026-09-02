//! Messaging client — 中继连通性探测（phase 2 step 4）。
//!
//! 与 `messaging_transport`（协议层）分工：本模块负责「连得上吗」的用户可见
//! 探测——设置中心「测试连接」按钮与 `ew config messaging test` 共用。
//! 完整的持久化聊天传输（收发循环、重连、离线队列投递）属 phase 3 范围。

use std::time::Duration;

/// 探测中继可达性：完成 WebSocket 升级后立即关闭。
///
/// 阻塞式 API（与 `llm::test_connection` 同风格）：内部起 current-thread
/// tokio runtime，调用方（tauri spawn_blocking / CLI 主线程）无需异步上下文。
/// 返回带延迟的中文成功文案；失败给可读错误。
pub fn probe_relay(url: &str, timeout: Duration) -> Result<String, String> {
    crate::core::llm::validate_relay_url(url).map_err(|e| format!("URL 非法: {}", e))?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("初始化异步运行时失败: {}", e))?;
    rt.block_on(async {
        let started = std::time::Instant::now();
        let connect = tokio_tungstenite::connect_async(url);
        let (_ws, _resp) = tokio::time::timeout(timeout, connect)
            .await
            .map_err(|_| format!("连接超时（>{}ms）", timeout.as_millis()))?
            .map_err(|e| format!("连接失败: {}", e))?;
        // _ws drop 即关闭连接——探测不参与任何房间路由
        let latency = started.elapsed();
        Ok(format!(
            "已连接（WebSocket 升级成功，耗时 {}ms）",
            latency.as_millis()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_rejects_invalid_url_before_connecting() {
        // 格式非法应直接拒绝，不发起网络
        let r = probe_relay("http://example.com", Duration::from_secs(1));
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("URL 非法"));
    }

    #[test]
    fn probe_reports_unreachable_host_quickly() {
        // 保留端口段（127.0.0.1:1 通常拒绝连接）——期望快速报错而非挂起
        let started = std::time::Instant::now();
        let r = probe_relay("ws://127.0.0.1:1", Duration::from_secs(3));
        assert!(r.is_err());
        assert!(started.elapsed() < Duration::from_secs(3), "应在超时前失败");
    }
}

// ---------- sync_peer：与单个联系人的「连接→握手→验身份→flush→收→关」闭环 ----------
//
// 设计（ADR-003 §D2/D3）：
//   - 成对房间：min(idA,idB)-max(idA,idB)，两端算出同一路径
//   - 角色：ID 小者恒为 initiator——两端同时上线也不撞角色
//   - 握手后校验 remote_static == 联系人 DH 公钥（不符即断开，防中间人）
//   - flush：发件箱按 FIFO 解密 → 新会话重加密投递 → remove；失败保留并记 attempts
//   - 收：Data 帧解密落收件箱，空闲 idle_timeout 即收尾关闭
//
// 阻塞式 API（内部起 current-thread runtime），供 spawn_blocking / listener 线程用。

use std::time::Duration as StdDuration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::core::contacts::Contact;
use crate::core::identity::Identity;
use crate::core::messaging_inbox::Inbox;
use crate::core::messaging_queue::Outbox;
use crate::core::messaging_transport::{Frame, FrameType, Handshake};

/// 本端在一对联系人中的角色（ADR-003 §D2：ID 小者为 initiator）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Initiator,
    Responder,
}

/// 成对房间号：两端独立计算结果一致（Crockford 无大小写歧义，原样拼接）。
pub fn pair_room(my_id: &str, peer_id: &str) -> String {
    if my_id <= peer_id {
        format!("{}-{}", my_id, peer_id)
    } else {
        format!("{}-{}", peer_id, my_id)
    }
}

/// 本端角色。
pub fn role_for(my_id: &str, peer_id: &str) -> Role {
    if my_id <= peer_id {
        Role::Initiator
    } else {
        Role::Responder
    }
}

/// 一次 sync 的结果统计。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub struct SyncReport {
    /// 本次从发件箱成功投递的条数
    pub flushed: usize,
    /// 本次收进收件箱的条数
    pub received: usize,
}

/// sync 参数。
pub struct SyncParams<'a> {
    pub relay_url: &'a str,
    pub identity: &'a Identity,
    pub contact: &'a Contact,
    pub local_key: &'a [u8; 32],
    /// None = 只收不发（listener 收件轮）；Some = 先 flush 再收
    pub outbox: Option<&'a Outbox>,
    pub inbox: &'a Inbox,
    /// 连接与每步握手帧的等待超时
    pub connect_timeout: StdDuration,
    /// 收尾静默超时：这段时间没有新帧即正常关闭
    pub idle_timeout: StdDuration,
}

impl<'a> SyncParams<'a> {
    pub fn new(
        relay_url: &'a str,
        identity: &'a Identity,
        contact: &'a Contact,
        local_key: &'a [u8; 32],
        inbox: &'a Inbox,
    ) -> Self {
        Self {
            relay_url,
            identity,
            contact,
            local_key,
            outbox: None,
            inbox,
            connect_timeout: StdDuration::from_secs(5),
            idle_timeout: StdDuration::from_secs(2),
        }
    }
}

/// 与单个联系人完成一次闭环。任何一步失败返回 Err（发件箱条目原地保留）。
pub fn sync_peer(p: &SyncParams) -> Result<SyncReport, String> {
    crate::core::llm::validate_relay_url(p.relay_url)
        .map_err(|e| format!("中继 URL 非法: {}", e))?;
    let my_id = p.identity.id_base32();
    // 兼容 base（ws://host:9000）与完整（ws://host:9000/ws）两种配置写法
    let base = p.relay_url.trim_end_matches('/');
    let base = base.strip_suffix("/ws").unwrap_or(base);
    let url = format!("{}/ws/{}", base, pair_room(my_id, &p.contact.peer_id));
    let role = role_for(my_id, &p.contact.peer_id);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("初始化异步运行时失败: {}", e))?;
    rt.block_on(async {
        // 1. 连接
        let req = url
            .clone()
            .into_client_request()
            .map_err(|e| format!("中继 URL 无法发起请求: {}", e))?;
        let (ws, _) =
            tokio::time::timeout(p.connect_timeout, tokio_tungstenite::connect_async(req))
                .await
                .map_err(|_| "连接中继超时".to_string())?
                .map_err(|e| format!("连接中继失败: {}", e))?;
        let (mut sink, mut stream) = ws.split();

        // 2. Noise_XX 三步握手（显式按角色展开——responder 末步必须 read_final）
        let dh_secret = p.identity.dh_secret_bytes();
        let mut hs = match role {
            Role::Initiator => Handshake::new_initiator(dh_secret),
            Role::Responder => Handshake::new_responder(dh_secret),
        }
        .map_err(|e| format!("初始化握手失败: {}", e))?;
        let mut transport = match role {
            Role::Initiator => {
                let msg1 = hs.step(&[]).map_err(|e| format!("写 msg1 失败: {}", e))?;
                send_frame(&mut sink, &Frame::encode_handshake(&msg1)).await?;
                let msg2 = recv_handshake(&mut stream, p.connect_timeout).await?;
                let msg3 = hs
                    .step(&msg2)
                    .map_err(|e| format!("处理 msg2 失败: {}", e))?;
                send_frame(&mut sink, &Frame::encode_handshake(&msg3)).await?;
                hs.into_transport()
                    .map_err(|e| format!("握手未完成: {}", e))?
            }
            Role::Responder => {
                let msg1 = recv_handshake(&mut stream, p.connect_timeout).await?;
                let msg2 = hs
                    .step(&msg1)
                    .map_err(|e| format!("处理 msg1 失败: {}", e))?;
                send_frame(&mut sink, &Frame::encode_handshake(&msg2)).await?;
                let msg3 = recv_handshake(&mut stream, p.connect_timeout).await?;
                hs.read_final(&msg3)
                    .map_err(|e| format!("处理 msg3 失败: {}", e))?;
                hs.into_transport()
                    .map_err(|e| format!("握手未完成: {}", e))?
            }
        };

        // 3. 身份校验：remote_static 必须等于联系人 DH 公钥（防中间人）
        let expected =
            hex::decode(&p.contact.dh_pub_hex).map_err(|_| "联系人 DH 公钥损坏".to_string())?;
        match transport.remote_static() {
            Some(actual) if actual == expected.as_slice() => {}
            _ => return Err("对端身份校验失败（DH 公钥与联系人不符）".to_string()),
        }

        let mut report = SyncReport::default();

        // 4. flush 发件箱（FIFO；失败保留条目 + attempts，中止本次 flush）
        if let Some(outbox) = p.outbox {
            let entries = outbox
                .list(Some(&p.contact.peer_id))
                .map_err(|e| format!("读发件箱失败: {}", e))?;
            for entry in entries {
                let plaintext = match outbox.decrypt_payload(p.local_key, &entry) {
                    Ok(b) => b,
                    Err(e) => return Err(format!("发件箱条目解密失败: {}", e)),
                };
                let ct = match transport.send(&plaintext) {
                    Ok(ct) => ct,
                    Err(_) => {
                        let _ = outbox.record_attempt(entry.id);
                        break;
                    }
                };
                match sink.send(WsMessage::Binary(Frame::encode_data(&ct))).await {
                    Ok(()) => {
                        let _ = outbox.remove(entry.id);
                        report.flushed += 1;
                    }
                    Err(_) => {
                        let _ = outbox.record_attempt(entry.id);
                        break;
                    }
                }
            }
        }

        // 5. 收对端 Data 帧直到空闲超时/断开
        loop {
            let frame = tokio::time::timeout(p.idle_timeout, stream.next()).await;
            let msg = match frame {
                Err(_) => break,           // 空闲超时 = 正常收尾
                Ok(None) => break,         // 连接关闭
                Ok(Some(Err(_))) => break, // 读错误
                Ok(Some(Ok(m))) => m,
            };
            if let WsMessage::Binary(b) = msg {
                let (ft, hdr) = match Frame::parse_header(&b) {
                    Ok(x) => x,
                    Err(_) => continue, // 坏帧丢弃，不中断本轮
                };
                if ft != FrameType::Data {
                    continue;
                }
                match transport.recv(&b[hdr..]) {
                    Ok(plain) => {
                        let text = match String::from_utf8(plain) {
                            Ok(t) => t,
                            Err(_) => continue, // 非文本负载 MVP 不支持
                        };
                        if p.inbox
                            .append(p.local_key, &p.contact.peer_id, &text)
                            .is_ok()
                        {
                            report.received += 1;
                        }
                    }
                    Err(_) => continue, // 解密失败（篡改/乱序）丢弃
                }
            }
        }

        let _ = sink.close().await;
        Ok(report)
    })
}

async fn send_frame(
    sink: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        WsMessage,
    >,
    frame: &[u8],
) -> Result<(), String> {
    sink.send(WsMessage::Binary(frame.to_vec()))
        .await
        .map_err(|e| format!("发送帧失败: {}", e))
}

async fn recv_handshake(
    stream: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    timeout: StdDuration,
) -> Result<Vec<u8>, String> {
    loop {
        let msg = tokio::time::timeout(timeout, stream.next())
            .await
            .map_err(|_| "等待握手帧超时".to_string())?
            .ok_or_else(|| "连接已关闭".to_string())?
            .map_err(|e| format!("读握手帧失败: {}", e))?;
        match msg {
            WsMessage::Binary(b) => {
                let (ft, hdr) =
                    Frame::parse_header(&b).map_err(|e| format!("帧解析失败: {}", e))?;
                if ft == FrameType::Handshake {
                    return Ok(b[hdr..].to_vec());
                }
                continue; // 数据帧不应出现在握手期，忽略
            }
            WsMessage::Close(_) => return Err("握手期间连接已关闭".to_string()),
            _ => continue,
        }
    }
}

// ---------- listener 后台线程 + 全局同步锁 ----------

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn global_sync_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// 串行化对同一批联系人文件的 sync 访问（listener 线程与发送 IPC 并发时，
/// 同房间出现 >2 连接会导致 nonce 乱序解密失败——MVP 用全局锁串行）。
pub fn acquire_sync_lock() -> MutexGuard<'static, ()> {
    global_sync_lock().lock().unwrap_or_else(|e| e.into_inner())
}

static LISTENER_STARTED: AtomicBool = AtomicBool::new(false);

/// 启动后台 listener 线程（幂等；重复调用返回 false）。
///
/// 线程每轮：读中继配置（未配置长睡）→ 加载身份/本地密钥/联系人 →
/// 逐个 `sync_peer`（带发件箱 flush，实现对端上线后的队列补投）→ 睡 5s。
/// 所有错误静默吞掉（个人应用 MVP；网络异常下轮重试）。
pub fn ensure_listener_started() -> bool {
    if LISTENER_STARTED.swap(true, Ordering::SeqCst) {
        return false;
    }
    std::thread::Builder::new()
        .name("messaging-listener".into())
        .spawn(|| loop {
            let relay = crate::core::llm::read_messaging_relay_url();
            if relay.is_empty() {
                std::thread::sleep(StdDuration::from_secs(15));
                continue;
            }
            let Some(user_root) = crate::core::identity::user_root() else {
                std::thread::sleep(StdDuration::from_secs(30));
                continue;
            };
            let run = || -> Result<(), String> {
                let id_dir =
                    crate::core::identity::default_user_identity_dir().ok_or("主目录不可定位")?;
                let identity = Identity::load_or_create(&id_dir)
                    .map_err(|e| format!("身份加载失败: {}", e))?;
                let local_key = crate::core::identity::load_or_create_local_key(&id_dir)
                    .map_err(|e| format!("本地密钥加载失败: {}", e))?;
                let msg_dir = user_root.join("messaging");
                let outbox = Outbox::open(&msg_dir).map_err(|e| e.to_string())?;
                let inbox = Inbox::open(&msg_dir).map_err(|e| e.to_string())?;
                let contacts =
                    crate::core::contacts::list(&user_root).map_err(|e| e.to_string())?;
                for contact in contacts {
                    let _guard = acquire_sync_lock();
                    let mut params =
                        SyncParams::new(&relay, &identity, &contact, &local_key, &inbox);
                    params.outbox = Some(&outbox);
                    let _ = sync_peer(&params);
                }
                Ok(())
            };
            let _ = run();
            std::thread::sleep(StdDuration::from_secs(5));
        })
        .ok();
    true
}
