# 工程质量第三档 ESLint + Coverage — Verification

> 每条标注【自动化】/【手测】。命令在 `src/` 目录下执行（除 cargo 与全局脚本外）。

## 配置层

- [ ] **【自动化】** `cd src && npm run lint` 退出码 0，无任何告警（首次启用即 fail 级，禁止分阶段留 warning）
- [ ] **【自动化】** `cd src && npm run test:coverage` 退出码 0；输出报告四指标 ≥ 阈值（lines/functions/statements 70%, branches 60%）
- [ ] **【自动化】** `cd src && npm run vitest run` 退出码 0（coverage 不阻塞纯单测）

## CI 接缝

- [ ] **【自动化】** `.github/workflows/ci.yml` frontend job 内含 `npm run lint` 与 `npm run test:coverage` 两步
- [ ] **【自动化】** 矩阵三平台（macOS / Ubuntu / Windows）一致执行；CI 7/7 + frontend 6/6 全绿
- [ ] **【自动化】** release.yml（v0.1.11 触发时）frontend job 同样 6/6 全绿

## 文档

- [ ] **【自动化】** `docs/features/engineering-quality/README.md` 第三档状态 = 已完成；包含 ADR-002 链接
- [ ] **【自动化】** `docs/ROADMAP.md` 第三档条目带 ~~删除线~~ + ADR-002 链接
- [ ] **【自动化】** `docs/ROADMAP.md` 「未发版」「进行中」「历史」段均含 Q20

## 真机点验

- [ ] **【手测】** 故意留一行 `import { ref } from 'vue'` 未使用，`npm run lint` 必须非绿——确认 lint 真实拦截（CI 已隐含覆盖，但本机手动验一次心安）
- [ ] **【手测】** 故意在 `lib/preferences.ts` 加一个未导出函数（仅声明不调用），`npm run test:coverage` 应让 functions % 下降——确认阈值真实生效（首次设置阈值时验一次）
