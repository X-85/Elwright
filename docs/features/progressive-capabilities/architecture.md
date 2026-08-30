# 架构

`registry::Capability` 增加可选 `releaseTier` 与 `unlockAfterUses` 字段，旧注册表通过 serde 默认值保持兼容。Tauri IPC 继续返回完整能力元数据，前端根据本地使用计数计算展示和锁定状态。

MVP 不新增网络服务、不自动下载代码，也不在 Rust 侧记录行为。后续若需要 CLI/桌面共享解锁进度，再将本地记录抽到用户层 JSON，并保持字段级迁移。
