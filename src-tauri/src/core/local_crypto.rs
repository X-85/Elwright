//! 本地静态密钥加密（ADR-003 §D4）。
//!
//! 发件箱/收件箱落盘一律经此加密：`chacha20poly1305(local_key, nonce12, aad)`
//! 密文 hex（`nonce || ct`）。密钥是身份目录下的随机 32 字节
//! （`identity::load_or_create_local_key`），不与任何会话密钥挂钩——
//! 队列内容因此与会话生命周期解耦，flush 时用当次握手的新会话重加密。

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};

#[derive(Debug, thiserror::Error)]
pub enum LocalCryptoError {
    #[error("密文格式非法")]
    BadFormat,
    #[error("解密失败（密钥不符或内容被篡改）")]
    DecryptFailed,
}

/// 加密：返回 `nonce(12B) || ciphertext+tag` 的 hex。
pub fn encrypt(local_key: &[u8; 32], aad: &str, plaintext: &[u8]) -> String {
    let cipher = ChaCha20Poly1305::new(local_key.into());
    let nonce_bytes = crate::core::identity::random_nonce12();
    let payload = Payload {
        msg: plaintext,
        aad: aad.as_bytes(),
    };
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), payload)
        .expect("chacha20poly1305 加密在正确长度 nonce 下不会失败");
    let mut blob = Vec::with_capacity(12 + ct.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ct);
    hex::encode(blob)
}

/// 解密 `encrypt` 的输出。AAD 必须与加密时一致（本设计用 peer_id 绑定条目归属）。
pub fn decrypt(
    local_key: &[u8; 32],
    aad: &str,
    blob_hex: &str,
) -> Result<Vec<u8>, LocalCryptoError> {
    let blob = hex::decode(blob_hex).map_err(|_| LocalCryptoError::BadFormat)?;
    if blob.len() < 12 + 16 {
        return Err(LocalCryptoError::BadFormat);
    }
    let (nonce_bytes, ct) = blob.split_at(12);
    let cipher = ChaCha20Poly1305::new(local_key.into());
    let payload = Payload {
        msg: ct,
        aad: aad.as_bytes(),
    };
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), payload)
        .map_err(|_| LocalCryptoError::DecryptFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_aad_binding() {
        let key = [7u8; 32];
        let plain = "hello 本地加密".as_bytes();
        let blob = encrypt(&key, "PEER_A", plain);
        assert_eq!(decrypt(&key, "PEER_A", &blob).unwrap(), plain);
        // AAD 不符（换了归属方）必须失败
        assert!(matches!(
            decrypt(&key, "PEER_B", &blob),
            Err(LocalCryptoError::DecryptFailed)
        ));
        // 密钥不符必须失败
        assert!(matches!(
            decrypt(&[8u8; 32], "PEER_A", &blob),
            Err(LocalCryptoError::DecryptFailed)
        ));
        // 每次加密 nonce 不同
        assert_ne!(encrypt(&key, "PEER_A", b"x"), encrypt(&key, "PEER_A", b"x"));
    }

    #[test]
    fn tampered_and_malformed_input_rejected() {
        let key = [7u8; 32];
        let mut blob = encrypt(&key, "P", b"data");
        // 翻转密文中间一个 hex 字符
        let bytes: Vec<char> = blob.chars().collect();
        let mid = bytes.len() / 2;
        let flipped = if bytes[mid] == '0' { '1' } else { '0' };
        blob = bytes[..mid].iter().collect::<String>()
            + &flipped.to_string()
            + &bytes[mid + 1..].iter().collect::<String>();
        assert!(decrypt(&key, "P", &blob).is_err());
        assert!(matches!(
            decrypt(&key, "P", "00"),
            Err(LocalCryptoError::BadFormat)
        ));
        assert!(matches!(
            decrypt(&key, "P", "zz-not-hex"),
            Err(LocalCryptoError::BadFormat)
        ));
    }
}
