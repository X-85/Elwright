# verification：代码浏览器找不到「复制文件路径」按钮（B6）

## 修复内容

- 「复制文件路径」按钮从 13px 纯图标升级为「图标 + 复制路径」文字按钮。
- 绝对路径拼接改 `joinProjectPath`：按项目根分隔符风格拼接（Windows `\`），
  归一根尾与相对路径头部分隔符，混合分隔符问题消除。
- 新增 e2e：`copy-path.spec.ts` + `copy-path-invoke-helper.ts`
  （`__TAURI_INTERNALS__` mock 注入，全链路驱动 tauriBridge）。

## 验证记录

- 【自动化】Playwright e2e `copy-path.spec.ts`（本机系统 Chrome，
  2026-09-02）：选项目 → 展开 src → 打开 App.java → 点「复制路径」→
  剪贴板精确等于 `D:\demo\myproj\src\main\java\com\example\App.java`
  → 「已复制」通知可见。**1 passed**。
- 【自动化】全套 e2e 13/13 通过（既有 12 例无回归）。
- 【自动化】`npm run lint` 0 错误；`npm test` 15 文件 86 测试全过；
  `npm run build` 成功；`cargo fmt --check` / `clippy --lib --tests
  -D warnings` 全绿。
- 【自动化】截图留证：`screenshots/copy-path-button.png`
  （文件打开后的工具条，「复制路径」按钮与「已复制」通知可见）。
- 【手测】（留给用户，装新 msi 后）代码浏览器打开任意文件 → 点工具条
  「复制路径」→ 在资源管理器地址栏粘贴回车，应直接定位到该文件。
