# 变更日志（Changelog）

## 2026-08-31 · 最近项目支持删除（Q24）

- 最近项目列表每行新增 × 删除按钮：core `remove_recent_project`（连同项目名下
  最近文件清除；收藏/书签保留）+ IPC `code_browser_recent_remove_project`
  + Bridge 三端方法 + 前端删除行与成功提示。
- 配套测试修复：IPC 测试新增互斥锁（两用例并行读写真实用户层会竞态丢更新）、
  收藏/书签断言改增量式（不再假设用户层为空）。
- 真机验证：删除行消失 + 持久化 + 项目名下最近文件同清；系统目录选择器
  → 打开项目全链路首次真机验证通过。
- 任务目录：`docs/work/active/enhancement-2026-08-code-browser-recent-remove/`。

## 2026-08-31 · bugfix：展开目录后子级出现两次（Q23）

- 外层 `v-for treeCache` 会把每个已缓存层级再当顶级列表渲染一遍（与模板手工嵌套
  重复），展开 src 后 main/test 出现两份。改为 `visibleRows` computed 扁平化渲染
  （按 expanded 递归，任意深度，缩进 paddingLeft）；删除写死三层的嵌套模板；
  ★ 收藏按钮统一到所有层级文件行。任务目录
  `docs/work/active/bugfix-2026-08-code-browser-tree-duplicate/`。

## 2026-08-31 · bugfix：树不可见 + 预览栏压窄（Q22）

- 树永不渲染：`treeCache` 用 Map 存储导致模板 `v-for` 遍历拿到 `[key, value]`
  对儿 + 数字下标，`v-show` 恒 false——改为普通对象（Record）修复。
  该缺陷因阶段①真机点验跳过 + e2e 浏览器桥选不了目录而漏网。
- 预览栏被压进 300–380px 窄列：`.code-browser` 补 `grid-column: 1 / -1`
  （对齐 chat-view 等全宽视图惯例）。
- 配套：App.vue 报错 toast 移出 toolbox-only 模板，代码浏览器 IPC 失败不再静默。
- 任务目录：`docs/work/active/bugfix-2026-08-code-browser-tree-invisible/`。

## 2026-08-23 · Feature 立项

- 将代码浏览器纳入 V2 主干优先路线。
- 确定第一阶段为本地项目只读查看和用户主动选择的代码上下文。
- 明确不替代完整 IDE，不默认扫描或上传代码，不在第一阶段修改源文件。

## 2026-08-24 · 增加轻量代码跳转规划

- 将接口到实现类、JavaBean 类型到类定义纳入近期阶段。
- 采用按需扫描的轻量符号索引，避免第一阶段引入常驻高内存 Java Language Server。
- Java Language Server 和完整语义导航后置评估。

## 2026-08-31 · 阶段③第一批（研发上下文联动）

- 收藏文件、代码书签（行级，面板跳回）、发送到 AI（确认面板 + 对话预填）。
- 存储：code-browser.json 扩 favorites / bookmarks；新增两条切换 IPC。
- Todo 联动、终端定位、流程图关联留第二批（流程图依赖工程图 MVP）。

## 2026-08-31 · 阶段③第二批

- 转为 Todo：文件一键创建含代码位置标记（`绝对路径:行号`）的工作台 Todo；
  工作台侧标记可点击跳回代码浏览器。
- 终端定位：集成终端新 tab 打开文件所在目录（openTab 支持 cwd）。

## 2026-08-31 · 阶段④（受控补丁编辑）

- ADR-001 完成「再评估」：明确不引入 Java LSP，落地「受控 patch 编辑」：
  diff 预览 + 逐 hunk 接受/拒绝 + 单文件写入 + 快照回滚。
- 后端 `core::patch` 提供 `parse_unified_diff` / `apply_hunks_to_content` /
  `is_sensitive_path` / `sha256_hex` / `build_preview` / `apply_preview` /
  `revert_snapshot_in`，10/10 单元测试通过（多文件 / 上下文不匹配 / 空文件 /
  敏感路径 / SHA-256 / 快照往返）。
- 新增 IPC：`apply_patch_preview` / `apply_patch_apply` /
  `apply_patch_revert` / `apply_patch_snapshots`，
  mock runtime 三例覆盖预览不落盘 / apply→revert 闭环 / 敏感路径拒绝。
- 前端 `src/lib/patch.ts` 提供 `extractFirstDiff` + `renderDiffLines`；
  `PatchPreviewDialog.vue` 三栏渲染 + 逐文件勾选拒绝 + 快照 id 回滚按钮；
  `ChatView.vue` 在助手消息内识别 `\`\`\`diff` 围栏后露出「预览并应用到代码」入口。
- 路径黑名单：`.env` / `.pem` / `.key` / `.ssh` / `.aws` / `node_modules` /
  `target` / `.git` 一律写入 warnings，即便用户确认也拒绝落盘。
- 快照存 `~/.elwright/code-browser/applied-patches.json`；用户可按快照 id 一键回滚到写入前内容。
