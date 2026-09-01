# 架构

`registry::Capability` 增加可选 `releaseTier` 与 `unlockAfterUses` 字段，旧注册表通过 serde 默认值保持兼容。Tauri IPC 继续返回完整能力元数据，前端根据本地使用计数计算展示和锁定状态。

MVP 不新增网络服务、不自动下载代码，也不在 Rust 侧记录行为。后续若需要 CLI/桌面共享解锁进度，再将本地记录抽到用户层 JSON，并保持字段级迁移。

## 成长体系透明化（2026-08-31，ADR-001）

- 纯函数收口 `src/lib/growth.ts`：`isUnlocked`（tier≤1 恒解锁；tier>1 需
  `unlockAfterUses` 且累计使用达标）/ `growthSummary`（锁定清单 + 最近可解锁项）/
  `newlyUnlocked`（[prev, curr] 区间跨阈值检测）。
- 展示接线：App.vue 侧栏提示行与 `select()` 解锁 toast；CapabilityList 徽标 tooltip；
  CapabilityDetail 锁定横幅（新增 `totalUses` prop）。
- 无后端改动：计数仍在前端本机存储（`elwright-capability-uses`），解锁纯本地判定。
