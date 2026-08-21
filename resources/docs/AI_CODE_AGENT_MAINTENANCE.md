# 使用代码 Agent 进行代码开发与维护（方案）

> 本文件是规则说明。所有"长什么样"的成品示例见 [AI_CODE_AGENT_EXAMPLES.md](./AI_CODE_AGENT_EXAMPLES.md)。
> 修改规则时，必须同步检查示例文档是否需要更新。

## 1. 背景

使用 AI 代码 Agent 开发新功能时，通常先通过 Plan 模式规划再执行。一次完整开发往往能满足大部分需求，但后续还会出现：需求小范围变化、上线后 Bug 修复、一个月后追加需求、旧 Plan 不再代表当前代码、重读代码难以恢复上下文。根源是把一次性开发计划、当前功能说明、设计决策和历史变更混在了一起。

## 2. 目标

建立一套轻量、可持续的 Agent 开发与维护方案：

1. 区分一次性任务计划和长期功能文档。
2. 根据修改规模选择合适的记录方式。
3. 通过目录名识别新增功能、Bug 修复、中等需求和大改动。
4. 保留验证结果和历史变更，不让 `active` 无限膨胀。
5. 人和 Agent 在很久以后都能快速恢复代码上下文。
6. Feature 文档描述当前真实状态，不堆积历史 Plan。

## 3. 核心原则

| 原则 | 含义 |
|---|---|
| Plan 是一次性任务文档 | 只描述一项任务如何完成，完成后不追加新需求；新需求创建新任务和新 Plan |
| Feature Spec 描述当前状态 | 描述功能现在如何工作，业务规则或架构变化时更新，不记录代码演进历史 |
| 历史记录单独保存 | Git / PR / Issue / Change Note / Verification 记录具体任务，完成后移入 `archive`，不删除 |
| 当前事实优先于历史计划 | Agent 了解功能时优先读 Feature README → behavior → architecture → 当前代码和测试；归档文档只在调查历史时读 |

## 4. 目录结构

抽象骨架：

```text
project/
├── AGENTS.md                    # Agent 工作协议（见 §9）
├── docs/
│   ├── README.md
│   ├── features/<feature>/      # 长期功能文档（见 §5）
│   └── work/{active,archive}/   # 任务目录（见 §6、§7）
└── src/                         # 代码，按业务模块组织
```

`src/` 下的 README 讲"这个目录怎么用、怎么改"；`docs/features/` 讲"业务行为"。两者职责不同，不要互相复制。

→ 完整目录树示例见 EXAMPLES.md §1

## 5. Feature 文档的职责

| 文档 | 回答的问题 | 何时更新 |
|---|---|---|
| `README.md` | 功能是什么？是否上线？代码在哪？还要读什么？ | 功能入口信息变化时 |
| `behavior.md` | 当前业务规则是什么（触发 / 状态 / 结果 / 不允许 / 幂等并发重试 / 外部影响） | 业务行为变化时 |
| `architecture.md` | 当前技术结构、调用关系、事务 / 重试 / 幂等、系统边界、失败路径 | 技术结构变化时 |
| `changelog.md` | 功能有哪些重要变化 | 对外可见的业务、接口、数据或架构变化时 |
| `decisions/ADR-*.md` | 为什么选择这个方案 | 重要设计决策定案时；普通 Bug 修复不写 ADR |

"重要变化"判断启发式：

- 接口 / 数据 / 行为对外可见 → changelog
- 影响两周内难推翻的方案取舍 → ADR
- 普通 Bug 修复 → 两者都不用

→ 各文档成品示例见 EXAMPLES.md §2

## 6. `active` / `archive` 下的任务目录

目录名带类型前缀，有 Issue 编号时加入目录名（如 `enhancement-ORD-132-configurable-timeout`）：

| 前缀 | 用途 | 包含文件 |
|---|---|---|
| `feature-` | 第一次新增功能或新增独立能力 | plan, checklist, verification, (rollout) |
| `bugfix-` | Bug 修复、小范围修改 | change-note, verification |
| `enhancement-` | 中等业务增强或规则变化 | plan, checklist, behavior-change, verification, (rollout) |
| `refactor-` | 大型重构或核心技术方案替换 | plan, design, architecture-before/after, migration, verification, rollout |
| `migration-` | 数据或系统迁移 | 同 refactor |
| `hotfix-` | 紧急线上修复 | 同 bugfix |

各文件职责：

- `plan.md`：目标、范围、实现步骤、非目标、风险和验证方式
- `checklist.md`：实现进度和完成项
- `verification.md`：测试、静态检查、集成验证和结果
- `rollout.md`：上线顺序、监控、灰度和回滚；风险低时可省略
- `change-note.md`：问题、原因、修改范围、风险和影响
- `behavior-change.md`：修改前后的业务规则、兼容策略和边界情况（仅中等修改）
- `design.md`：详细技术方案和取舍（仅大修改）
- `architecture-before/after.md`：改造前后结构和调用流程（仅大修改）
- `migration.md`：数据迁移、双写、兼容和回滚（仅大修改）

→ 各类型任务目录树与成品示例见 EXAMPLES.md §3

## 7. 归档规则

任务代码和测试完成、Feature 文档更新到位后，**归档动作由人在确认上线后执行**，Agent 只负责更新文档和补全验证结果，不自行判断"已上线"。

归档前检查清单：

1. 代码和测试完成。
2. `behavior.md` 更新为当前真实行为。
3. `architecture.md`、`changelog.md` 或 ADR 按需更新。
4. `verification.md` 补全验证结果。
5. 将整个任务目录从 `active` 移到 `archive`。

`active` 只放进行中的任务，`archive` 放已完成任务及验证证据；归档文档不删除、不改写历史事实，当前行为以 Feature 文档为准。

→ 归档后的目录树示例见 EXAMPLES.md §4

> 建议：任务目录内加一个 `STATUS.md`（`in-progress / ready-for-release / archived`），避免 `active` 堆满"已上线但没人归档"的任务。

## 8. 不同规模修改的处理方式

| 规模 | 适合场景 | 处理方式 |
|---|---|---|
| 小修改 / Bug | 明显 Bug、边界判断、配置修正、局部实现调整 | 建 `bugfix-*`：读 Feature 文档和测试 → 总结当前行为 → 改代码和测试 → 写 verification → 按需更新 Feature 文档 → 归档 |
| 中等修改 | 业务规则改变、接口行为变化、影响多模块 | 建 `enhancement-*`，使用轻量或完整 Plan，不要修改旧 Plan |
| 大修改 | 数据模型 / 状态机变化、跨服务改动、核心架构替换、需迁移或灰度 | 建 `refactor-*` / `migration-*`，补充 design / migration / verification / rollout，必要时新增 ADR |
| 一个月后新增需求 | 同功能的后续需求 | 默认创建新任务，不继续追加旧 Plan；完成后更新同一个 Feature 的当前文档 |

## 9. Agent 工作协议

将修改前 / 修改时 / 修改后的工作规则写入项目根目录 `AGENTS.md`。全文见 EXAMPLES.md §5，提示词模板见 EXAMPLES.md §6。

## 10. 文档生命周期一览

| 文档或记录 | 生命周期 |
|---|---|
| Feature README / behavior / architecture / changelog / ADR | 长期维护 |
| 任务 plan / change-note / verification / rollout | 单个任务，完成后随目录归档保留 |
| Git / PR | 长期历史 |

这样避免每个小修改都重新写完整计划，同时保留足够的上下文，让人和 Agent 在几周或几个月后都能继续维护代码。
