//! 测试专用：进程级环境变量串行锁。
//!
//! `ELWRIGHT_ROOT` / `ELWRIGHT_USER_ROOT` / `ELWRIGHT_LLM_*` 是进程级
//! 环境变量，cargo test 默认多线程并行跑测试——改这些变量的测试必须
//! 互斥。此前 llm / registry / chat_store 各持一把**不同的**局部
//! `Mutex`，锁了等于没锁（enhancement-2026-08-quality-tier2-e2e 期间
//! chat_store 移入 lib 测试桶后并发踩踏暴露）。所有改这些变量的测试
//! 统一从本模块取同一把锁。
//!
//! 仅在 `cfg(test)`（含集成测试）下编译。

use std::sync::{Mutex, OnceLock};

fn global_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// 拿全局环境变量锁的 guard。放在测试函数第一行：
/// `let _g = env_serialization_guard();`
pub fn env_serialization_guard() -> std::sync::MutexGuard<'static, ()> {
    global_lock().lock().unwrap_or_else(|e| e.into_inner())
}
