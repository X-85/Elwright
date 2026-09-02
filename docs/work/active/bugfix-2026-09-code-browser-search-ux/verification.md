# verification：代码浏览器搜索体验四子项（B4）

## 修复内容

- 后端 `search()`：200 条上限后彻底停止遍历（Cell 标志，filename 模式
  同样推进）；snippet 按字符截断（中文长行 panic 根除）。
- 前端：300ms 防抖即时搜索 + Enter 立即 + 模式切换触发；searchSeq 竞态
  防护；「清除搜索」按钮一键清空；结果文件名优先、路径弱化展示。

## 验证记录

- 【自动化】独立探针（GNU，2026-09-02）：UTF-8 边界截断（中/英）、
  capped 推进断言 ALL PASS；旧实现中文截断 panic 已用探针复现
  （`end byte index 200 is not a char boundary`）。
- 【自动化】新增单测 `search_snippet_truncates_on_char_boundary`、
  `search_filename_mode_respects_result_cap`（250 文件项目断言恰好
  200 条）；`cargo test --lib` 执行留 CI（本机 GNU 限制同 B3/B5）。
- 【自动化】e2e `search-ux.spec.ts` 4/4：
  ①输入停顿自动触发（mock IPC 计数 ≥1，无需回车）；
  ②结果文件名为主 + `.cb-hit-path` 次要路径；
  ③清除按钮一键清空、再输可搜（子项④回归）；
  ④3s 慢响应 + 清除 → 过期响应不回灌（竞态防护）。
- 【自动化】全套 e2e 19/19（15+4，无回归）；vitest 86/86；lint 0；
  vite build 成功。
- 【自动化】Rust 门禁：fmt clean；clippy --lib / --tests 各 0 警告
  （中途 clippy-driver STATUS_STACK_BUFFER_OVERRUN 为本机偶发，重跑通过）；
  ew 构建成功。
- 【自动化】截图：`screenshots/search-result-filename-first.png`。
- 【手测】（留给用户，装新 msi 后）①大项目输入即出结果；②结果首列
  文件名；③内容搜索不再长时间卡顿（200 条封顶后立即返回）；④中文
  长行命中不再令按钮置灰，清除按钮随时可清。
