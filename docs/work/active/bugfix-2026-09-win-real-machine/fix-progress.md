# 修复进度台账：bugfix-2026-09-win-real-machine

> 真机点验（v0.1.13 Windows）暴露问题的逐项修复进度。
> 顺序：B3 → B5 → B6 → C2 → B4 → C4 → P1。单轮定时任务只修一项。

| 项 | 问题 | 状态 | 结论 / 提交 | 遗留 |
|---|------|------|-------------|------|
| B3 | .env 可打开未提示敏感（.pem 正常） | ✅ 完成 | `is_sensitive` 补 .env 全变体规则 + search content 模式跳过敏感文件；提交 `d054e6a` | 【手测】真机复验开 .env 与内容搜索；cargo test 待 CI |
| B5 | 符号跳转无响应，候选为空 | ✅ 完成 | 根因：walk_files 深度上限 8 吞掉第 9 层起深包（src/main/java/com/... 每层包名耗一层）+ 扫描无进度反馈；新增 MAX_WALK_DEPTH=24 解耦 + 扫描中文案；提交 `3a4ad54` | 【手测】真机复验符号跳转；单测待 CI |
| B6 | 找不到复制文件路径按钮 | ✅ 完成 | 按钮纯图标可发现性差 → 补「复制路径」文字；顺手修 / 拼接混合分隔符 → joinProjectPath；e2e mock 架构（__TAURI_INTERNALS__ 注入）首次落地；提交 `a75aa08` | 【手测】真机点按钮粘贴验证；e2e mock 可供 B4/C2 复用 |
| C2 | 工作台「今日记录」被其他页面遮挡 | ✅ 完成 | 根因：grid 行高 minmax(230px,auto) 的 auto 上限使格子随内容收缩/膨胀；纯 CSS 修复：行高下限 260px + 格子 overflow:hidden + 编辑器自身滚动；e2e 2 用例（stash 对照验证有效）；提交 `b057f48` | 【手测】真机压矮窗口看四格齐全 |
| B4 | 搜索四子项（即时模糊/只显示文件名/卡顿/结果清不掉） | ✅ 完成 | ①300ms 防抖即时搜索；②文件名优先+路径弱化；③200 条上限后停止遍历（旧实现空跑整树）；④真 bug：snippet 字节截断中文行 panic → 按钮永久置灰，改字符截断 + searchSeq 竞态防护 + 清除按钮；e2e 4 用例；提交 `09a1ea8` | 【手测】大项目即时搜索/中文搜索不再卡死；cargo test 待 CI |
| C4 | OpenAI 兼容端点配置后不可用 | ✅ 完成（代码修复到位，真机待验） | 根因：base_url 裸根（无 /v1）时 chat_url 拼 404 路径；chat_url 自动补 /v1（三调用点生效），探针 6 用例 + 单测更新；提交 `b3a86b8` | 【手测】用户真机测试连接（需内网网络，Agent 不可代验） |
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
