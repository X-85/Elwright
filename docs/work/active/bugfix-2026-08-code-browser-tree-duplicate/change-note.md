# change-note：代码浏览器展开目录后子级出现两次

- 日期：2026-08-31
- 类型：bugfix
- 触发：用户真机反馈「打开 springbootDemoV1，展开 src 后 main / test 目录出现 2 次」（Q23-1）

## 根因

`CodeBrowserView.vue` 模板结构为：**外层 `<template v-for="(entries, rel) in treeCache">`
遍历全部已缓存层级 + 模板内手工嵌套子级（写死三层 cb-sub / cb-sub2）**。

展开 `src` 后 `treeCache` 有了 `'src'` 键，外层循环会把 `src` 的子级再渲染成一份
**顶级列表**（`v-show="expanded.has('src')"` 为 true），与嵌套在 src 下那份重复
→ main / test 各出现两次；继续展开更深目录会重复更多份。

该缺陷此前被 Q22 的 Map bug 掩盖（v-show 恒 false 时整树不渲染，重复问题一并不可见）；
Q22 修复树可见后暴露。

## 修复

改为**扁平化渲染**：

- 新增 `visibleRows` computed——按 `expanded` 递归下钻 `treeCache` 生成
  `{ entry, depth }[]` 单层列表（懒加载、任意深度）；
- 模板只渲染这一个列表，缩进用 `paddingLeft: depth * 14px`；
- 删除模板手工嵌套的三层结构（顺带解除「最多展示两层」的旧限制，核心本就支持 8 层深度）；
- 树行 ★ 收藏按钮统一到所有层级的文件行（原模板仅第一层文件有）。

## 影响面

- 仅 `src/components/CodeBrowserView.vue`；Rust core / IPC / bridge 零改动。
- 行为变化：目录树可展开深度 3 层以内 → 任意深度（与 core `MAX_TREE_DEPTH=8` 对齐）。
