# change-note：代码浏览器选项目后树不可见 + 预览栏被压窄

- 日期：2026-08-31
- 类型：bugfix（真机点验跳过后遗留的两个阶段①缺陷 + 一个壳层静默缺陷）
- 触发：用户真机反馈「选完项目看不到文件，预览栏也比较小」（Q22）

## 现象

1. `tauri dev` 桌面端：选择项目目录后，左侧树**一行都不渲染**（目录/文件全无），无任何报错提示。
2. 整个代码浏览器被挤在内容区一条 300–380px 的窄列里，右侧预览区极窄。
3. 点击「刷新当前目录树」无任何变化、无任何提示。

## 根因（三处，互为放大）

1. **树永不渲染（主根因）**：`CodeBrowserView.vue` 用 `ref(new Map())` 存 `treeCache`，
   模板 `v-for="(entries, rel) in treeCache"`。**Vue 3 的 v-for 遍历 Map 得到的是
   `[key, value]` 对儿 + 数字下标**，不是 `(value, key)`：`rel` 恒为 0/1/2，
   `v-show="rel === '' || expanded.has(rel)"` 恒 false → 整棵树 `display:none`。
   用项目自带 vue + jsdom 实测：set 3 条渲染 `length:2`（对儿长度），set 1 条渲染
   `1:2`——键值对+下标实锤。
   为什么一直没暴露：阶段①真机点验按用户指示跳过留档；Playwright e2e 走 browserBridge，
   `chooseProjectDirectory` 返回 null 根本到不了这行模板；vitest 只测纯逻辑不挂组件。
2. **预览栏窄（c 类）**：`.content` 是为工具箱设计的两列网格
   `minmax(300px,380px) 1fr`，`.code-browser` 漏了全宽视图惯例声明
   `grid-column: 1 / -1`（chat-view / workspace-view / people-chat-view 都有），
   整个视图被塞进第一列。
3. **错误全部静默**：App.vue 的操作反馈 toast 只渲染在
   `v-if="activeView === 'toolbox'"` 模板内，代码浏览器（及其他视图）里
   `notify()` 的任何报错用户都看不见——放大了缺陷 1 的「无声无息」。

## 修复

| 文件 | 改动 |
| --- | --- |
| `src/components/CodeBrowserView.vue` | `treeCache` 由 `Map` 改为普通对象 `Record<string, CodeTreeEntry[]>`（`has/set/delete/get` → 下标读写/`delete`/`[] ?? []`），v-for 遍历对象才是正确的 (value, key) 语义；源头加注释说明这个坑 |
| `src/style.css` | `.code-browser` 补 `grid-column: 1 / -1`（对齐 chat-view 等既有惯例）；`.cb-workspace` 补 `height:100% + align-self:stretch` 兜底撑高 |
| `src/App.vue` | 报错 toast 移出 toolbox-only 模板，全视图可见 |

## 影响面

- 纯前端三文件；Rust core / IPC / bridge 零改动。
- `loadTree` 语义不变（缓存命中判断 `undefined`）；`[]`（空目录）为 truthy，缓存判断不受影响。
