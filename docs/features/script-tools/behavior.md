# 脚本型 run —— 业务行为

## 触发条件

- 用户执行 `ew run <id> [args...]`（CLI）或点击桌面壳「运行」按钮（IPC `run_capability`，同一 executor）。
- `id` 必须在注册表中且 `type: "script"`；其他类型（knowledge/skill）调用 run 直接中文报错退出（退出码 1）。
- CLI 侧先校验：能力存在 → 类型为 script → `entry` 字段存在且文件存在，任一不满足中文报错退出。

## 执行规则

- 解释器按 entry 文件扩展名选择：
  - `.py` → `python3`，本机无 python3 时依次探测 `python` / `py`（Windows 官方启动器），探测结果进程内缓存一次；
  - `.ps1` → `powershell -NoProfile -ExecutionPolicy Bypass -File`；
  - `.sh` → `bash`；
  - `.bat` / `.cmd` → `cmd /C`；
  - 其他扩展名：报「不支持的脚本类型: .xxx（仅支持 py/ps1/sh/bat/cmd）」。
- 用户传入的 `args` 原样追加到脚本命令行，不做转义或过滤。
- CLI：脚本继承当前终端 stdio（流式直出），结束后打印「退出码: N」。
- 桌面：捕获 stdout 与 stderr 合并展示，退出码随结果返回前端。

## 边界与不允许的情况

- 无超时控制、无参数校验——脚本自身负责参数校验并输出中文报错。
- 脚本不存在（注册表有 entry 但文件缺失）：CLI 中文报错退出。
- 解释器启动失败（如未装 Python）：报「启动 xxx 失败」。
- 不依赖 LLM、不发起网络请求（脚本自身行为除外）。

## 对外部系统的影响

- 仅在本地 spawn 一次子进程执行脚本；无注册表回写、无配置变更。
