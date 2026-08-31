# verification：代码浏览器树不可见 + 预览栏压窄修复

日期：2026-08-31

## 【自动化】本地闸门

- `npm run lint`：exit 0（0 告警）
- `npm test`：10 文件 60/60 通过
- `npm run build`：成功（chunk 500kB 警告为存量）
- cargo 侧零改动，未重跑

## 【自动化】根因复现实验（node + 项目自带 vue + jsdom）

`/tmp/vue_map_vfor_test.cjs`（一次性脚本，未入库）：

- `ref(new Map())` + 模板 `v-for="(entries, rel) in treeCache"`：
  `.set('', [3 条])` 渲染为 `(root):2`、`.set('src', [1 条])` 渲染为 `1:2`
  —— 实证 v-for 遍历 Map 拿到 `[key, value]` 对儿与数字下标。
- 改为普通对象后同样模板正确渲染 `(root):3`、`src:1`。

## 【手测】真机点验（tauri dev，macOS，通过 ZCode 桌面控制实测 + 用户本人操作）

- 修复经 vite HMR 热更到用户正在运行的 `tauri dev` 实例（无重启）。
- 重开最近项目 springbootDemo1：根层树行出现（src / .DS_Store / .gitignore / springbootDemo1.iml）。
- 点击 `src` 目录：二级懒加载展开正常（Main.java / package-info.java）。
- 打开文件：预览区渲染行号 + 内容（.iml 11 行全显），收藏/复制路径/转为 Todo/终端定位/发送到 AI 按钮在位。
- 预览栏恢复全宽（grid-column 修复后 .content 第一列 300-380px 约束解除）。
- 用户本人随后自行操作（开 3 个 tab、移动窗口）确认可用。

## 遗留

- Playwright e2e 无法覆盖此路径（浏览器桥 `chooseProjectDirectory` 返回 null），
  该接缝回归依赖真机点验——已在 PENDING-REAL-MACHINE-CHECKLIST 语境中，见台账 Q22。
- 组件级挂载测试需要 @vue/test-utils（当前未引入），Map→Record 坑以源码注释 + 本文档留档。
