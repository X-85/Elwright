# 检查清单

- [x] ci.yml：rust 三平台矩阵（test + build ew）+ frontend（npm ci + build）
- [x] CLI 冒烟：ls / 降级 SOP / mock LLM 成功路径（断言本地复现）
- [x] concurrency 取消旧跑 + permissions 最小化 + workflow_dispatch
- [x] Windows `.exe` 处理 + 冒烟步骤显式 bash
- [x] YAML 语法校验
- [x] AGENTS.md 补 CI 说明
- [ ] 首次 push 后 Actions 首跑结果回填（三平台）
- [ ] 3b 合入后：补 tauri dmg/msi 制品 job（按阶段 4 设计文档）
