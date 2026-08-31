# verification：窗口布局按钮对齐 macOS 原生

日期：2026-08-31 · 对应 plan.md checklist，全部勾选。

## 【自动化】

- eslint exit 0 / vitest 60/60 / vite build 成功。

## 【手测】真机（tauri dev macOS + ZCode 桌面控制）

- 绿点点击 → 全屏（窗口 0,34,1710,1073 全屏）；再点 → 退出全屏，
  窗口精确恢复 71,77,1200,780（用户原始位置）。
- 绿点 title/aria = 「全屏（悬停显示移动与调整大小）」/「全屏」。
- 四角/半屏/填充按钮共用 applyWindowLayout，半屏管道已被既有版本验证；
  四角为纯几何换算（x/y/halfWidth/halfHeight 组合），逐项点验挂 PENDING 真机清单。

## 已知未覆盖

- 全屏态下悬停菜单项（半屏/四角）行为未点验——原生在全屏态禁用平铺项，
  Elwright 菜单在全屏态仍可点，点选会退出全屏并平铺（可接受，后续按需收敛）。
