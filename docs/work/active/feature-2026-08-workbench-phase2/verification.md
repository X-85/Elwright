# verification：工作台第二阶段（常用能力 + 实用工具）

日期：2026-08-31

## 【自动化】

- vitest：capabilityRecents 4（空默认/toggle 去重持久化/recordRecent 去重置顶淘汰/
  损坏存储回退）+ convert 6（JSON 往返与中文报错/Base64 UTF-8 往返与非法报错/
  时间戳秒毫秒识别/日期往返）；全量 vitest 81/81。
- eslint 0 告警（preserve-caught-error 要求 catch 抛错带 cause，已补）；vite build 成功。
- Playwright e2e 10/10（`.wb-count` 与新区块撞名已作用域化为 `.wb-todo .wb-count`）。

## 【手测】浏览器预览（IAB）

- 工作台渲染 4 区块（Todo | 今日记录 ‖ 常用能力 | 实用工具）。
- 常用能力空态文案正确；实用工具三个 tab 在位，JSON 格式化
  `{"b":2,"a":[1,2]}` → 输出 2 空格缩进排序保持原序。

## 【手测】真机（待用户）

- 桌面端使用能力后工作台「最近使用」出现条目、★ 收藏、点击跳转工具箱选中详情。
