//! Messaging phase 2 step 3 — IPC 冒烟。
//!
//! 覆盖：
//! 1. `identity_get` 首次调用生成新身份，再次调用返回同一 ID
//! 2. `identity_create_invite` 返回 v2 QR（8 段）+ short_code 6 字符
//! 3. `identity_accept_invite` 合法邀请 → Ok；篡改签名 → Err
//! 4. `get_messaging_config` 初始 relay_url 空
//! 5. `set_messaging_relay_url` 写入 → 读出；非法值拒绝

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;

type Wv = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn setup(tag: &str) -> (PathBuf, impl Drop) {
    let guard = env_lock();
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let user = std::env::temp_dir().join(format!(
        "elwright-msg-ipc-user-{}-{}-{}",
        std::process::id(),
        tag,
        seq
    ));
    let _ = fs::remove_dir_all(&user);
    fs::create_dir_all(&user).unwrap();
    std::env::set_var("ELWRIGHT_USER_ROOT", &user);
    struct Guard(
        #[allow(dead_code)] std::sync::MutexGuard<'static, ()>,
        PathBuf,
    );
    impl Drop for Guard {
        fn drop(&mut self) {
            std::env::remove_var("ELWRIGHT_USER_ROOT");
            let _ = fs::remove_dir_all(&self.1);
        }
    }
    (user.clone(), Guard(guard, user))
}

fn build_wv() -> Wv {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            elwright_core::core::commands::identity_get,
            elwright_core::core::commands::identity_create_invite,
            elwright_core::core::commands::identity_accept_invite,
            elwright_core::core::commands::get_messaging_config,
            elwright_core::core::commands::set_messaging_relay_url,
        ])
        .build(mock_context(noop_assets()))
        .expect("build mock app");
    tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("创建 main webview")
}

fn request(cmd: &str, body: serde_json::Value) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        url: if cfg!(any(windows, target_os = "android")) {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        }
        .parse()
        .unwrap(),
        body: body.into(),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    }
}

fn call(
    wv: &Wv,
    cmd: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    get_ipc_response(wv, request(cmd, body)).map(|b| match &b {
        tauri::ipc::InvokeResponseBody::Json(s) => {
            serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
        }
        tauri::ipc::InvokeResponseBody::Raw(bytes) => {
            serde_json::from_slice(bytes).unwrap_or(serde_json::Value::Null)
        }
    })
}

fn call_ok(wv: &Wv, cmd: &str, body: serde_json::Value) -> serde_json::Value {
    call(wv, cmd, body).expect("命令应成功")
}

#[test]
fn identity_get_returns_stable_id_across_calls() {
    let (_user, _g) = setup("id-stable");
    let wv = build_wv();
    let first = call_ok(&wv, "identity_get", serde_json::json!({}));
    let id1 = first["id_base32"].as_str().expect("id_base32 字段");
    assert_eq!(id1.len(), 16);
    let second = call_ok(&wv, "identity_get", serde_json::json!({}));
    let id2 = second["id_base32"].as_str().expect("id_base32 字段");
    assert_eq!(id1, id2, "同一身份应稳定");
    // 公钥 hex 都是 64 字符（32 字节）
    assert_eq!(first["signing_pub_hex"].as_str().unwrap().len(), 64);
    assert_eq!(first["dh_pub_hex"].as_str().unwrap().len(), 64);
    assert_eq!(first["signing_pub_hex"], second["signing_pub_hex"]);
}

#[test]
fn identity_create_invite_returns_v3_qr_and_short_code() {
    let (_user, _g) = setup("invite-create");
    let wv = build_wv();
    let out = call_ok(
        &wv,
        "identity_create_invite",
        serde_json::json!({ "ttlSecs": 300 }),
    );
    let qr = out["qr_payload"].as_str().expect("qr_payload");
    let short = out["short_code"].as_str().expect("short_code");
    assert_eq!(short.len(), 6, "short_code 必须 6 字符");
    // v3（ADR-003 §D1）：9 段，第 4 字段为对端 DH 公钥
    let parts: Vec<&str> = qr.split(':').collect();
    assert_eq!(parts.len(), 9);
    assert_eq!(parts[0], "elwright-invite");
    assert_eq!(parts[1], "v3");
    assert_eq!(parts[3].len(), 64, "ed25519 公钥 hex");
    assert_eq!(parts[4].len(), 64, "x25519 DH 公钥 hex");
    assert_eq!(
        parts[5], short,
        "qr_payload 中的 short_code 字段必须与字段一致"
    );
    let expires: i64 = parts[6].parse().expect("expires_at 可解析");
    assert!(expires > 0);
}

#[test]
fn identity_create_invite_rejects_bad_ttl() {
    let (_user, _g) = setup("invite-ttl");
    let wv = build_wv();
    let r = call(
        &wv,
        "identity_create_invite",
        serde_json::json!({ "ttlSecs": 5 }),
    );
    assert!(r.is_err(), "ttl < 30 应被拒绝");
}

#[test]
fn identity_accept_invite_round_trip_and_tamper() {
    let (_user, _g) = setup("invite-accept");
    let wv = build_wv();
    // 生成合法邀请
    let invite = call_ok(
        &wv,
        "identity_create_invite",
        serde_json::json!({ "ttlSecs": 300 }),
    );
    let qr = invite["qr_payload"].as_str().unwrap();
    // 直接用生成的 QR 接受（同一身份校验自己的合法签名）→ 应成功
    let accepted = call_ok(
        &wv,
        "identity_accept_invite",
        serde_json::json!({ "qrPayload": qr }),
    );
    let contact = accepted["inviter_id"].as_str().expect("inviter_id 字段");
    assert!(!contact.is_empty());
    // v3 QR 自带 DH 公钥——联系人视图必须回填（不再占位空串）
    let dh = accepted["dh_pub_hex"].as_str().expect("dh_pub_hex");
    let parts: Vec<&str> = qr.split(':').collect();
    assert_eq!(dh, parts[4], "回填的 DH 公钥须与 QR 载荷一致");

    // 篡改签名段 → 应被拒
    let mut tampered = parts.to_vec();
    let mut sig_bytes = hex::decode(parts[8]).unwrap();
    sig_bytes[0] ^= 0xff;
    let bad_sig = hex::encode(sig_bytes);
    tampered[8] = &bad_sig;
    let bad_qr = tampered.join(":");
    let r = call(
        &wv,
        "identity_accept_invite",
        serde_json::json!({ "qrPayload": bad_qr }),
    );
    assert!(r.is_err(), "签名被篡改应被拒");
}

#[test]
fn identity_accept_invite_rejects_garbage_qr() {
    let (_user, _g) = setup("invite-garbage");
    let wv = build_wv();
    let r = call(
        &wv,
        "identity_accept_invite",
        serde_json::json!({ "qrPayload": "elwright-invite:v2:id:pk:short:0:nonce:sig" }),
    );
    assert!(r.is_err(), "v2 格式不再接受");
    let r = call(
        &wv,
        "identity_accept_invite",
        serde_json::json!({ "qrPayload": "not-a-valid-qr" }),
    );
    assert!(r.is_err());
}

#[test]
fn messaging_config_round_trip() {
    let (_user, _g) = setup("msg-config-rt");
    let wv = build_wv();
    let initial = call_ok(&wv, "get_messaging_config", serde_json::json!({}));
    assert_eq!(initial["relay_url"].as_str().unwrap(), "");

    // 写合法值
    let after = call_ok(
        &wv,
        "set_messaging_relay_url",
        serde_json::json!({ "url": "wss://relay.example.com:9443" }),
    );
    assert_eq!(
        after["relay_url"].as_str().unwrap(),
        "wss://relay.example.com:9443"
    );
    let read_back = call_ok(&wv, "get_messaging_config", serde_json::json!({}));
    assert_eq!(
        read_back["relay_url"].as_str().unwrap(),
        "wss://relay.example.com:9443"
    );

    // 清除（空串）
    let cleared = call_ok(
        &wv,
        "set_messaging_relay_url",
        serde_json::json!({ "url": "" }),
    );
    assert_eq!(cleared["relay_url"].as_str().unwrap(), "");
}

#[test]
fn messaging_config_rejects_invalid_url() {
    let (_user, _g) = setup("msg-config-invalid");
    let wv = build_wv();
    let r = call(
        &wv,
        "set_messaging_relay_url",
        serde_json::json!({ "url": "http://example.com" }),
    );
    assert!(r.is_err(), "非 ws/wss 应被拒");
    let r = call(
        &wv,
        "set_messaging_relay_url",
        serde_json::json!({ "url": "ws://" }),
    );
    assert!(r.is_err(), "缺 host 应被拒");
}
