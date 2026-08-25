# Verification · 工作工具栏第一阶段

按 checklist 逐项记录结果（自动化项附命令与输出摘要，手测项贴用户确认）。

## Step 2 Rust core

- `cargo test workbench`：**5/5 绿**——Todo 往返（id 递增/completedAt 维护/camelCase 落盘）、空文本与超长拒绝、Note 往返与倒序、路径穿越与非法日期拒绝（含合法边界闰日/年末）、损坏文件降级空列表。

## Step 3 IPC 命令

- `cargo test --test workbench_ipc`：**4/4 绿**——mock runtime 真协议：Todo 增查勾删往返（camelCase 字段断言）、未知 id/空文本中文报错、Note 存取与日期列表、非法日期拒绝。
- 全量 `cargo test`：42（lib）+ 6（terminal_ipc）+ 4（workbench_ipc）= 52 绿，无回归。

## Step 4 前端

- `npm run build` 通过（bridge 类型扩展无错误）。
- `npm test`：**26/26 绿**（含新增 workbench-bridge 4 例：模拟存储语义与 Rust core 同口径）。

## Step 5 e2e

- `npm run test:e2e`：**6/6 绿**（含新增工作台场景：Todo 添加→勾选划线→计数→删除→空态、今日记录输入→「已保存」徽标→Markdown 预览渲染、日期翻页清空、预览模式提示）。
- 勘误记录：bridge.ts 生成时模板字符串被转义坏（\` 变 \`），build 报 Invalid Unicode escape sequence，已修复并重验。

## CI

- 分支 push 后待合并 main 首跑回填（ci.yml 仅 main/PR 触发，分支直推不跑）。

## 手测项

- 桌面壳持久化冒烟：（待用户执行后回填）
