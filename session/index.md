# 会话问题索引

| 编号 | 问题 | 状态 | 最近进展 | 下一步 |
| --- | --- | --- | --- | --- |
| Q1 | 启用本次会话台账 | 已解决 | 已创建并校验基础台账结构 | 后续会话按协议维护 |
| Q2 | Elwright 自动化交互测试 | 已解决 | PR #1（含 Playwright 冒烟）已合并 main（7e222ab），CI 全绿；早期 run 失败由后续 e2e 重构与平台修复覆盖 | 无遗留 |
| Q3 | 工程质量第二档收口（tier2） | 已解决 | 合入 main（6f84109）+ 两平台 CI 修复后 7/7 全绿 | 归档待用户确认 |
| Q4 | 工作台第一阶段（Todo + 今日记录） | 已解决 | 合入 main（c0c46b1）CI 7/7 全绿 | 归档待用户确认 |
| Q5 | 双线分支合并协调（tier2/workbench vs codex） | 已完成 | PR #1 已合并 main（7e222ab），main CI 7 job 全绿；路线图/进度指针已同步（7bd0f9e） | v0.1.6 发版与任务目录归档待用户决策 |
| Q6 | 本分支下一步评估（roadmap 复查） | 已解决 | traffic-light 补做、文档同步、两轮 rebase 全部完成（详见 events Q6 各轮） | 无遗留；等合并流程推进 |
| Q7 | 归档 + v0.1.6 发版 | 已解决 | 22 目录归档；v0.1.6 发版一次全绿；但真机冒烟发现资源收藏 IPC 恒失败（见 Q8），v0.1.6 已被 v0.1.7 取代 | 真机安装直接用 v0.1.7 |
| Q8 | v0.1.6 功能冒烟（抓到发布阻断 bug） | 已完成 | bug 已修随 v0.1.7 发版；真机补点验按用户指示跳过，留档 PENDING 清单 | 清单随 v0.1.8+ 复验 |
| Q9 | 代码浏览器阶段① | 已完成 | 已合并 main 并随 v0.1.8 发版；真机点验跳过留档、任务目录归档 | 阶段③第一批已接续（见 Q12） |
| Q12 | 代码浏览器③（收藏/书签/发送到 AI + Todo 联动/终端定位） | 已完成 | 第一批随 v0.1.9 发版；第二批完成并进 main（ecda801，CI 全绿）；真机点验留档 | 真机点验待用户；随下版发版 |
| Q13 | 流程失误：绕过 PR 直推 main | 已解决 | wt-main 分支未切回导致直推；内容完整 CI 全绿，未重写历史 | 教训：提交前查分支；反引号信息用安全方式传入（Q14 一度重犯，已当场纠正） |
| Q15 | 路线图未做功能盘点 | 已解决 | 主干未完成 8 项 + 后置 2 项已列明；ROADMAP 漂移已同步（PR #5，370bb7b） | 按盘点结果排期 |
| Q16 | AI 对话阶段③④（能力协作 + 流式/取消） | 已完成 | 阶段③ PR #6 合并（4b70c27）；阶段④ 按 ADR-003 落地，PR #8 合并（84e9667，CI 全绿）；顺手修 code_browser 测试并行撞名 flake | 真机点验（流式体验/能力协作全链路）待用户；v0.1.10 待发（攒 4 个功能） |
| Q17 | 发版 v0.1.10 | 已解决 | 按用户指令发版，本地五道闸全绿（cargo 62+1+6+4+1、strict clippy 0 警告、fmt、vitest 43、build），release.yml 一次全绿（dmg 2m18s + msi 3m24s + Publish 11s），tag v0.1.10 + 完整 changelog 上 GitHub Release。ROADMAP 当前版本/里程碑/进行中三处同步更新；台账 Q16 收口 | 真机点验项延续挂起（流式/能力协作全链路 + 历史清单）；下一阶段待用户定向 |
| Q10 | CI 教训沉淀 | 已解决 | Windows IPC origin 需 cfg 区分；本地 clippy 须带 -D warnings（新 IPC 测试/命令参数两处踩坑） | 后续新增 IPC 测试照抄 terminal_ipc.rs 的 origin 处理 |
| Q18 | 代码浏览器阶段④（受控补丁编辑） | 已完成 | ADR-001 「再评估」先合（PR #9，6f5171a）；实施 PR #10 合并（90bc346，CI 7/7 全绿）。后端 core::patch 10 单测 + IPC mock runtime 3 例；前端 lib/patch.ts + PatchPreviewDialog + ChatView diff 识别 + Bridge 4 方法 + patch.test.ts 4 例；行为/架构/changelog/ADR 验证回填 + chat ③④回填（v0.1.10 漏的）。本地 87 cargo + 47 vitest + strict clippy + fmt + build 全绿 | 真机点验待用户（选中→AI 回复→应用补丁→撤销 + 敏感路径拒绝 + 写入冲突兜底） |
| Q19 | 设置中心：模型档案（多套 LLM 配置切换） | 已完成 | ADR-001 模型档案 PR #11 已合并（65a13cb）；实施 PR #12 已合并（92597f8，CI 7/7 全绿）。core::llm 5 单测 + CLI `ew config profile list/show/use/add/remove/rename` 6 子命令（key 脱敏 + 错误 exit 1）+ 5 IPC（main.rs 注册 + mock runtime 5 例）+ Bridge 5 方法 + LlmSettings.vue 档案下拉/新建/清单删除 + vitest 4 例。本地五道闸全绿（cargo 97 + strict clippy + fmt + vitest 51 + build）。文档回填 settings-center behavior/architecture/changelog/README + ROADMAP 「设置中心后续阶段」标记完成 | 真机点验（建 2 profile → 切换 → ChatView 头部变化；flat-only 旧配置兼容；删除激活档案回退 flat）待用户 |
| Q20 | 工程质量第三档（ESLint + coverage 阈值） | 已完成 | ADR-002 ESLint+coverage PR #13 已合并（f25501f）；实施 PR #14 已合并（21cfbc1，CI 7/7 全绿）。ESLint 10 flat config（`typescript-eslint` + `eslint-plugin-vue`，规则 `no-unused-vars` + `vue/no-unused-components` + `vue/no-unused-vars` + `no-explicit-any` warn；TS/Vue 下 `no-undef` 关闭；vue 风格规则 off 留 prettier 独立 ADR）；vitest coverage v8 provider（include `lib/**/*.ts` exclude `lib/bridge.ts` facade + test 文件，thresholds lines/functions/statements 70% / branches 60%）；CI frontend job 加 `npm run lint` 与 `npm run test:coverage` 两步；vitest 51 → 60（preferences 副作用新增 9 例）；实测覆盖率 95/76/100/95；TypeScript 7.0.2 → 6.0.3（typescript-eslint 8.68 显式拒绝 TS 7）。本地六道闸全绿（eslint + vitest + coverage + vite build + cargo fmt + clippy + test）。文档回填 engineering-quality README 表格 / changelog / ADR-002 链接 + ROADMAP 第三档「按需」→「已完成」 | 真机点验（故意留未使用 import → lint fail；故意在 lib/preferences.ts 加未调用函数 → coverage % 下降）已写入 verification.md【手测】段，本档不阻塞 CI；待用户点测 |
