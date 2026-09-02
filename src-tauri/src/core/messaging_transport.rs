//! Messaging phase 2 — 协议层骨架
//!
//! 范围（ADR-002 Step 1）：
//!   - Noise_XX 握手（snow crate，纯 Rust，跨平台）
//!   - 数据通道：snow TransportState 自带的 ChaCha20-Poly1305 AEAD
//!     （nonce 单调维护，重放自动拒绝；`rekey_*` 触发显式 rekey）
//!   - 帧格式：握手 / 数据 / 控制 三类，带版本号
//!
//! 实施期调整（vs ADR-002 §D4）：
//!   ADR 原计划握手后用独立 libsodium secretstream 流加密；调研发现 snow
//!   TransportState 自带 ChaCha20-Poly1305 AEAD + nonce 管理 + rekey，
//!   完全覆盖 ADR §D4 的安全需求（AEAD 完整性、重放拒绝、顺序保证、显式 rekey）。
//!   实施期去掉独立 secretstream 封装，简化协议层、少两个依赖（dryoc / libsodium-sys）。
//!   此调整写入 ADR-002「实施偏差」段。
//!
//! 不在范围（本模块边界）：
//!   - WebSocket 连接管理（Step 4：客户端 + 中继最小回路）
//!   - 身份密钥对生成与持久化（Step 2：`core::identity`）
//!   - 离线消息队列（Step 5）
//!   - 中继路由（参考实现见 `docs/features/messaging/relay/`）
//!
//! 帧格式（详见 `docs/features/messaging/transport-protocol.md`）：
//!   - 握手帧：`[u8 version=0][u8 msg_type=0][noise payload...]`
//!     payload = Noise_XX 协议原生消息（snow `HandshakeState::write_message`）
//!   - 数据帧：`[u8 version=0][u8 msg_type=1][snow transport payload...]`
//!     snow TransportState::write_message 返回 `[AEAD ciphertext + 16B tag]`，nonce 内部维护
//!   - 控制帧：`[u8 version=0][u8 msg_type=2][code(1B)][payload...]`
//!     code = 0=Ping / 1=Pong / 2=Close / 3=Error（控制帧明文，仅用于协议信号）

use snow::Builder as NoiseBuilder;
use snow::Error as NoiseError;
use snow::HandshakeState as NoiseHandshakeState;
use snow::TransportState as NoiseTransportState;

/// Noise 协议单消息明文上限（与 snow `constants::MAXMSGLEN` 一致，
/// snow 未 re-export，这里手填以避免依赖私有模块）。
pub const NOISE_MAX_MESSAGE: usize = 65535;

/// 协议版本号。任何不兼容改动必须 bump。
pub const PROTOCOL_VERSION: u8 = 0;

/// 帧类型字节（在版本号之后）
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Handshake = 0,
    Data = 1,
    Control = 2,
}

impl FrameType {
    fn from_u8(v: u8) -> Result<Self, TransportError> {
        Ok(match v {
            0 => FrameType::Handshake,
            1 => FrameType::Data,
            2 => FrameType::Control,
            other => return Err(TransportError::UnknownFrameType(other)),
        })
    }
}

/// 控制帧子类型
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCode {
    Ping = 0,
    Pong = 1,
    Close = 2,
    Error = 3,
}

impl ControlCode {
    fn from_u8(v: u8) -> Result<Self, TransportError> {
        Ok(match v {
            0 => ControlCode::Ping,
            1 => ControlCode::Pong,
            2 => ControlCode::Close,
            3 => ControlCode::Error,
            other => return Err(TransportError::UnknownControlCode(other)),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("协议版本不匹配（收到 {0}，期望 {PROTOCOL_VERSION}）")]
    VersionMismatch(u8),
    #[error("未知帧类型 {0}")]
    UnknownFrameType(u8),
    #[error("未知控制码 {0}")]
    UnknownControlCode(u8),
    #[error("帧截断（期望至少 {expected} 字节，收到 {actual}）")]
    Truncated { expected: usize, actual: usize },
    #[error("控制帧截断")]
    ControlTruncated,
    #[error("Noise 协议错误：{0}")]
    Noise(String),
    #[error("消息缓冲过小：需要 {needed} 字节，提供 {provided}")]
    BufferTooSmall { needed: usize, provided: usize },
    #[error("AEAD 标签校验失败（可能被篡改或重放）")]
    AeadFailed,
}

impl From<NoiseError> for TransportError {
    fn from(e: NoiseError) -> Self {
        TransportError::Noise(e.to_string())
    }
}

/// 握手状态机包装。是否完成由 snow `HandshakeState::is_handshake_finished()` 决定——
/// snow 对 initiator 写完 msg 3 与 responder 读完 msg 3 都返回 true（中间轮次不返回）。
/// 早期完成的检查依赖 snow 的内部状态，本结构不缓存 finished 字段，避免误判。
pub struct Handshake {
    state: NoiseHandshakeState,
}

impl Handshake {
    pub fn new_initiator(static_secret: [u8; 32]) -> Result<Self, TransportError> {
        let builder = NoiseBuilder::new("Noise_XX_25519_ChaChaPoly_SHA256".parse().unwrap());
        let state = builder
            .local_private_key(&static_secret)
            .build_initiator()?;
        Ok(Self { state })
    }

    pub fn new_responder(static_secret: [u8; 32]) -> Result<Self, TransportError> {
        let builder = NoiseBuilder::new("Noise_XX_25519_ChaChaPoly_SHA256".parse().unwrap());
        let state = builder
            .local_private_key(&static_secret)
            .build_responder()?;
        Ok(Self { state })
    }

    /// 处理一轮握手。
    /// - `inbound = &[]`：本端先手（仅 write；仅 initiator 第一轮与 responder 第二轮如此）
    /// - `inbound = 对方消息`：先 read 解密（产出 payload），再 write 加密（产出 out）
    ///
    /// 真实使用方应按 XX 协议的 3 轮消息规则调用（详见 `complete_handshake`）。
    pub fn step(&mut self, inbound: &[u8]) -> Result<Vec<u8>, TransportError> {
        let payload: Vec<u8> = if inbound.is_empty() {
            // 本端先手：直接 write_message，无需 read。
            Vec::new()
        } else {
            let mut buf = vec![0u8; NOISE_MAX_MESSAGE];
            let n = self.state.read_message(inbound, &mut buf)?;
            buf.truncate(n);
            buf
        };
        let mut out = vec![0u8; NOISE_MAX_MESSAGE];
        let n = self.state.write_message(&payload, &mut out)?;
        out.truncate(n);
        Ok(out)
    }

    pub fn is_finished(&self) -> bool {
        self.state.is_handshake_finished()
    }

    /// 握手完成后转化为传输态。调用方拿走所有权。
    /// `into_transport_mode` 自身会在握手未完成时返回 snow 错误，本函数不再额外守卫。
    pub fn into_transport(self) -> Result<Transport, TransportError> {
        let transport = self.state.into_transport_mode()?;
        Ok(Transport { inner: transport })
    }

    /// 仅读取最后一条握手消息（不写）。Responder XX 协议最后一步专用：
    /// snow 0.9 在 read 最后一条后握手完成；`step()` 因含 write 在此场景下会失败。
    pub fn read_final(&mut self, inbound: &[u8]) -> Result<Vec<u8>, TransportError> {
        let mut buf = vec![0u8; NOISE_MAX_MESSAGE];
        let n = self.state.read_message(inbound, &mut buf)?;
        buf.truncate(n);
        Ok(buf)
    }
}

/// 传输态。包装 snow TransportState（自带 ChaCha20-Poly1305 AEAD）。
/// 一个实例代表一方的一对方向（send/recv）；双向通信需要两端各一个 Transport。
pub struct Transport {
    inner: NoiseTransportState,
}

impl Transport {
    /// 加密一条消息，返回 AEAD 字节流（不含版本/类型头，调用方负责加 Frame 头）。
    /// 输出长度 = 明文 + 16B AEAD tag。
    pub fn send(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, TransportError> {
        let needed = plaintext.len() + 16;
        let mut buf = vec![0u8; needed];
        let written = self.inner.write_message(plaintext, &mut buf)?;
        buf.truncate(written);
        Ok(buf)
    }

    /// 解密一条消息（来自对端推送的字节流，含 16B tag）。
    pub fn recv(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, TransportError> {
        if ciphertext.len() < 16 {
            return Err(TransportError::AeadFailed);
        }
        let plain_len = ciphertext.len() - 16;
        let mut buf = vec![0u8; plain_len];
        match self.inner.read_message(ciphertext, &mut buf) {
            Ok(n) => {
                buf.truncate(n);
                Ok(buf)
            }
            Err(e) => Err(TransportError::Noise(format!("AEAD decrypt failed: {e}"))),
        }
    }

    /// 显式 rekey 出站方向。
    pub fn rekey_send(&mut self) {
        self.inner.rekey_outgoing();
    }

    /// 显式 rekey 入站方向。
    pub fn rekey_recv(&mut self) {
        self.inner.rekey_incoming();
    }

    /// 当前出站 nonce（用于测试断言重放拒绝）。
    pub fn send_nonce(&self) -> u64 {
        self.inner.sending_nonce()
    }

    /// 当前入站 nonce。
    pub fn recv_nonce(&self) -> u64 {
        self.inner.receiving_nonce()
    }

    /// 取得对端静态公钥（handshake 完成后才有）。
    pub fn remote_static(&self) -> Option<&[u8]> {
        self.inner.get_remote_static()
    }
}

/// 帧的字节层序列化 / 反序列化。
pub struct Frame;

impl Frame {
    pub fn encode_handshake(payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(payload.len() + 2);
        out.push(PROTOCOL_VERSION);
        out.push(FrameType::Handshake as u8);
        out.extend_from_slice(payload);
        out
    }

    pub fn encode_data(transport_payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(transport_payload.len() + 2);
        out.push(PROTOCOL_VERSION);
        out.push(FrameType::Data as u8);
        out.extend_from_slice(transport_payload);
        out
    }

    pub fn encode_control(code: ControlCode, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(payload.len() + 3);
        out.push(PROTOCOL_VERSION);
        out.push(FrameType::Control as u8);
        out.push(code as u8);
        out.extend_from_slice(payload);
        out
    }

    pub fn parse_header(buf: &[u8]) -> Result<(FrameType, usize), TransportError> {
        if buf.len() < 2 {
            return Err(TransportError::Truncated {
                expected: 2,
                actual: buf.len(),
            });
        }
        if buf[0] != PROTOCOL_VERSION {
            return Err(TransportError::VersionMismatch(buf[0]));
        }
        let ft = FrameType::from_u8(buf[1])?;
        Ok((ft, 2))
    }

    pub fn parse_control(buf: &[u8]) -> Result<(ControlCode, &[u8]), TransportError> {
        if buf.len() < 3 {
            return Err(TransportError::ControlTruncated);
        }
        let code = ControlCode::from_u8(buf[2])?;
        Ok((code, &buf[3..]))
    }
}

/// 辅助函数：两端的 static secret 走完 XX 握手，返回 `(alice_transport, bob_transport)`。
/// alice 始终是 initiator；真实部署里谁发起由用户交互决定（邀请生成方为 initiator）。
///
/// snow 0.9 的 Noise_XX 流程：
///   - msg 1：alice.write → bob.read + write(msg2)
///   - alice.read(msg2) + write(msg3)——msg 3 是 initiator 单方面完成标志
///   - bob 在 write msg 2 后即 `is_handshake_finished()=true`，不需要 read msg 3
///     （这与 [Noise spec §5.3](http://noiseprotocol.org/noise.html) 一致——XX 是
///     initiator 主动写完 msg 3 才双方对称进入 transport，responder 提前 finished
///     只是意味着 responder 端随时可调用 into_transport_mode）。
pub fn complete_handshake(
    alice_static: [u8; 32],
    bob_static: [u8; 32],
) -> Result<(Transport, Transport), TransportError> {
    let mut alice = Handshake::new_initiator(alice_static)?;
    let mut bob = Handshake::new_responder(bob_static)?;

    // Msg 1: Alice → Bob（alice 先手只写，bob 读+写 msg 2）
    let p1 = alice.step(&[])?;
    let p2 = bob.step(&p1)?;
    // Msg 2: Bob → Alice（alice 读+写 msg 3）
    let p3 = alice.step(&p2)?;
    // Msg 3: Alice → Bob（bob 仅读最后一条完成握手——step 含 write，
    // 雪 0.9 在最后轮 write 会因 my_turn=false 失败，故用 read_final）
    let _final = bob.read_final(&p3)?;

    let alice_t = alice.into_transport()?;
    let bob_t = bob.into_transport()?;
    Ok((alice_t, bob_t))
}

// ---------- 测试 ----------

#[cfg(test)]
mod tests {
    use super::*;

    const ALICE_STATIC: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];
    const BOB_STATIC: [u8; 32] = [
        0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
        0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe,
        0xbf, 0xc0,
    ];

    fn pair() -> (Transport, Transport) {
        complete_handshake(ALICE_STATIC, BOB_STATIC).unwrap()
    }

    #[test]
    fn handshake_round_trip_yields_remote_static() {
        let (alice, bob) = pair();
        assert!(alice.remote_static().is_some());
        assert!(bob.remote_static().is_some());
        // XX 协议双向交换静态公钥：alice 看到的是 bob 的 static，bob 看到的是 alice 的 static
        assert_ne!(alice.remote_static().unwrap(), bob.remote_static().unwrap());
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let (mut alice, mut bob) = pair();
        let plaintext = b"hello from alice";

        let wire = alice.send(plaintext).unwrap();
        let plain = bob.recv(&wire).unwrap();
        assert_eq!(plain, plaintext);

        // 反向也能通
        let wire2 = bob.send(b"hi alice").unwrap();
        let plain2 = alice.recv(&wire2).unwrap();
        assert_eq!(plain2, b"hi alice");
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let (mut alice, mut bob) = pair();
        let mut wire = alice.send(b"hi").unwrap();
        wire[0] ^= 0x01;
        assert!(bob.recv(&wire).is_err());
    }

    #[test]
    fn nonce_replay_rejected() {
        let (mut alice, mut bob) = pair();
        let wire = alice.send(b"once").unwrap();
        // 第一次 recv 成功
        bob.recv(&wire).unwrap();
        // 第二次 recv 同一条 cipher 失败（AEAD nonce 已被推进，重放必失败）
        assert!(bob.recv(&wire).is_err());
    }

    #[test]
    fn rekey_works_round_trip() {
        let (mut alice, mut bob) = pair();
        // rekey 重置 cipher key，不重置 nonce——只验证 rekey 后通信仍通
        alice.rekey_send();
        bob.rekey_recv();
        let wire = alice.send(b"after-rekey").unwrap();
        let plain = bob.recv(&wire).unwrap();
        assert_eq!(plain, b"after-rekey");
    }

    #[test]
    fn frame_encode_decode_handshake() {
        let payload = b"noise-msg-bytes";
        let frame = Frame::encode_handshake(payload);
        let (ft, off) = Frame::parse_header(&frame).unwrap();
        assert_eq!(ft, FrameType::Handshake);
        assert_eq!(&frame[off..], payload);
    }

    #[test]
    fn frame_encode_decode_data() {
        let payload = b"transport-bytes";
        let frame = Frame::encode_data(payload);
        let (ft, off) = Frame::parse_header(&frame).unwrap();
        assert_eq!(ft, FrameType::Data);
        assert_eq!(&frame[off..], payload);
    }

    #[test]
    fn frame_encode_decode_control() {
        let payload = b"ping-payload";
        let frame = Frame::encode_control(ControlCode::Ping, payload);
        let (ft, _) = Frame::parse_header(&frame).unwrap();
        assert_eq!(ft, FrameType::Control);
        let (code, rest) = Frame::parse_control(&frame).unwrap();
        assert_eq!(code, ControlCode::Ping);
        assert_eq!(rest, payload);
    }

    #[test]
    fn frame_version_mismatch() {
        let mut frame = Frame::encode_handshake(b"x");
        frame[0] = 99;
        let err = Frame::parse_header(&frame).unwrap_err();
        assert!(matches!(err, TransportError::VersionMismatch(99)));
    }

    #[test]
    fn frame_truncated_header() {
        let frame = vec![PROTOCOL_VERSION];
        let err = Frame::parse_header(&frame).unwrap_err();
        assert!(matches!(err, TransportError::Truncated { .. }));
    }

    #[test]
    fn frame_unknown_frame_type() {
        let frame = vec![PROTOCOL_VERSION, 99];
        let err = Frame::parse_header(&frame).unwrap_err();
        assert!(matches!(err, TransportError::UnknownFrameType(99)));
    }

    #[test]
    fn ciphertext_is_not_plaintext() {
        let (mut alice, _) = pair();
        let plaintext = b"super secret";
        let wire = alice.send(plaintext).unwrap();
        // 密文里不应出现明文任何字节序列（防止明文泄露）
        assert!(!window_contains(&wire, plaintext));
    }

    fn window_contains(haystack: &[u8], needle: &[u8]) -> bool {
        if needle.is_empty() || needle.len() > haystack.len() {
            return false;
        }
        haystack.windows(needle.len()).any(|w| w == needle)
    }
}
