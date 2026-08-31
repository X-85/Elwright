# verification：代码浏览器树重复渲染修复 + 窗口布局原生对齐

日期：2026-08-31

## 【自动化】本地闸门

- `npm run lint`：exit 0（Grid2X2 未用 import 清理后无告警）
- `npm test`：10 文件 60/60 通过
- `npm run build`：成功（chunk 警告为存量）

## 【手测】真机点验（tauri dev macOS，ZCode 桌面控制实测）

- 打开 springbootDemoV1 → 根层 8 条正常（.mvn / src 目录在前 + 6 文件）。
- 展开 `src`：**main / test 各出现一次**（修复前两份）；顺序 src → main/test → 后续文件。
- 再展开 `main`：java / resources 嵌套正确、无重复（深度 3）；扁平化递归支持任意深度。
- 绿点按钮：title 变为「全屏（悬停显示移动与调整大小）」；点击进入全屏
  （窗口 bounds 0,34,1710,1073 = 整屏），再点击退出全屏，
  窗口精确恢复原位（71,77,1200,780）。
- 悬停菜单为既有行为（mouseenter），本轮未改动触发方式；四角按钮（左上/右上/左下/右下）
  走既有 applyWindowLayout 管道，与半屏同一实现路径，未逐一真机点（低风险）。

## 遗留

- 四角分屏 / 三列排列的逐项真机点验未做（与半屏共用管道）；挂 PENDING 真机清单语境。
