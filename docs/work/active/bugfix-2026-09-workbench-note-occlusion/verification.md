# verification：工作台「今日记录」被遮挡（C2）

## 修复内容（纯 CSS）

- `.workbench` 行高 `minmax(230px, auto)` → `minmax(260px, auto)`。
- `.wb-todo/.wb-note/.wb-panel` 加 `overflow: hidden`（行高由 grid 决定）。
- `.wb-note-editor` 补 `overflow-y: auto`（长文自身滚动）。

## 验证记录

- 【自动化】`e2e/workbench-note-layout.spec.ts`（本机系统 Chrome，
  2026-09-02）：
  - 用例①「矮窗口（400px）下今日记录格子保持 ≥259px 且容器可滚动」
    ——**旧 CSS 下失败（复现点验现象）、新 CSS 下通过**（stash/pop
    对照验证，测试有效性成立）；
  - 用例②「200 行长文：编辑器自身滚动、格子高度不膨胀」——新旧均通过
    （该形态旧 CSS 靠 auto 上限间接成立，新 CSS 由 overflow:hidden 保证）。
  - 结果：**2/2 passed**。
- 【自动化】全套 e2e 15/15；vitest 15 文件 86 测试全过；`npm run lint`
  0 错误；`npm run build` 成功；cargo fmt/clippy 例行全绿（无 Rust 改动）。
- 【自动化】截图留证：`screenshots/note-long-text.png`（长文场景工作台
  2×2 格局完整）。
- 【手测】（留给用户，装新 msi 后）①工作台四格齐全；②窗口压矮后今日
  记录仍可达（工作台内滚动）；③今日记录预览/编辑长文不撑破格子。
