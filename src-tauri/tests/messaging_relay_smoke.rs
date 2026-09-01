//! Messaging phase 2 step 4 — 中继冒烟测试。
//!
//! 起 relay 二进制 + 两端 mock 客户端，完整跑：
//!   1. WebSocket 握手
//!   2. Noise_XX 三步协商（snow）：Alice 发起 → Bob 响应 → Alice 完成
//!   3. AEAD 加解密 round-trip（snow TransportState）
//!   4. 一端收到对端的密文帧，解密后内容正确
//!   5. relay 子进程 stderr 截取，断言**不包含**明文片段
//!
//! 验证清单里的「明文不出现于 relay 日志」在这里以代码形式落地。

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};

use elwright_core::core::messaging_transport::{complete_handshake, Frame, FrameType, Handshake};

fn find_relay_binary() -> Option<std::path::PathBuf> {
    // CARGO_MANIFEST_DIR = .../Elwright/src-tauri；仓库根 = ../（一层即到）
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        crate_dir.join("../docs/features/messaging/relay/target/release/elwright-relay"),
        crate_dir.join("../docs/features/messaging/relay/target/debug/elwright-relay"),
    ];
    candidates
        .into_iter()
        .find(|p| p.exists())
        .map(|p| p.canonicalize().unwrap_or(p))
}

fn spawn_relay(port: u16) -> Child {
    let bin = find_relay_binary().expect(
        "未找到 elwright-relay 可执行文件——先 cd docs/features/messaging/relay && cargo build",
    );
    let mut cmd = Command::new(bin);
    cmd.env("BIND_ADDR", format!("127.0.0.1:{}", port))
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("启动 relay 失败");
    // 轮询等 relay 真正进入 LISTEN，避免竞速
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", port).parse().unwrap(),
            Duration::from_millis(100),
        )
        .is_ok()
        {
            return child;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("relay 在 3s 内未进入 LISTEN");
}

fn pick_free_port() -> u16 {
    // 用 0 让 OS 选，然后 close 后该端口可能被复用——v0 测试接受小概率冲突；
    // CI 重试间隔足够长。
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

async fn run_initiator(
    url: &str,
    role: &str,
    static_secret: [u8; 32],
    outbound_msg: Vec<u8>,
) -> Vec<u8> {
    let req = url.into_client_request().unwrap();
    let (mut ws, _) = tokio_tungstenite::connect_async(req).await.unwrap();
    eprintln!("[{}] WS 已连接（initiator）", role);

    // Alice（initiator）：写 msg1，等 msg2，写 msg3
    let mut hs = Handshake::new_initiator(static_secret).expect("handshake init");
    let msg1 = hs.step(&[]).expect("msg1");
    ws.send(Message::Binary(Frame::encode_handshake(&msg1)))
        .await
        .unwrap();
    eprintln!("[{}] → msg1 ({} B)", role, msg1.len());

    // 等 msg2
    let msg2_frame = loop {
        match ws.next().await {
            Some(Ok(Message::Binary(b))) => {
                let (ft, hdr_len) = Frame::parse_header(&b).expect("解析 msg2 帧");
                assert_eq!(ft, FrameType::Handshake);
                break b[hdr_len..].to_vec();
            }
            Some(Ok(_)) => continue,
            other => panic!("[{}] 收到非预期消息: {:?}", role, other),
        }
    };
    let msg3 = hs.step(&msg2_frame).expect("msg3");
    ws.send(Message::Binary(Frame::encode_handshake(&msg3)))
        .await
        .unwrap();
    eprintln!("[{}] → msg3 ({} B)", role, msg3.len());

    let mut transport = hs.into_transport().expect("进入 transport 失败");
    eprintln!("[{}] 握手完成", role);

    // initiator 先发自己的密文（与 responder 的先收后发错开，避免互等死锁）
    let cipher = transport.send(&outbound_msg).expect("加密失败");
    ws.send(Message::Binary(Frame::encode_data(&cipher)))
        .await
        .unwrap();
    eprintln!("[{}] → 加密 {} B 明文", role, outbound_msg.len());

    // 再收对端 data 帧（Bob 的 ciphertext）
    let inbound = loop {
        match ws.next().await {
            Some(Ok(Message::Binary(b))) => {
                let (ft, hdr_len) = Frame::parse_header(&b).expect("解析 data 帧");
                if ft == FrameType::Data {
                    let plaintext = transport.recv(&b[hdr_len..]).expect("解密失败");
                    break plaintext;
                }
                continue;
            }
            Some(Ok(_)) => continue,
            other => panic!("[{}] 等待密文时收到非预期: {:?}", role, other),
        }
    };

    let _ = ws.send(Message::Close(None)).await;
    inbound
}

async fn run_responder(
    url: &str,
    role: &str,
    static_secret: [u8; 32],
    outbound_msg: Vec<u8>,
) -> Vec<u8> {
    let req = url.into_client_request().unwrap();
    let (mut ws, _) = tokio_tungstenite::connect_async(req).await.unwrap();
    eprintln!("[{}] WS 已连接（responder）", role);

    // Bob（responder）：等 msg1，写 msg2，等 msg3
    let mut hs = Handshake::new_responder(static_secret).expect("handshake init");
    let msg1_frame = loop {
        match ws.next().await {
            Some(Ok(Message::Binary(b))) => {
                let (ft, hdr_len) = Frame::parse_header(&b).expect("解析 msg1 帧");
                assert_eq!(ft, FrameType::Handshake);
                break b[hdr_len..].to_vec();
            }
            Some(Ok(_)) => continue,
            other => panic!("[{}] 收到非预期消息: {:?}", role, other),
        }
    };
    let msg2 = hs.step(&msg1_frame).expect("msg2");
    ws.send(Message::Binary(Frame::encode_handshake(&msg2)))
        .await
        .unwrap();
    eprintln!("[{}] → msg2 ({} B)", role, msg2.len());

    // 等 msg3（responder 读完最后一步才能 into_transport）
    let msg3_frame = loop {
        match ws.next().await {
            Some(Ok(Message::Binary(b))) => {
                let (ft, hdr_len) = Frame::parse_header(&b).expect("解析 msg3 帧");
                assert_eq!(ft, FrameType::Handshake);
                break b[hdr_len..].to_vec();
            }
            Some(Ok(_)) => continue,
            other => panic!("[{}] 收到非预期消息: {:?}", role, other),
        }
    };
    hs.read_final(&msg3_frame).expect("read msg3");
    let mut transport = hs.into_transport().expect("进入 transport 失败");
    eprintln!("[{}] 握手完成", role);

    // 收对端 data 帧（Alice 的 ciphertext）
    let inbound = loop {
        match ws.next().await {
            Some(Ok(Message::Binary(b))) => {
                let (ft, hdr_len) = Frame::parse_header(&b).expect("解析 data 帧");
                if ft == FrameType::Data {
                    let plaintext = transport.recv(&b[hdr_len..]).expect("解密失败");
                    break plaintext;
                }
                continue;
            }
            Some(Ok(_)) => continue,
            other => panic!("[{}] 等待密文时收到非预期: {:?}", role, other),
        }
    };

    let cipher = transport.send(&outbound_msg).expect("加密失败");
    ws.send(Message::Binary(Frame::encode_data(&cipher)))
        .await
        .unwrap();
    eprintln!("[{}] → 加密 {} B 明文", role, outbound_msg.len());

    let _ = ws.send(Message::Close(None)).await;
    inbound
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn relay_round_trip_two_clients_exchange_ciphertext() {
    // 端到端夹具是预编译的 relay 二进制：CI 有专门构建步；本地缺二进制时
    // 优雅跳过（纯协议层回环仍由 complete_handshake_direct_loop_works 覆盖）
    if find_relay_binary().is_none() {
        eprintln!(
            "跳过中继端到端：未找到 elwright-relay 二进制（cd docs/features/messaging/relay && cargo build --release 后重跑）"
        );
        return;
    }
    let port = pick_free_port();
    let mut relay = spawn_relay(port);
    let url = format!("ws://127.0.0.1:{}/ws/test-room", port);
    eprintln!("中继 URL：{}", url);

    // 两端静态密钥（伪随机即可——测试不验 ID，只验加解密）
    let alice_static: [u8; 32] = std::array::from_fn(|i| (i as u8).wrapping_mul(7));
    let bob_static: [u8; 32] = std::array::from_fn(|i| (i as u8).wrapping_add(13));

    // 客户端 A 发 + 客户端 B 发（两个独立任务并发）
    let url_a = url.clone();
    let url_b = url.clone();
    let alice_msg = b"hello-from-alice (this plaintext must never appear in relay stderr)".to_vec();
    let bob_msg = b"hello-from-bob (this plaintext must never appear in relay stderr)".to_vec();

    let task_a =
        tokio::spawn(async move { run_initiator(&url_a, "alice", alice_static, alice_msg).await });
    let task_b =
        tokio::spawn(async move { run_responder(&url_b, "bob", bob_static, bob_msg).await });

    let alice_in = task_a.await.unwrap();
    let bob_in = task_b.await.unwrap();

    assert_eq!(
        alice_in,
        b"hello-from-bob (this plaintext must never appear in relay stderr)".to_vec()
    );
    assert_eq!(
        bob_in,
        b"hello-from-alice (this plaintext must never appear in relay stderr)".to_vec()
    );

    // 关闭 relay 并捕获日志
    let _ = relay.kill();
    let output = relay.wait_with_output().expect("wait relay");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("hello-from-alice"),
        "relay stderr 不应出现明文片段：\n{}",
        stderr
    );
    assert!(
        !stderr.contains("hello-from-bob"),
        "relay stderr 不应出现明文片段：\n{}",
        stderr
    );
}

#[test]
fn complete_handshake_direct_loop_works() {
    // 烟雾：纯本地两端 snow 握手 + AEAD（不依赖中继，确认 messaging_transport API 易用）
    let alice = [1u8; 32];
    let bob = [2u8; 32];
    let (mut at, mut bt) = complete_handshake(alice, bob).expect("snow 握手失败");
    let msg = b"ping";
    let cipher = at.send(msg).unwrap();
    let plain = bt.recv(&cipher).unwrap();
    assert_eq!(plain, msg);
    let back = bt.send(b"pong").unwrap();
    let plain = at.recv(&back).unwrap();
    assert_eq!(plain, b"pong");
}

// ---------- 双身份全链路（ADR-003 接线闭环）：v3 邀请互加 → 发件箱 flush → 收件箱 ----------

use elwright_core::core::contacts::{self, Contact};
use elwright_core::core::identity::{self, Identity};
use elwright_core::core::messaging_client::{pair_room, role_for, sync_peer, SyncParams};
use elwright_core::core::messaging_inbox::Inbox;
use elwright_core::core::messaging_queue::Outbox;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_identities_full_loop_via_relay() {
    if find_relay_binary().is_none() {
        eprintln!("跳过：elwright-relay 二进制不存在");
        return;
    }
    let port = pick_free_port();
    let mut relay = spawn_relay(port);
    let relay_url = format!("ws://127.0.0.1:{}/ws", port);

    // 两个独立身份（各自临时目录）
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();
    let id_a = Identity::generate().unwrap();
    let id_b = Identity::generate().unwrap();
    id_a.persist(dir_a.path()).unwrap();
    id_b.persist(dir_b.path()).unwrap();
    let key_a = identity::load_or_create_local_key(dir_a.path()).unwrap();
    let key_b = identity::load_or_create_local_key(dir_b.path()).unwrap();

    // v3 邀请互加（练习签名 + ID-DH 硬绑定 + 联系人落盘）
    let invite_b = id_b.create_invite(300).unwrap();
    let inbound_b = identity::parse_invite_qr(&invite_b.qr_payload).unwrap();
    let contact_in_a = Contact {
        peer_id: inbound_b.inviter_id.clone(),
        signing_pub_hex: inbound_b.inviter_signing_pub_hex.clone(),
        dh_pub_hex: inbound_b.inviter_dh_pub_hex.clone(),
        alias: String::new(),
        added_at: 0,
    };
    contacts::add(dir_a.path(), contact_in_a.clone()).unwrap();

    let invite_a = id_a.create_invite(300).unwrap();
    let inbound_a = identity::parse_invite_qr(&invite_a.qr_payload).unwrap();
    let contact_in_b = Contact {
        peer_id: inbound_a.inviter_id.clone(),
        signing_pub_hex: inbound_a.inviter_signing_pub_hex.clone(),
        dh_pub_hex: inbound_a.inviter_dh_pub_hex.clone(),
        alias: String::new(),
        added_at: 0,
    };
    contacts::add(dir_b.path(), contact_in_b.clone()).unwrap();

    // A 的发件箱：两条待发消息（离线暂存场景）
    let outbox_a = Outbox::open(&dir_a.path().join("messaging")).unwrap();
    let inbox_a = Inbox::open(&dir_a.path().join("messaging")).unwrap();
    let _ = &inbox_a; // 保留外部句柄语义；闭包内已自开句柄
    outbox_a
        .enqueue(&key_a, &contact_in_a.peer_id, b"msg-1-for-b")
        .unwrap();
    outbox_a
        .enqueue(
            &key_a,
            &contact_in_a.peer_id,
            "第二条：中文消息 ✓".as_bytes(),
        )
        .unwrap();

    let inbox_b = Inbox::open(&dir_b.path().join("messaging")).unwrap();

    // 双端并发 sync（角色由 ID 决定；两端同时上线也要能完成握手）
    let relay_a = relay_url.clone();
    let relay_b = relay_url.clone();
    let a_id = id_a.clone();
    let b_id = id_b.clone();
    let ca = contact_in_a.clone();
    let cb = contact_in_b.clone();
    // TempDir 不能被 move 进闭包（任务结束会连带删目录）——只搬路径
    let path_a = dir_a.path().to_path_buf();
    let path_b = dir_b.path().to_path_buf();
    let t_a = tokio::task::spawn_blocking(move || {
        // 容器只是路径包装——闭包内开自己的句柄，外部句柄留给断言
        let outbox = Outbox::open(&path_a.join("messaging")).unwrap();
        let inbox = Inbox::open(&path_a.join("messaging")).unwrap();
        let mut params = SyncParams::new(&relay_a, &a_id, &ca, &key_a, &inbox);
        params.outbox = Some(&outbox);
        sync_peer(&params)
    });
    let t_b = tokio::task::spawn_blocking(move || {
        let inbox = Inbox::open(&path_b.join("messaging")).unwrap();
        let params = SyncParams::new(&relay_b, &b_id, &cb, &key_b, &inbox);
        sync_peer(&params)
    });
    let report_a = t_a.await.unwrap().expect("A sync 应成功");
    let _report_b = t_b.await.unwrap().expect("B sync 应成功");

    // A 的发件箱被 flush 清空
    assert_eq!(
        outbox_a.list(Some(&contact_in_a.peer_id)).unwrap().len(),
        0,
        "A 发件箱应全部投递"
    );
    assert_eq!(report_a.flushed, 2);

    // B 的收件箱收到两条（顺序 = FIFO）
    let items = inbox_b.poll(&key_b, 0).unwrap();
    assert_eq!(items.len(), 2, "B 应收到两条消息");
    assert_eq!(items[0].peer_id, id_a.id_base32());
    assert_eq!(items[0].text, "msg-1-for-b");
    assert_eq!(items[1].text, "第二条：中文消息 ✓");

    // 角色/房间纯函数自检
    let (x, y) = (id_a.id_base32(), id_b.id_base32());
    assert_eq!(pair_room(x, y), pair_room(y, x));
    assert_ne!(role_for(x, y), role_for(y, x));

    let _ = relay.kill();
    let _ = relay.wait_with_output();
}
