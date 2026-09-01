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
