# plan：最近项目支持删除

日期：2026-08-31 · 类型：enhancement · 来源：用户反馈 Q24

## 需求

代码浏览器「最近项目」列表支持逐条移除。语义决策：

- 删除最近项目时**连同其名下最近文件**清除（属于同一"足迹"数据）；
- **收藏与书签保留**——它们是用户显式沉淀的数据，不是足迹；
- 不弹确认框：最近列表是可再生数据（重新打开项目即回来），保持轻交互；
- 删除不影响磁盘上的项目文件（只动 `~/.elwright/code-browser.json`）。

## checklist

- [x] core：`remove_recent_project(store, root_path) -> bool` + 单测（项目+其文件删除、
      其他项目数据与收藏/书签保留、不存在返回 false 幂等）
- [x] IPC：`code_browser_recent_remove_project(projectRoot) -> RecentStore`（load→remove→save→返回更新后列表）
- [x] main.rs 注册；IPC mock runtime 测试（open 临时项目→remove→断言消失→幂等再删，自清理）
- [x] bridge.ts：接口 `codeBrowserRecentRemoveProject` + 浏览器 stub（空操作）+ tauri invoke
- [x] CodeBrowserView.vue：最近项目行拆「打开按钮 + × 删除按钮」，删除后用返回的 store 刷新
      两个列表 + 成功 toast
- [x] style.css：`.cb-recent-open`（flex:1）+ `.cb-recent-remove`（18px 方形，hover 红）
- [x] 闸门：cargo fmt/clippy(-D warnings)/test 全绿 + eslint/vitest/build 全绿
