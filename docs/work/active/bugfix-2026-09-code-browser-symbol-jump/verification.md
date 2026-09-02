# verification：代码浏览器符号跳转无响应（B5）

## 修复内容

- `walk_files` 遍历深度与目录树深度解耦：新增 `MAX_WALK_DEPTH = 24`
  供 search / scan_symbols 使用，9 层以上深包不再静默漏扫。
- 跳转输入框扫描期间显示进行中文案（placeholder + 候选区 loading 行）。
- 新增单测 `scan_symbols_covers_deep_packages`（10 层深包回归）。

## 验证记录

- 【自动化】独立探针（GNU 工具链，2026-09-02）：深度边界 0..=30，
  depth ≤ 24 扫描 / > 24 跳过；最深典型 Java 包路径 depth 10 判定
  SCAN。结果 ALL PASS。
- 【自动化】`scan_symbols_covers_deep_packages`：随代码提交；本机
  `cargo test` 受 B3 同款 GNU 限制无法执行，**待 CI 跑**。
- 【自动化】`cargo fmt --check`：FMT-CLEAN。
- 【自动化】`cargo clippy --lib --tests -- -D warnings`：0 警告。
- 【自动化】`cargo build --bin ew`（GNU）：成功。
- 【自动化】前端：eslint 0；vitest 15 文件 86 测试全过；vite build 成功。
- 【手测】（留给用户，装新 msi 后）真实 Java 项目 → 跳转输入接口名 →
  ①扫描期间有「正在扫描项目符号…」提示；②候选同时出现接口与实现类；
  ③点击候选能打开文件并定位到声明行；④深层包（≥9 层）里的类也能搜到。
