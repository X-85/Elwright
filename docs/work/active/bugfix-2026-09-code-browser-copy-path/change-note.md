# change-note：代码浏览器找不到「复制文件路径」按钮（B6）

## 问题（真机点验 v0.1.13，Windows）

用户打开 .java 文件后找不到「复制文件路径」按钮。

## 根因

1. **按钮是 13px 纯图标**（`Copy` 图标，无文字、`padding: 4px`），夹在
   收藏星标和「转为 Todo」文字按钮之间，可发现性极差——点验单原话
   「没有看到复制文件路径按钮」。功能本身存在，属于可用性缺陷而非缺失。
2. **点击后的行为也有 bug**：拼接绝对路径用 `projectRoot + '/' + path`，
   Windows 项目根（`D:\demo`，反斜杠结尾与否不定）拼出来是
   `D:\demo/src/...` 混合分隔符，粘贴到资源管理器/终端可能解析失败。

## 修改范围

- `src/components/CodeBrowserView.vue`
  - 按钮补文字标签「复制路径」（图标 + 文字，与「转为 Todo」「发送到 AI」
    同一视觉层级），title 保持「复制文件路径」。
  - 新增 `joinProjectPath(root, rel)`：按项目根的分隔符风格（含 `\` 即用
    `\`）拼接，并归一根尾/相对路径头部的分隔符，Windows/Unix 双端正确。
- `src/e2e/copy-path.spec.ts` + `copy-path-invoke-helper.ts`（新增）
  - 浏览器 e2e 注入 `__TAURI_INTERNALS__` mock（invoke/transformCallback/
    convertFileSrc/metadata），走真实 tauriBridge 全链路：
    选项目（plugin:dialog|open mock）→ 展开目录 → 打开文件 → 点「复制路径」
    → 断言剪贴板内容为 `D:\demo\myproj\src\main\java\com\example\App.java`
    → 「已复制」通知出现，并截图留证。
  - helper 以源码字符串嵌入页面（new Function），树 mock 按 rel 分层返回
    （对任意 rel 返回含同名子目录的列表会让 visibleRows 的 walk 无限递归）；
    反斜杠一律 `String.fromCharCode(92)` 运行时构造（Function.toString
    序列化会吃掉字面量转义）。

## 风险

- 低。UI 仅一个按钮改文字；`joinProjectPath` 只影响复制到剪贴板的字符串。
- e2e mock 架构（注入 Tauri internals）后续 B4/C2 等前端修复可复用。

## 验证

- 【自动化】Playwright（系统 Chrome 通道，本机）：copy-path 用例通过，
  剪贴板内容精确断言；全套 e2e 13/13（含既有 12 例无回归）。
- 【自动化】eslint 0 / vitest 15 文件 86 测试全过 / vite build 成功 /
  cargo fmt + clippy 0 警告（本轮无 Rust 改动，例行验证）。
- 【手测】（留给用户，装新 msi 后）打开 .java 文件 → 工具条可见
  「复制路径」按钮 → 点击 → 粘贴到资源管理器地址栏应直接定位文件。
