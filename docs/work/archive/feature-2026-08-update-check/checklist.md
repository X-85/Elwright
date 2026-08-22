# 检查清单

- [x] core/version.rs（normalize + is_newer + 4 组单测）
- [x] main.rs check_update IPC + opener 插件注册 + capabilities/default.json 授权
- [x] bridge.ts：checkUpdate/openExternal 双适配器 + compareVersions + UpdateInfo
- [x] App.vue 侧栏按钮/消息/下载链接 + style.css
- [x] vite.config.ts __APP_VERSION__ 注入（真相源 tauri.conf.json）
- [x] cargo test 18/18、npm run build、cargo build --release
- [x] GUI 实测：已是最新 / 发现新版本两分支
- [ ] CI 三平台 + dmg/msi 打包绿
- [ ] 实机（桌面壳）点击检查更新确认（用户）
