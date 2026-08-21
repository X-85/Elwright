# 验证记录（2026-08-21，家里 macOS）

## 验证方式

用阶段 2 构建的既有二进制 `src-tauri/target/debug/ew`（未重新编译，避免与并行进行的阶段 3b 抢 target 目录；SOP 读取是运行时行为，与二进制新旧无关）逐个调用：

## 结果

| 能力 | 命令 | 结果 |
|---|---|---|
| prompt-optimizer | `ew invoke prompt-optimizer` | ✅ 降级展示「提示词优化 · 离线 SOP」全文 |
| knowledge-sharing | `ew invoke knowledge-sharing` | ✅ 降级展示「知识总结分享 · 离线 SOP」全文 |
| work-summary | `ew invoke work-summary` | ✅ 降级展示「工作总结 · 离线 SOP」全文 |
| api-doc-formatter | `ew invoke api-doc-formatter` | ✅ 降级展示「接口文档格式化 · 离线 SOP」全文 |
| skill-selpoint | `ew invoke skill-selpoint` | ✅ 降级展示「需求评审选点 · 离线 SOP」全文 |

## 结论

6/6 技能型能力（含阶段 2 的 tech-grill）离线降级均可读到真实 SOP，降级链路完整。纯资源新增，无代码风险。
