# 修复进度台账：bugfix-2026-09-win-real-machine

> 真机点验（v0.1.13 Windows）暴露问题的逐项修复进度。
> 顺序：B3 → B5 → B6 → C2 → B4 → C4 → P1。单轮定时任务只修一项。

| 项 | 问题 | 状态 | 结论 / 提交 | 遗留 |
|---|------|------|-------------|------|
| B3 | .env 可打开未提示敏感（.pem 正常） | ✅ 完成 | `is_sensitive` 补 .env 全变体规则 + search content 模式跳过敏感文件；提交 `d054e6a` | 【手测】真机复验开 .env 与内容搜索；cargo test 待 CI |
| B5 | 符号跳转无响应，候选为空 | ⏳ 待修 | | |
| B6 | 找不到复制文件路径按钮 | ⏳ 待修 | | |
| C2 | 工作台「今日记录」被其他页面遮挡 | ⏳ 待修 | | |
| B4 | 搜索四子项（即时模糊/只显示文件名/卡顿/结果清不掉） | ⏳ 待修 | | |
| C4 | OpenAI 兼容端点配置后不可用 | ⏳ 待修 | | |
| P1 | 后端 elwright.exe 弹可见控制台窗口 | ⏳ 待修 | | |
| B2 | Java 着色不够美观 | 🚫 不修 | 非 bug，轻量高亮本为刻意取舍；是否立项 enhancement 由用户定 | 用户决策 |
| A4 | 升级后旧数据保留 | 🚫 跳过 | 首装无历史数据，不适用 | 无 |
| B8 | 浏览器预览降级提示 | 🚫 跳过 | 用户未测 | 无 |

## 环境备注（本机 GNU 工具链）

- `cargo test`：lib-test 目标不应用 `rustc-link-arg-tests`（cargo 边缘行为），
  本机无法运行测试二进制；build.rs 已扩展 GNU manifest 嵌入，独立探针可验证
  纯逻辑；测试执行交 CI（Windows MSVC + macOS）。
- 前端依赖：公司网络直连 npm 源会 SELF_SIGNED_CERT_IN_CHAIN / ETIMEDOUT，
  用内部 nexus 镜像安装；node 20 + jsdom 30 不兼容，以 `--no-save` 装
  undici@6 + 手工恢复 npm 半解包目录解决，未改 package.json。
