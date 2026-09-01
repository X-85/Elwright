//! Messaging phase 2 — 本地身份（step 2，ADR-002 §D2）
//!
//! 范围：
//!   - `~/.elwright/identity/` 目录下的密钥对持久化（ed25519 签名 + X25519 协商）
//!   - 派生人类可读 ID：X25519 公钥 → SHA-256 → 头 80 bit → 16 字符 base32 Crockford
//!   - 邀请生成 / 邀请校验（带签名 + 有效期）
//!
//! 不在本模块：
//!   - Noise 握手（用 `messaging_transport`）
//!   - 中继 URL（用 `UserConfigFile::messaging_relay_url` flat 字段，IPC 在 commands）

use std::path::{Path, PathBuf};

use ed25519_dalek::{Signer, Verifier};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Crockford base32 字母表（无 I/L/O/U，减错友好）。
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// 身份目录默认路径（基于 user_root）。
pub fn default_identity_dir(user_root: &Path) -> PathBuf {
    user_root.join("identity")
}

/// 16 字符 ID 派生长度（base32 字符数）。
pub const ID_LENGTH: usize = 16;

/// 邀请短码长度（6 字符 base32）。
pub const INVITE_CODE_LENGTH: usize = 6;

/// 邀请默认有效期：5 分钟（300 秒）。
pub const DEFAULT_INVITE_TTL_SECS: i64 = 300;

/// Identity 文件名约定。
const SIGNING_KEY_FILE: &str = "signing.ed25519";
const DH_KEY_FILE: &str = "dh.x25519";
const ID_FILE: &str = "id.base32";

#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("密钥文件读取失败：{0}")]
    KeyFileRead(String),
    #[error("密钥文件写入失败：{0}")]
    KeyFileWrite(String),
    #[error("密钥格式非法：{0}")]
    KeyFormat(String),
    #[error("身份目录创建失败：{0}")]
    DirCreate(String),
    #[error("邀请码无效或已过期")]
    InviteInvalid,
    #[error("邀请签名校验失败")]
    InviteBadSignature,
    #[error("ed25519 错误：{0}")]
    Ed25519(String),
    #[error("dryoc 错误：{0}")]
    Dryoc(String),
}

impl From<ed25519_dalek::SignatureError> for IdentityError {
    fn from(e: ed25519_dalek::SignatureError) -> Self {
        IdentityError::Ed25519(e.to_string())
    }
}

/// 本地身份：ed25519（签名）+ X25519（协商）+ 派生 ID。
#[derive(Clone)]
pub struct Identity {
    signing_secret: ed25519_dalek::SigningKey,
    signing_public: ed25519_dalek::VerifyingKey,
    dh_secret: x25519_dalek::StaticSecret,
    dh_public: x25519_dalek::PublicKey,
    id_base32: String,
}

impl Identity {
    /// 从 `identity_dir` 加载身份；目录为空或缺失则生成新身份并持久化。
    pub fn load_or_create(identity_dir: &Path) -> Result<Self, IdentityError> {
        if identity_dir.is_dir() {
            // 尝试加载现有文件
            if let Ok(id) = Self::load(identity_dir) {
                return Ok(id);
            }
        }
        // 生成新身份
        let id = Self::generate()?;
        std::fs::create_dir_all(identity_dir)
            .map_err(|e| IdentityError::DirCreate(e.to_string()))?;
        id.persist(identity_dir)?;
        Ok(id)
    }

    /// 仅加载；目录缺失或文件损坏返回错误。
    pub fn load(identity_dir: &Path) -> Result<Self, IdentityError> {
        let signing_bytes = read_key_file(&identity_dir.join(SIGNING_KEY_FILE))?;
        let dh_bytes = read_key_file(&identity_dir.join(DH_KEY_FILE))?;
        let id_base32 = read_text_file(&identity_dir.join(ID_FILE))?;
        Self::from_bytes(&signing_bytes, &dh_bytes, &id_base32)
    }

    fn from_bytes(
        signing_bytes: &[u8; 32],
        dh_bytes: &[u8; 32],
        id_base32: &str,
    ) -> Result<Self, IdentityError> {
        let signing_secret = ed25519_dalek::SigningKey::from_bytes(signing_bytes);
        let signing_public = signing_secret.verifying_key();
        let dh_secret = x25519_dalek::StaticSecret::from(*dh_bytes);
        let dh_public = x25519_dalek::PublicKey::from(&dh_secret);
        // 校验 id_base32 与公钥一致
        let expected = derive_id_from_dh_public(&dh_public);
        if expected != id_base32 {
            return Err(IdentityError::KeyFormat(
                "id.base32 与 X25519 公钥不匹配".into(),
            ));
        }
        Ok(Self {
            signing_secret,
            signing_public,
            dh_secret,
            dh_public,
            id_base32: id_base32.to_string(),
        })
    }

    /// 生成全新身份（密钥从 OS RNG 出）。
    pub fn generate() -> Result<Self, IdentityError> {
        let mut rng = rand::rngs::OsRng;
        let mut signing_bytes = [0u8; 32];
        rng.fill_bytes(&mut signing_bytes);
        let signing_secret = ed25519_dalek::SigningKey::from_bytes(&signing_bytes);
        let signing_public = signing_secret.verifying_key();
        let dh_secret = x25519_dalek::StaticSecret::random_from_rng(rng);
        let dh_public = x25519_dalek::PublicKey::from(&dh_secret);
        let id_base32 = derive_id_from_dh_public(&dh_public);
        Ok(Self {
            signing_secret,
            signing_public,
            dh_secret,
            dh_public,
            id_base32,
        })
    }

    fn persist(&self, identity_dir: &Path) -> Result<(), IdentityError> {
        let signing_bytes = self.signing_secret.to_bytes();
        let dh_bytes = self.dh_secret.to_bytes();
        std::fs::write(identity_dir.join(SIGNING_KEY_FILE), signing_bytes)
            .map_err(|e| IdentityError::KeyFileWrite(e.to_string()))?;
        std::fs::write(identity_dir.join(DH_KEY_FILE), dh_bytes)
            .map_err(|e| IdentityError::KeyFileWrite(e.to_string()))?;
        std::fs::write(identity_dir.join(ID_FILE), self.id_base32.as_bytes())
            .map_err(|e| IdentityError::KeyFileWrite(e.to_string()))?;
        Ok(())
    }

    pub fn id_base32(&self) -> &str {
        &self.id_base32
    }

    pub fn signing_public_bytes(&self) -> [u8; 32] {
        self.signing_public.to_bytes()
    }

    pub fn dh_public_bytes(&self) -> [u8; 32] {
        self.dh_public.to_bytes()
    }

    /// 签发邀请：6 字符短码 + 二维码原文（含签名公钥 + 签名）+ 有效期秒数。
    pub fn create_invite(&self, ttl_secs: i64) -> Result<Invite, IdentityError> {
        let now = current_unix_secs();
        let expires_at = now + ttl_secs;
        let nonce: [u8; 8] = random_bytes();
        // payload: id || expires_at || nonce
        let mut payload = Vec::with_capacity(self.id_base32.len() + 8 + 8);
        payload.extend_from_slice(self.id_base32.as_bytes());
        payload.extend_from_slice(&expires_at.to_be_bytes());
        payload.extend_from_slice(&nonce);
        let sig = self.signing_secret.sign(&payload);
        let sig_bytes = sig.to_bytes();
        let short_code = encode_base32(&sig_bytes[0..5], INVITE_CODE_LENGTH);
        let qr_payload = format!(
            "elwright-invite:v2:{}:{}:{}:{}:{}:{}",
            self.id_base32,
            hex::encode(self.signing_public.to_bytes()),
            short_code,
            expires_at,
            hex::encode(nonce),
            hex::encode(sig_bytes),
        );
        Ok(Invite {
            short_code,
            qr_payload,
            expires_at,
        })
    }

    /// 校验对端发来的邀请（含完整签名公钥 + 签名）。返回 Ok 即接受为联系人。
    pub fn accept_invite(&self, invite: &InboundInvite) -> Result<(), IdentityError> {
        if invite.short_code.len() != INVITE_CODE_LENGTH
            || !invite
                .short_code
                .chars()
                .all(|c| CROCKFORD.contains(&(c.to_ascii_uppercase() as u8)))
        {
            return Err(IdentityError::InviteInvalid);
        }
        let now = current_unix_secs();
        if now > invite.expires_at {
            return Err(IdentityError::InviteInvalid);
        }
        let nonce = hex::decode(&invite.nonce_hex).map_err(|_| IdentityError::InviteInvalid)?;
        if nonce.len() != 8 {
            return Err(IdentityError::InviteInvalid);
        }
        let sig_bytes =
            hex::decode(&invite.signature_hex).map_err(|_| IdentityError::InviteInvalid)?;
        if sig_bytes.len() != 64 {
            return Err(IdentityError::InviteInvalid);
        }
        let pk_bytes = hex::decode(&invite.inviter_signing_pub_hex)
            .map_err(|_| IdentityError::InviteInvalid)?;
        if pk_bytes.len() != 32 {
            return Err(IdentityError::InviteInvalid);
        }
        let pk_arr: [u8; 32] = pk_bytes.as_slice().try_into().unwrap();
        let signing_pub = ed25519_dalek::VerifyingKey::from_bytes(&pk_arr)?;
        let mut payload = Vec::with_capacity(invite.inviter_id.len() + 8 + 8);
        payload.extend_from_slice(invite.inviter_id.as_bytes());
        payload.extend_from_slice(&invite.expires_at.to_be_bytes());
        payload.extend_from_slice(&nonce);
        let sig_arr: [u8; 64] = sig_bytes.as_slice().try_into().unwrap();
        let sig = ed25519_dalek::Signature::from_bytes(&sig_arr);
        signing_pub
            .verify(&payload, &sig)
            .map_err(|_| IdentityError::InviteBadSignature)?;
        let expected_short = encode_base32(&sig_bytes[0..5], INVITE_CODE_LENGTH);
        if expected_short != invite.short_code {
            return Err(IdentityError::InviteInvalid);
        }
        Ok(())
    }
}

/// 邀请：含短码、二维码原文、有效期。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Invite {
    pub short_code: String,
    pub qr_payload: String,
    pub expires_at: i64,
}

/// 对端发来的邀请二维码原文（接收方解析后用 `verify` 校验签名）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InboundInvite {
    pub inviter_id: String,
    pub inviter_signing_pub_hex: String,
    pub short_code: String,
    pub expires_at: i64,
    pub nonce_hex: String,
    /// ed25519 对 (inviter_id || expires_at || nonce) 的 64 字节签名（hex）。
    pub signature_hex: String,
}

/// 由 X25519 公钥派生 16 字符 base32 ID（Crockford）。
pub fn derive_id_from_dh_public(dh_public: &x25519_dalek::PublicKey) -> String {
    let mut hasher = Sha256::new();
    hasher.update(dh_public.as_bytes());
    let hash = hasher.finalize();
    encode_base32(&hash[..5], ID_LENGTH)
}

/// 由原始字节编 base32 Crockford（按 5 bit 一组，MSB 在前）。
fn encode_base32(input: &[u8], char_count: usize) -> String {
    let mut bits = 0u64;
    let mut bit_len = 0usize;
    let mut out = String::with_capacity(char_count);
    for &b in input {
        bits = (bits << 8) | b as u64;
        bit_len += 8;
        while bit_len >= 5 {
            bit_len -= 5;
            let idx = ((bits >> bit_len) & 0x1f) as usize;
            out.push(CROCKFORD[idx] as char);
        }
    }
    if bit_len > 0 && out.len() < char_count {
        let idx = ((bits << (5 - bit_len)) & 0x1f) as usize;
        out.push(CROCKFORD[idx] as char);
    }
    out.truncate(char_count);
    out
}

fn read_key_file(path: &Path) -> Result<[u8; 32], IdentityError> {
    let bytes = std::fs::read(path).map_err(|e| IdentityError::KeyFileRead(e.to_string()))?;
    if bytes.len() != 32 {
        return Err(IdentityError::KeyFormat(format!(
            "期望 32 字节，收到 {}",
            bytes.len()
        )));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

fn read_text_file(path: &Path) -> Result<String, IdentityError> {
    std::fs::read_to_string(path).map_err(|e| IdentityError::KeyFileRead(e.to_string()))
}

fn current_unix_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut arr = [0u8; N];
    let mut rng = rand::rngs::OsRng;
    rng.fill_bytes(&mut arr);
    arr
}

// 由 inviter_id 反查 ed25519 签名公钥——**当前实现无法仅凭 ID 反查签名密钥**，
// 因为签名密钥与 DH 公钥之间没有官方数学映射。这是 §D2 的设计取舍：
//   - 选项 A：把签名密钥的公钥也写进 ID 派生（ID 长度翻倍 + 双密钥绑定）
//   - 选项 B：邀请 QR 携带 inviter 的签名公钥（hex），接收方校验后保存
//
// 当前实现选 B（ADR §D2 实施说明）：v2 邀请 qr_payload 段 6 已含签名公钥；
// 接收方先解析再校验，验后把 inviter_signing_pub_hex 持久化到 contacts.json。

// ---------- 测试 ----------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn encode_base32_charset_and_length() {
        // 10 字节输入 → 16 字符 base32（10*8 = 80 bit / 5 = 16 字符）
        let s = encode_base32(&[0x00; 10], 16);
        assert_eq!(s.len(), 16);
        for c in s.chars() {
            assert!(CROCKFORD.contains(&(c.to_ascii_uppercase() as u8)));
        }
        // 5 字节输入 → 8 字符 base32
        let s2 = encode_base32(&[0xff; 5], 8);
        assert_eq!(s2.len(), 8);
        for c in s2.chars() {
            assert!(CROCKFORD.contains(&(c as u8)));
        }
    }

    #[test]
    fn identity_generate_is_unique() {
        let a = Identity::generate().unwrap();
        let b = Identity::generate().unwrap();
        assert_ne!(a.id_base32(), b.id_base32());
        assert_ne!(a.dh_public_bytes(), b.dh_public_bytes());
    }

    #[test]
    fn identity_persist_and_reload() {
        let dir = tempdir().unwrap();
        let original = Identity::generate().unwrap();
        original.persist(dir.path()).unwrap();
        let loaded = Identity::load(dir.path()).unwrap();
        assert_eq!(original.id_base32(), loaded.id_base32());
        assert_eq!(original.dh_public_bytes(), loaded.dh_public_bytes());
        assert_eq!(
            original.signing_public_bytes(),
            loaded.signing_public_bytes()
        );
    }

    #[test]
    fn identity_load_or_create_idempotent() {
        let dir = tempdir().unwrap();
        let first = Identity::load_or_create(dir.path()).unwrap();
        let second = Identity::load_or_create(dir.path()).unwrap();
        // 不重新生成；同一身份稳定
        assert_eq!(first.id_base32(), second.id_base32());
    }

    #[test]
    fn invite_short_code_length_and_charset() {
        let id = Identity::generate().unwrap();
        let invite = id.create_invite(DEFAULT_INVITE_TTL_SECS).unwrap();
        assert_eq!(invite.short_code.len(), INVITE_CODE_LENGTH);
        for c in invite.short_code.chars() {
            assert!(CROCKFORD.contains(&(c.to_ascii_uppercase() as u8)));
        }
    }

    #[test]
    fn invite_expires_in_future() {
        let id = Identity::generate().unwrap();
        let invite = id.create_invite(DEFAULT_INVITE_TTL_SECS).unwrap();
        let now = current_unix_secs();
        assert!(invite.expires_at > now);
        assert!(invite.expires_at <= now + DEFAULT_INVITE_TTL_SECS + 1);
    }

    #[test]
    fn invite_qr_payload_format() {
        let id = Identity::generate().unwrap();
        let invite = id.create_invite(DEFAULT_INVITE_TTL_SECS).unwrap();
        let parts: Vec<&str> = invite.qr_payload.split(':').collect();
        assert_eq!(parts.len(), 8);
        assert_eq!(parts[0], "elwright-invite");
        assert_eq!(parts[1], "v2");
        assert_eq!(parts[2], id.id_base32());
        assert_eq!(parts[3].len(), 64); // 32 字节 ed25519 pub hex
        assert_eq!(parts[4], invite.short_code);
        assert!(parts[5].parse::<i64>().is_ok());
        // parts[6] = nonce hex (8 字节 → 16 字符), parts[7] = signature hex (64 字节 → 128 字符)
        assert_eq!(parts[6].len(), 16);
        assert_eq!(parts[7].len(), 128);
    }

    #[test]
    fn accept_invite_rejects_wrong_length_code() {
        let id = Identity::generate().unwrap();
        let inviter = Identity::generate().unwrap();
        let invite = InboundInvite {
            inviter_id: inviter.id_base32().to_string(),
            inviter_signing_pub_hex: hex::encode(inviter.signing_public_bytes()),
            short_code: "ABC".into(),
            expires_at: current_unix_secs() + 300,
            nonce_hex: "0011223344556677".into(),
            signature_hex: hex::encode([0u8; 64]),
        };
        assert!(matches!(
            id.accept_invite(&invite),
            Err(IdentityError::InviteInvalid)
        ));
    }

    #[test]
    fn accept_invite_rejects_expired() {
        let id = Identity::generate().unwrap();
        let inviter = Identity::generate().unwrap();
        let invite = InboundInvite {
            inviter_id: inviter.id_base32().to_string(),
            inviter_signing_pub_hex: hex::encode(inviter.signing_public_bytes()),
            short_code: "ABCDEF".into(),
            expires_at: current_unix_secs() - 100,
            nonce_hex: "0011223344556677".into(),
            signature_hex: hex::encode([0u8; 64]),
        };
        assert!(matches!(
            id.accept_invite(&invite),
            Err(IdentityError::InviteInvalid)
        ));
    }

    #[test]
    fn invite_round_trip_accepted() {
        let me = Identity::generate().unwrap();
        let inviter = Identity::generate().unwrap();
        let created = inviter.create_invite(DEFAULT_INVITE_TTL_SECS).unwrap();
        let parts: Vec<&str> = created.qr_payload.split(':').collect();
        assert_eq!(parts.len(), 8);
        assert_eq!(parts[0], "elwright-invite");
        assert_eq!(parts[1], "v2");
        let inbound = InboundInvite {
            inviter_id: parts[2].to_string(),
            inviter_signing_pub_hex: parts[3].to_string(),
            short_code: parts[4].to_string(),
            expires_at: parts[5].parse().unwrap(),
            nonce_hex: parts[6].to_string(),
            signature_hex: parts[7].to_string(),
        };
        me.accept_invite(&inbound).expect("合法邀请应被接受");
    }

    #[test]
    fn accept_invite_rejects_tampered_short_code() {
        let me = Identity::generate().unwrap();
        let inviter = Identity::generate().unwrap();
        let created = inviter.create_invite(DEFAULT_INVITE_TTL_SECS).unwrap();
        let parts: Vec<&str> = created.qr_payload.split(':').collect();
        let mut inbound = InboundInvite {
            inviter_id: parts[2].to_string(),
            inviter_signing_pub_hex: parts[3].to_string(),
            short_code: parts[4].to_string(),
            expires_at: parts[5].parse().unwrap(),
            nonce_hex: parts[6].to_string(),
            signature_hex: parts[7].to_string(),
        };
        // 篡改 short_code 一个字符（保持长度+Crockford）
        let first = inbound.short_code.chars().next().unwrap();
        let bad = if first == 'A' { 'B' } else { 'A' };
        let mut tampered: String = bad.into();
        tampered.push_str(&inbound.short_code[1..]);
        inbound.short_code = tampered;
        assert!(matches!(
            me.accept_invite(&inbound),
            Err(IdentityError::InviteInvalid)
        ));
    }

    #[test]
    fn accept_invite_rejects_tampered_signature() {
        let me = Identity::generate().unwrap();
        let inviter = Identity::generate().unwrap();
        let created = inviter.create_invite(DEFAULT_INVITE_TTL_SECS).unwrap();
        let parts: Vec<&str> = created.qr_payload.split(':').collect();
        let mut sig_bytes = hex::decode(parts[7]).unwrap();
        sig_bytes[0] ^= 0xff;
        let inbound = InboundInvite {
            inviter_id: parts[2].to_string(),
            inviter_signing_pub_hex: parts[3].to_string(),
            short_code: parts[4].to_string(),
            expires_at: parts[5].parse().unwrap(),
            nonce_hex: parts[6].to_string(),
            signature_hex: hex::encode(sig_bytes),
        };
        assert!(matches!(
            me.accept_invite(&inbound),
            Err(IdentityError::InviteBadSignature)
        ));
    }
}
