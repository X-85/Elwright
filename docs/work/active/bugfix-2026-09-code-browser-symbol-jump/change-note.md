# change-note：代码浏览器符号跳转无响应（B5）

## 问题（真机点验 v0.1.13，Windows）

用户在真实 Java 项目中输入接口名，符号跳转候选为空、无任何响应；
同项目内容搜索也结果偏少。

## 根因

1. **主根因——目录遍历深度限制吞掉深层包**：`search()` 与
   `scan_symbols()` 共用的 `walk_files` 深度上限 `MAX_TREE_DEPTH = 8`。
   Java 项目路径 `src/main/java/com/example/...` 每层包名消耗一层：
   `src(1)/main(2)/java(3)/com(4)/example(5)/company(6)/product(7)/module(8)/service(9)`
   —— 第 9 层起 `depth > 8` 整棵子树**静默 return**，深层类的符号与内容
   都不在索引里。跳转输入框首次聚焦触发全项目扫描，扫到的全是浅层文件，
   输入接口名自然「无响应」。（惰性加载的目录树 UI 不受影响——`tree()`
   每层单独请求，不走 `walk_files`。）
2. **次因——扫描无任何进度反馈**：大项目首次符号扫描（上限 5000 文件）
   需数秒，期间输入框无占位提示、无 loading 文案，观感即「点了没反应」。

## 修改范围

- `src-tauri/src/core/code_browser.rs`
  - 新增 `MAX_WALK_DEPTH = 24`（`src/main/java` + 20 层包名，远超常见
    项目），`walk_files` 改用该常量；`MAX_TREE_DEPTH` 保留给语义上的
    目录树场景注释说明。二者此前混用同一常量是根因。
  - 新增单测 `scan_symbols_covers_deep_packages`：10 层深包里的类必须
    被索引到（旧代码下此测试必失败）。
- `src/components/CodeBrowserView.vue`
  - 跳转输入框：扫描进行中 placeholder 显示「正在扫描项目符号…（大项目
    首次需数秒）」，候选区同步显示 loading 行；完成后恢复原占位文案。

## 风险

- 低。深度提升只放大遍历范围，`MAX_SYMBOL_FILES`（5000）与
  `MAX_SEARCH_RESULTS`（200）两个结果上限仍兜底，不会失控；
  SKIP_DIRS（target/node_modules/.git/build 等）照常剪枝。
- 24 层极端目录下 `walk_files` 递归栈深线性增长，实际项目不可达。

## 验证

- 【自动化】独立探针（GNU）：深度 0..=30 扫描/跳过边界断言 ALL PASS；
  最深典型 Java 包路径（depth 10）判定 SCAN。
- 【自动化】单测 `scan_symbols_covers_deep_packages` 已随代码提交，
  `cargo test --lib code_browser` 执行待 CI（本机 GNU 限制同 B3）。
- 【自动化】`cargo fmt --check` / `clippy --lib --tests -D warnings` /
  `cargo build --bin ew` 全绿；前端 eslint / vitest 86 / vite build 全绿。
- 【手测】（留给用户，装新 msi 后）代码浏览器选真实 Java 项目 → 跳转
  输入框输入接口名（如 UserService）→ 预期扫描期间显示「正在扫描」、
  完成后候选同时出现接口与实现类，点击定位到行。
