# Verification

纯文档任务，验证方式为文档与代码逐条对照（2026-08-22）：

1. **executor 解释器映射**：对照 `src-tauri/src/core/executor.rs` `build_command`——`.py`（python3→python→py 探测 + OnceLock 缓存，全失败回落 python3）、`.ps1`（-NoProfile -ExecutionPolicy Bypass -File）、`.sh`（bash）、`.bat/.cmd`（cmd /C）、其他扩展名中文报错「不支持的脚本类型」——behavior.md/architecture.md 记录一致。
2. **CLI run 校验顺序**：对照 `src-tauri/src/bin/ew.rs:99-130`——能力存在 → type=script → entry 存在 → 文件存在，任一不满足中文报错退出 1——behavior.md 一致。
3. **两条调用路径**：CLI `run_script`（继承 stdio）与桌面 `run_script_capture`（stdout+stderr 合并）——与 executor.rs 双函数一致。
4. **text-stats 行为**：读 `resources/tools/text-stats/text_stats.py`——行数/字符数/中文字符（CJK 区间）/英文单词统计、仅 UTF-8、中文报错——README 清单描述一致。
5. **CI 冒烟口径**：核对 `.github/workflows/ci.yml`——冒烟为 `ew ls | grep text-stats`（不实际执行脚本），architecture.md 初稿误写为「ew run text-stats」，已修正。
6. **无代码改动**：`git status` 仅含文档变更（本任务目录 + docs/features/script-tools/ + docs/ROADMAP.md + AGENTS.md 当前进度行）。
