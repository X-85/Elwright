# 内置注册表精简为纯示例

## 变更（2026-08-22，用户决策）

内置 `capabilities.json` 从 24 条种子清单精简为**每类一个真实小示例**（共 3 条），旧条目与其资源文件全部移除。个人能力不再进仓库，统一走用户叠加层 `~/.elwright/`（导入按钮 / `ew import`）。

## 新示例（不复古有内容）

| id | 类型 | 内容 |
|---|---|---|
| `text-stats` | script | `resources/tools/text-stats/text_stats.py` 文本统计（行/字符/中文/英文单词，纯 stdlib，中文报错） |
| `capability-types` | knowledge | `resources/docs/capability-types.md` 能力三分法速览（script/knowledge/skill 怎么选） |
| `weekly-report` | skill | `resources/docs/weekly-report-sop.md` 周报生成（prompt + 离线 SOP 四桶分类法） |

## 移除内容

- 21 条旧注册表条目（13 script / 5 knowledge / 6 skill 种子清单）。
- 10 个资源文件：doc-keyword-search / xlsx-to-md / docx-to-md 三个脚本 + 6 个旧 SOP + company-home-collab.md。保留 `AI_CODE_AGENT_*.md`（AGENTS.md 引用）与 `resources/docs/AI_CODE_AGENT_MAINTENANCE.md` 维护方案本身。
- 连带作废：ROADMAP「公司机原版 10 脚本 + 4 知识文档导入」计划；api-doc-formatter 归类决策（条目已删）。

## 同步修改

- CI 冒烟（ci.yml）：`doc-keyword-search` → `text-stats`；`tech-grill` → `weekly-report`（降级 SOP 与 mock LLM 两处）。
- AGENTS.md 目录段：注册表描述、resources 说明更新。
- ROADMAP：V1 条目作废标记 + 已完成里程碑补记。

## 验证（见 verification.md）

ew ls（3 项）/ view / run（正常 + 边界报错）/ invoke 降级 + mock LLM 成功路径、cargo test 22 绿。
