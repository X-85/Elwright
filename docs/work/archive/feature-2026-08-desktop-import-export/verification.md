# 验证记录（2026-08-22，家里 macOS）

## 单元测试（cargo test）

22 通过 / 0 失败。新增 4 例：

- `registry::tests::overlay_merges_and_shadows` — 合并视图（base 2 + overlay 2 = 3，同 id 遮蔽）、origin 标记（custom/builtin）、双根文件解析（遮蔽条目读 overlay 文件、仅内置文件回退 base）。
- `registry::tests::overlay_missing_registry_falls_back_to_base` — overlay 目录无 capabilities.json / None 时退化为纯 base。
- `export::tests::import_creates_target_root_when_missing` — 装机首导：overlay 目录不存在时自动创建，base 不被污染。
- `export::tests::delete_only_custom_and_cleans_files` — 内置拒删；独占文件清理；共享文件引用计数（最后一个引用者删除时才清理）；注册表终态正确。

## CLI 端到端（临时用户根 `ELWRIGHT_USER_ROOT`，不污染真实 `~/.elwright`）

1. `ew export tech-grill /tmp/test-export.elw.json` → 导出成功（含 SOP 文件）。
2. `ew import /tmp/test-export.elw.json` → `已导入能力 tech-grill（含 1 个文件）`，落盘 `<user-root>/capabilities.json` + `<user-root>/resources/docs/tech-grill-sop.md`。
3. `ew ls` → tech-grill 行 SRC 列显示 `user`（遮蔽内置同名）。
4. `ew delete tech-grill` → `已删除自定义能力 tech-grill（清理 1 个文件）`。
5. 再次 `ew delete tech-grill` → `错误: tech-grill 是内置能力，不可删除`（删完后露出内置条目，拒删符合预期）。

## 前端构建

`npm run build` 通过（TypeScript + Vite，124.6 kB js）。

## 浏览器预览 GUI（vite dev + in-app browser）

- 页面加载 24 项能力，侧栏出现「＋ 导入能力…」按钮。
- 点导入 → toast 降级文案：`【预览模式】浏览器无法写入用户叠加层。真实导入请用桌面应用或 CLI：ew import <文件.elw.json>`。
- 选中 tech-grill → 详情头出现「⬇ 导出」按钮；内置项无删除按钮（正确）。
- 点导出 → 触发浏览器下载 `tech-grill.elw.json` + toast `已导出 tech-grill（含 1 个文件）`。
- 截图留档：`sess_af664187…/call_bb5f6039…png`。

## 桌面 debug 构建

`../src/node_modules/.bin/tauri build --debug --bundles app` 成功（注意：非登录 shell 需 `PATH="$HOME/.cargo/bin:$PATH"`），产出 `target/debug/bundle/macos/Elwright.app`，启动正常（进程 38057）。

### 真机 GUI 导入（待用户实操）

Agent 已备好测试材料：`~/Desktop/work-summary.elw.json`（由 debug 版 `ew export` 生成）。用户步骤：

1. 打开 debug app（或等 v0.1.1 正式包），点侧栏「＋ 导入能力…」，选桌面该文件 → 应见绿色 toast，列表「工作总结」带「自定义」徽标。
2. 详情页应有「🗑 删除」；点删（confirm 确认）→ toast「已删除…（清理 1 个文件）」，徽标消失（内置版仍在）。

Agent 侧已验证的近路证据：同一套 core 落盘逻辑在 CLI 端到端（临时用户根）全通过；GUI 侧按钮/徽标/确认框/toast 在浏览器预览验证通过；桌面壳仅多了 dialog 选择路径一步。轮询 `~/.elwright/` 约 10 分钟未见用户操作（不在电脑前），未阻塞收尾。
