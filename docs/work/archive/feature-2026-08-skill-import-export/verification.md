# 验证记录（2026-08-22）

- 单测：A 根导出 → B 根导入完整还原（注册表条目 + SOP 文件）；id 冲突报错与 --force 覆盖；`..`/绝对路径均被拒（路径逃逸防护）。
- CLI 手动：`ew export tech-grill /tmp/x.elw.json`（schema elwright-skill/0.1，files 含 SOP）；导入空根后 `ew ls` 正确列出新能力；重复导入报错清晰。
- serde_json preserve_order：capabilities.json 既有键序保持。
