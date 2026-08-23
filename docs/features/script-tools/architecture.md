# 脚本型 run —— 技术结构

## 调用链

```
CLI  ew run <id>      → bin/ew.rs（类型/entry/文件存在校验） → executor::run_script        （继承 stdio，流式）
桌面 IPC run_capability → main.rs → executor::run_script_capture（捕获 stdout+stderr 合并）
```

两壳共用 `core/executor.rs`；输出格式化（CLI 的「▶ 运行」/「退出码」文案、桌面的结果面板）属壳专属逻辑，留在各自壳内，不进 core。

## executor 内部

- `build_command`：扩展名（小写化）→ 解释器映射表；不支持的扩展名返回中文 Err。
- `python_interpreter`：按 `python3` → `python` → `py` 顺序探测（`--version` 静默验证），结果存 `OnceLock`，每进程只探测一次；全部失败时回落 `python3`（由后续启动报错兜底）。
- `run_script` / `run_script_capture`：前者给 CLI（`.status()` 继承 stdio），后者给桌面（`.output()` 捕获合并，stderr 追加在 stdout 之后）。

## 失败路径

- 解释器启动失败（spawn err）→ Err「启动 <entry> 失败: <原因>」。
- 脚本自身非零退出：退出码原样透传（CLI 打印退出码；桌面随 ScriptOutput 返回），不算执行器错误。
- 进程被信号杀死（无退出码）：按 -1 处理。

## 测试

- `executor.rs` 单测：解释器探测命中可用候选、不支持扩展名报错、捕获 stdout/stderr/非零退出码（unix）。
- CI 冒烟（`.github/workflows/ci.yml`）：三平台 `ew ls` 校验注册表可加载（含 `text-stats` 条目），不实际执行脚本。
