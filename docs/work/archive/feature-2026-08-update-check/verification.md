# 验证记录

- cargo test：18/18 全绿（新增 `core::version` 4 组：v 前缀剥离、patch/minor/major 判定、数值比较非字典序（0.10>0.9）、短形式/预发布后缀容忍）。
- `cargo build --release` 通过（新增 tauri-plugin-opener 依赖后，CI dmg/msi 打包路径编通）。
- `npm run build` 通过（`__APP_VERSION__` 注入正常）。
- 浏览器预览 GUI 实测（vite dev + In-app Browser）：
  - 最新版分支：点击「检查更新」→ 显示「已是最新版本（v0.1.0）」（真实 GitHub API 响应）。
  - 有更新分支：临时把注入版本改为 0.0.1 → 显示「发现新版本 v0.1.0（当前 v0.0.1）」+「前往下载 →」按钮渲染正常。
  - 「前往下载」在 IAB 自动化环境被弹窗拦截未开出新 tab（环境特性）；桌面壳走原生 opener（@tauri-apps/plugin-opener openUrl），不受此影响。恢复真实版本号。
- 桌面壳 IPC 路径（check_update / opener）依赖 CI 三平台编译 + 后续实机验证；实机首次点按钮由用户确认。

## 已知边界

- GitHub API 限额：未认证 60 次/小时/IP——手动触发场景无压力；打不开时报「检查更新失败：无法访问 GitHub」中文文案。
- 预览模式版本号来自 tauri.conf.json 注入（构建时静态），桌面壳来自 CARGO_PKG_VERSION——发版三处同步约定下两者一致。
