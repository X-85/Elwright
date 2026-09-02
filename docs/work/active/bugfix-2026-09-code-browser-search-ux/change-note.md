# change-note：代码浏览器搜索体验四子项（B4）

## 问题（真机点验 v0.1.13，Windows）

①输入后必须点搜索按钮才生效，期望即时模糊搜索；
②结果每条都显示完整路径，占空间太多，期望只显示文件名；
③内容搜索点了按钮后很卡（项目文件多）；
④搜索结果偶现清不掉，搜索按钮直接置灰。

## 根因（逐项）

- **①非即时**：搜索只绑 `@keyup.enter` 与按钮 click，无 input 触发路径。
- **②全路径**：结果按钮主文本直接渲染 `hit.path`，未区分文件名/路径层级。
- **③卡顿（复合）**：
  a. `search()` 达到 200 条上限后只在回调入口 `return`，`walk_files` 仍要
     空跑完整棵目录树（filename 模式命中后更是从不推进上限标志）；
  b. 即时搜索缺失放大了体感——用户反复点按钮，每次都全树重扫。
- **④清不掉/按钮置灰（真 bug，panic 级）**：snippet 截断
  `&trimmed[..200]` 按**字节**切——中文行命中时切在 UTF-8 字符边界内
  （`中文内容`×N 场景必现），后端 panic → 该 IPC Promise 永久 pending →
  `searching` 恒为 true → 搜索按钮永久置灰、结果无法再触发更新。
  已用独立探针复现：`end byte index 200 is not a char boundary`。

## 修改范围

- `src-tauri/src/core/code_browser.rs`（`search()`）
  - 上限推进改为 `Cell<bool>` 标志，达 200 条**彻底停止遍历**；
    filename 模式命中后同样推进（修 ③）。
  - snippet 截断改按**字符**：`chars().take(200)` + 省略号（修 ④ panic）。
  - 新增单测：`search_snippet_truncates_on_char_boundary`（中文长行）、
    `search_filename_mode_respects_result_cap`（上限推进）。
- `src/components/CodeBrowserView.vue`
  - 输入防抖 300ms 即时搜索（`@input`），Enter 立即；模式切换也触发（修 ①）。
  - 竞态防护：`searchSeq` 序号——慢响应返回时若已过期则丢弃，不覆盖新
    结果、不复位 `searching`（修 ④ 的次生形态）。
  - `clearSearch`：输入、结果、定时器、序号一次清空；有内容/结果时显示
    「清除搜索」按钮（X）。
  - 结果条目主文本改 `hit.name`（文件名），完整路径弱化为次要 span
    `cb-hit-path`（修 ②）。
- `src/style.css`：`.cb-hit-path` 次要路径样式（11px / 55% 透明 / 省略号）。
- `src/e2e/search-ux.spec.ts`（新增 4 用例）：即时触发（IPC 计数）、
  文件名优先展示（截图留证）、一键清除后可再搜、竞态（3s 慢响应 +
  清除后不回灌）。mock 沿用 `__TAURI_INTERNALS__` 注入架构；状态通过
  页内 `__ELWRIGHT_SEARCH_STATE__` 对象引用设置（initScript 参数按值
  序列化，闭包变量改动不生效——调试踩坑已注释）。

## 风险

- 低。`search()` 行为收紧仅限提前终止与 panic 修复；前端防抖 300ms 在
  手输速度下不会逐键打爆 IPC，Enter 仍可立即触发。

## 验证

- 【自动化】独立探针（GNU）：中文/英文截断、capped 推进断言 ALL PASS。
- 【自动化】单测 2 个新增；`cargo test --lib` 执行留 CI（本机 GNU 限制）。
- 【自动化】e2e 4/4；全套 19/19；vitest 86/86；lint/build 全绿。
- 【自动化】Rust 门禁：fmt clean、clippy lib+tests 0 警告、ew 构建成功。
- 【自动化】截图：`screenshots/search-result-filename-first.png`。
- 【手测】（留给用户，装新 msi 后）大项目输入即搜、清空按钮可用、
  中文内容搜索不再卡死按钮。
