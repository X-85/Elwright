# verification：能力渐进式发布后续（成长体系透明化）

日期：2026-08-31

## 【自动化】

- vitest：`lib/growth.test.ts` 6 用例全绿（tier1 恒解锁 / 门槛判定 / 缺门槛恒锁 /
  nearest 选剩余最少且缺门槛不参与 / 跨阈值检测含倒退与 tier1 不触发）；
  全量 vitest 66/66；eslint 0 告警；vite build 成功。cargo 侧零改动。

## 【手测】浏览器预览四层实测（IAB，localhost:5173，同一套 Vue 代码）

1. 核心视图：默认 3/4 项（weekly-report 隐藏）；侧栏提示行
   「距解锁「周报生成」还差 3 次（累计已用 0/3 次）；规则与记录仅保存在本机」。
2. 「查看全部能力」：列表出现「周报生成 …待解锁」徽标。
3. 点开 weekly-report：详情横幅「该能力尚未解锁：累计使用任意能力 3 次后自动解锁
   （当前 0/3）。使用记录仅保存在本机。」，调用按钮禁用。
4. 预置 localStorage 已用 2 次后点击 text-stats（第 3 次使用）：
   toast「🎉 已解锁进阶能力「周报生成」——在列表中即可使用」弹出；
   提示行消失；周报生成进入核心视图且无待解锁徽标；uses 落盘 {"text-stats":3}。
   验证后已清除浏览器测试用 localStorage。

## 【自动化】e2e 对齐新行为（CI 首跑抓到，已修）

- `app.spec.ts` 首用例改为断言新行为：默认核心视图 3 项 + 提示行进度 +
  查看全部后 4 项且 weekly-report 带待解锁徽标。
- `app-smoke.spec.ts` 离线 SOP 用例：weekly-report 现为进阶档位，追加后置
  initScript 预置累计使用 3 次（注册序在 beforeEach 的 clear 之后）再验调用降级。
- 本地 Playwright 10/10 全绿。
