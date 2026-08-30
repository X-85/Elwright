# 工作工具栏变更记录

## 2026-08-25

- 第一阶段实现：Todo 清单 + 今日记录（`feature-2026-08-workbench-phase1`）。
  - core/workbench.rs（todos.json + notes/ 存储，日期校验防穿越，损坏降级）；
    commands.rs 7 条 IPC；WorkbenchView.vue；App.vue 顶栏导航。
  - 测试：core 单测 5、IPC 冒烟 4、vitest 4、Playwright 1 场景。
  - 范围调整：今日记录由后置提前进第一阶段（用户 2026-08-25 拍板）。
## 2026-08-22

- 将工作工具栏（Workbench）规划为独立 V2 Feature。
- 分为每日工作工具、实用转换工具、AI 联动和桌宠联动五个建设阶段。
