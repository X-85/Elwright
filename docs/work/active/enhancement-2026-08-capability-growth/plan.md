# plan：能力渐进式发布后续——解锁规则透明 + 成长提示

日期：2026-08-31 · 来源：用户在 V2 剩余主干盘点中定向「先做能力渐进式发布后续」
ADR：[ADR-001-growth-transparency](../../features/progressive-capabilities/decisions/ADR-001-growth-transparency.md)

## checklist

- [x] `lib/growth.ts` 纯函数（isUnlocked / growthSummary / newlyUnlocked）+ vitest 6
- [x] App.vue：跨阈值解锁 toast + 侧栏提示行具体化（最近解锁进度）
- [x] CapabilityList：锁定徽标 tooltip 显示条件与进度（新增 `totalUses` prop）
- [x] CapabilityDetail：锁定横幅显示条件与进度（新增 `totalUses` prop）
- [x] capabilities.json：weekly-report → releaseTier 2 + unlockAfterUses 3（示例策略，见 ADR）
- [x] 文档回填：progressive-capabilities behavior/architecture/changelog + ROADMAP
- [x] 闸门：eslint 0 / vitest 66（含 6 新）/ build 成功；cargo 侧零改动

## 状态

ADR 与实施同轮完成（小任务，合并两阶段）。关键决策点（需用户知悉，可否决）：
**内置 weekly-report 升为进阶档位**——默认从核心视图隐藏、累计使用任意能力 3 次
解锁。理由与影响见 ADR-001 第 3 条。
