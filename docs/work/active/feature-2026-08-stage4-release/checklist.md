# 检查清单

- [x] core：resolve_root 三段式（env > cwd > probes）+ 纯函数化便于单测
- [x] 单测：三级解析各自覆盖（9/9 全过）
- [x] ew.rs / main.rs 接入（main.rs 用 OnceLock 存 setup 期解析结果）
- [x] tauri.conf.json bundle.resources（map 形式，实测无 `_up_`）
- [x] cargo test 全绿（9/9）
- [x] tauri build 出 dmg（8.2MB）；挂载核验资源落位 + 卷内启动冒烟
- [ ] 文档：verification / desktop-ui Feature（architecture+changelog）/ README 发布说明
- [ ] 提交推送，CI 全绿
- [ ] （后续）Windows msi：公司机装 MSVC 后按 design §3
- [ ] （后续）GitHub Release + 版本三处同步（Cargo.toml / tauri.conf.json / package.json）
