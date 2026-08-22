# Elwright Logo：方案二细化

## 目标

优化“上升动势 E”，让 E 与 AI 能量符号成为一个整体，而不是两个并排的图形。

## 设计约束

- 不使用盾牌、边框或防护型外轮廓。
- 保持黑白几何草图，先判断结构再上色。
- 闪电必须嵌入 E 的笔画或负空间。
- 只保留一个核心符号，不添加节点和装饰。

## 概念候选

- `concept-rising-e-v2a.svg`：闪电直接切入 E 的中部结构。
- `concept-rising-e-v2b.svg`：用 E 内部负空间形成闪电。
- `concept-rising-e-v2c.svg`：用 E 的右侧笔画与闪电合并成单一符号。

## 备选方案记录

- `concept-human-ai-tool.svg` 记录为备选方案：上半部分代表人，中间横线代表 AI，下半部分 L 代表工具。
- 当前不替换正式应用图标，待品牌方向确定后再制作彩色、单色和平台尺寸版本。

## 第三轮概念

- `concept-open-diamond.svg`：被打开的菱形，表达能力入口和向上空间。
- `concept-collaborative-paths.svg`：两条路径协同汇聚，表达人与 AI 的配合。
- `concept-modular-units.svg`：模块化工作单元，表达工具、知识和能力的组合。

## 第四轮定稿（参考图驱动）

用户提供手绘参考图（纯黑白粗笔、大量留白）：上横杠（右端重）+ 下横杠（左端重）+ 中间向右粗折角 + 右侧小元素。

定稿 `concept-e-forge-forward.svg`，语义映射到产品理念：

- **E 骨架**（上横杠 / 左立柱 / 下横杠）= 离线核心地基——没有 LLM 也完整可用的部分
- **中横替换为带缺口的向右折角** = 注入的前进动力——LLM 是增强不是地基
- **右侧四角星** = El（超人 Kryptonian 星/光）——品牌名的出处

正式产出（`assets/branding/`）：

- `elwright-mark.svg` — 单色主版本（白底石墨）
- `elwright-mark-dark.svg` — 深底反白版本
- `elwright-mark-color.svg` — 彩色版本（石墨结构 + 青色渐变能量）
- `app-icon.svg` → 渲染 `src-tauri/app-icon.png`（暖纸底）→ `tauri icon` 全平台尺寸

应用图标配色：暖纸底 `#f2efe8`、石墨 `#1e2833`、能量青渐变 `#2fb8a6 → #0e7f8c`。

## 第五轮定稿（单字母 E，学 ZCode 手法）

用户反馈 E+`>` 复合方案仍不够干净，提出参考 ZCode 图标：ZCode 就是一个 Z，Elwright 主要用一个 E。

分析 ZCode 图标（`/Applications/ZCode.app/Contents/Resources/icon.png`，1024×1024）：

- 纯黑满铺方底（`#000` 88% 占比），系统显示时套 squircle 圆角
- 白色字母作**负空间**挖出（黑底白字）
- 粗几何无衬线，单笔画宽度
- 字母占画面约 48% 宽 × 42% 高，居中

定稿 `concept-e-forge-forward.svg` 与正式三版本，完全照 ZCode 手法做 Elwright 的 E：

- 黑底 `#0d0d0d` + 白 E `#f5f5f5`（负空间）
- E 四笔等粗（~10.4% 画面高），中横略短于上下杠（经典 E 比例）
- 占画面 48%×42%，居中
- 彩色版：黑底白 E + 中横青渐变（`#2fb8a6→#0e7f8c`）= LLM 增强注入中横

正式产出（`assets/branding/`）：

- `elwright-mark.svg` — 黑底白 E，主版本（ZCode 式）
- `elwright-mark-light.svg` — 白底黑 E，浅色背景用
- `elwright-mark-color.svg` — 黑底白 E + 青色中横
