# change-note：工作台「今日记录」被其他页面遮挡（C2）

## 问题（真机点验 v0.1.13，Windows）

用户反馈工作台「今日记录」看不到，被其他页面遮挡。

## 根因

`.workbench` 采用 2×2 grid（Todo | 今日记录 ‖ 常用能力 | 实用工具，
changelog 明确「布局 2×2」），但行高用的是
`grid-auto-rows: minmax(230px, auto)`——**auto 上限让格子随内容收缩/膨胀**：

1. 矮窗口下，第一行（Todo/今日记录）之后的格子被推出视口，工作台容器
   虽可整体滚动，但格子没有固定高度保底，用户看到的就只有半屏 Todo——
   观感即「今日记录被遮挡」。
2. 今日记录预览切换后长文会把 `.wb-note` 格子撑到数千像素，
   `.wb-note-editor` 的 `flex:1 + min-height:0` 又会被压成 0 高——
   编辑态内容不可见的另一种形态。

## 修改范围（仅 CSS）

- `.workbench`：行高下限 230px → **260px** 并保留 auto 上限；配合格子级
  `overflow: hidden`，行高由 grid 决定、内容不再参与行高计算。
- `.wb-todo / .wb-note / .wb-panel`：加 `overflow: hidden`，内部各自滚动。
- `.wb-note-editor`：补 `overflow-y: auto`（格子裁剪后 textarea 必须自己
  吃掉长文溢出，否则编辑内容不可见）。

净效果：四宫格永远是完整 2×2（每格 ≥260px 高），超出视口由工作台容器
滚动到达；格子内长文由编辑器/预览/列表自身的滚动条消化。

## 风险

- 低。纯 CSS，不改任何逻辑与存储；todo-list/fav-list/tools 原本就有
  `overflow-y: auto`，行为不变。

## 验证

- 【自动化】新增 `e2e/workbench-note-layout.spec.ts` 2 用例：
  ①压扁视口（400px 高）后今日记录格子仍 ≥259px、容器可滚动到达；
  ②200 行长文下编辑器自身出现滚动、格子高度不膨胀出格局。
  **用 git stash 对旧 CSS 验证过测试有效性：旧 CSS 下用例 ① 失败、
  ② 通过（符合两形态根因）；新 CSS 下 2/2 通过。**
- 【自动化】全套 e2e 15/15（13+2 新增，无回归）；vitest 86/86；
  lint/build；fmt/clippy（无 Rust 改动，例行）。
- 【自动化】截图留证：`screenshots/note-long-text.png`。
- 【手测】（留给用户，装新 msi 后）工作台四格齐全；把窗口压矮后今日
  记录仍在（容器滚动可达）；预览长文不把格子撑爆。
