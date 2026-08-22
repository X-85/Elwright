# 验证记录（2026-08-21，家里 macOS，rustc 1.98.0）

## 单元测试

- `cargo test`：**9/9 通过**（新增 resolve_root 三测：env 覆盖 / cwd 上溯语义 / bundle 布局探测与无误报；既有 6 测回归）。

## CLI 冒烟（新解析）

| 场景 | 结果 |
|---|---|
| `ew ls`（cwd 在仓库内，第 2 档上溯） | ✅ 24 项 |
| `ELWRIGHT_ROOT=$PWD ew ls`（第 1 档 env 覆盖） | ✅ 24 项 |

## Tauri 打包（release）

- `tauri build`（须自 `src-tauri/` 调 `../src/node_modules/.bin/tauri`；`npx tauri` 在 `src/` 下找不到配置）→ release 编译 1m03s 通过。
- 首次 dmg 打包失败：AppleScript 美化窗口时 Finder 超时（-1712，已知抖动）；重试即成功。
- 产物：`src-tauri/target/release/bundle/dmg/Elwright_0.1.0_aarch64.dmg`（8.2 MB）。

## bundle 资源落位核验（待回填）

挂载 dmg 实测 `Elwright.app/Contents/Resources/`：
- ✅ `capabilities.json`（map 映射生效）
- ✅ `resources/`（docs 全部落位，含 7 个 SOP 与知识文档）
- ✅ 无 `_up_` 目录（plan 风险项排除：map 目标路径显式指定，不产生层级逃逸）

启动冒烟（最终形态）：挂载 dmg → 自 `/tmp`（cwd 无注册表）执行卷内 .app 二进制 → 进程存活不崩溃，第 3 档资源目录探测在真实 bundle 布局下工作。GUI 交互留用户复验。

## 未验证项（如实记录）

- 打包后 GUI 交互（双击 .app 后列表/文档/技能降级）——环境限制，留用户复验。
- Windows msi：需公司机装 MSVC 后按 design §3 执行。
- dmg 安装流程与未签名 Gatekeeper 提示：留发版时用户复验。
