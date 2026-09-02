//! Elwright 消息中继（参考实现）
//!
//! 协议见 `../transport-protocol.md` 与 ADR-002。
//! 核心安全前提：**中继只看见密文与路由元数据（房间 ID），永远不解密业务数据。**
//!
//! 路由规则（v0，最小可用）：
//! - 客户端连接时 URL 形如 `ws://host:port/ws/<room_id>`
//! - 同房间内任意一方发来帧（type 0 握手 / type 1 数据 / type 2 控制），全部转发给其他成员
//! - 单成员房间暂存 64 帧，凑够第二个客户端即投递；超时 30s 清空
//! - 此参考实现不做持久化——重启即丢未投递消息
//!
//! **绝不记录载荷字节**——日志只输出房间 ID、连接数、字节数。
//! 这是「明文不出现于 relay 日志」验证清单的代码层落实。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State as AxumState,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:9000".into());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .expect("绑定端口失败");
    let shared: Shared = Arc::new(Mutex::new(RoomMap::default()));
    let janitor_shared = shared.clone();
    tokio::spawn(async move { janitor(janitor_shared).await });
    log::info!("elwright-relay 监听 {}", bind);
    axum::serve(
        listener,
        Router::new()
            .route("/ws/:room_id", get(ws_handler))
            .with_state(shared)
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("axum 服务异常退出");
}

type Shared = Arc<Mutex<RoomMap>>;

#[derive(Default)]
struct RoomMap {
    rooms: HashMap<String, Room>,
}

struct Room {
    live: HashMap<ConnectionId, TxHalf>,
    buffered: Vec<BufferedFrame>,
    empty_since: Option<Instant>,
}

/// 一端的发送半（Sink 包装在 Mutex 里供广播循环复用）。
type TxHalf = Arc<Mutex<futures_util::stream::SplitSink<WebSocket, Message>>>;

struct BufferedFrame {
    payload: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct ConnectionId(u64);

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "conn#{}", self.0)
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AxumState(shared): AxumState<Shared>,
    Path(room_id): Path<String>,
) -> impl IntoResponse {
    log::info!("{} 连接房间 '{}'", addr, room_id);
    ws.on_upgrade(move |socket| client_loop(socket, shared, room_id))
}

async fn client_loop(socket: WebSocket, shared: Shared, room_id: String) {
    let (sink, mut stream) = socket.split();
    let tx: TxHalf = Arc::new(Mutex::new(sink));
    let conn_id = ConnectionId(next_conn_id());

    // 注册到房间
    {
        let mut map = shared.lock().await;
        let room = map.rooms.entry(room_id.clone()).or_insert_with(|| Room {
            live: HashMap::new(),
            buffered: Vec::new(),
            empty_since: None,
        });
        room.live.insert(conn_id, tx.clone());
        room.empty_since = None;
        log::info!(
            "房间 '{}' 新成员 {:#x}（在线 {}）",
            room_id,
            conn_id.0,
            room.live.len()
        );
        // 房间内有 ≥2 人 → 把积压帧全部投递（除了自己）
        flush_buffered_to_newcomer(room, conn_id);
    }

    // 收包循环
    while let Some(msg_res) = stream.next().await {
        let msg = match msg_res {
            Ok(m) => m,
            Err(e) => {
                log::warn!("读 WS 失败 {:#x}: {}", conn_id.0, e);
                break;
            }
        };
        let payload = match msg {
            Message::Binary(b) => b,
            Message::Close(_) => {
                log::info!("{} 收到 Close 帧", conn_id);
                break;
            }
            // 文本/ping/pong 不参与转发
            other => {
                log::debug!("忽略非二进制帧 {:?}", other);
                continue;
            }
        };
        log::debug!(
            "转发 {} 字节，{} → room '{}'",
            payload.len(),
            conn_id,
            room_id
        );
        forward_payload(&shared, &room_id, conn_id, payload).await;
    }

    // 清理
    {
        let mut map = shared.lock().await;
        if let Some(room) = map.rooms.get_mut(&room_id) {
            room.live.remove(&conn_id);
            log::info!(
                "房间 '{}' 成员 {:#x} 断开（剩余 {}）",
                room_id,
                conn_id.0,
                room.live.len()
            );
            if room.live.is_empty() {
                room.empty_since = Some(Instant::now());
            }
        }
    }
    // 关闭 sink
    let mut tx = tx.lock().await;
    let _ = tx.close().await;
}

async fn forward_payload(shared: &Shared, room_id: &str, from: ConnectionId, payload: Vec<u8>) {
    let mut map = shared.lock().await;
    let Some(room) = map.rooms.get_mut(room_id) else {
        return;
    };
    if room.live.len() >= 2 {
        // 直接转发给所有非发送方
        let targets: Vec<TxHalf> = room
            .live
            .iter()
            .filter(|(id, _)| **id != from)
            .map(|(_, tx)| tx.clone())
            .collect();
        for tx in targets {
            let mut sink = tx.lock().await;
            if let Err(e) = sink.send(Message::Binary(payload.clone())).await {
                log::warn!("写入对端失败: {}", e);
            }
        }
    } else {
        // 仅自己：暂存
        if room.buffered.len() >= 64 {
            log::warn!("房间 '{}' 暂存已满 64 帧，丢弃最旧", room_id);
            room.buffered.remove(0);
        }
        room.buffered.push(BufferedFrame { payload });
    }
}

fn flush_buffered_to_newcomer(room: &mut Room, newcomer: ConnectionId) {
    if room.live.len() < 2 || room.buffered.is_empty() {
        return;
    }
    let tx = match room.live.get(&newcomer) {
        Some(t) => t.clone(),
        None => return,
    };
    let drained: Vec<Vec<u8>> = room.buffered.drain(..).map(|f| f.payload).collect();
    let tx_clone = tx.clone();
    // 异步发送：在外部锁外执行，避免 send 时持房间锁
    tokio::spawn(async move {
        let mut sink = tx_clone.lock().await;
        for payload in drained {
            if let Err(e) = sink.send(Message::Binary(payload)).await {
                log::warn!("投递积压帧失败: {}", e);
            }
        }
    });
}

fn next_conn_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(1);
    SEQ.fetch_add(1, Ordering::Relaxed)
}

/// 周期清理 30s 无人的房间（防止 map 无限增长）。
async fn janitor(shared: Shared) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        let mut map = shared.lock().await;
        let stale: Vec<String> = map
            .rooms
            .iter()
            .filter_map(|(id, room)| {
                if room.live.is_empty()
                    && room
                        .empty_since
                        .map(|t| t.elapsed() > Duration::from_secs(30))
                        .unwrap_or(false)
                {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        for id in &stale {
            map.rooms.remove(id);
            log::info!("清理空房间 '{}'", id);
        }
    }
}