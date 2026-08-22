# 验证记录（2026-08-22，家里 macOS）

## ew ls

```
text-stats                 script      yes       -      文本统计
capability-types           knowledge   yes       -      能力类型速览
weekly-report              skill       no        -      周报生成
共 3 项能力
```

## view（knowledge 型）

`ew view capability-types` 正确读出 `resources/docs/capability-types.md` 全文。

## run（script 型）

- `ew run text-stats README.md` → 行数 58 / 字符 1804 / 中文 648 / 英文单词 132，退出码 0。
- 边界：不存在的文件 → `错误：文件不存在或不是普通文件：…`，退出码 1（中文报错符合约定）。

## invoke（skill 型）

- 降级路径（无 LLM 环境，$meta.llmDefault 指向的 localhost:11434 不可达）→ `【LLM 调用失败】…已自动降级为离线 SOP`，正确展示 `# 周报生成 · 离线 SOP`。
- 成功路径（本地 mock OpenAI 兼容端点 127.0.0.1:18434）→ `[模拟LLM回复] system='你是周报整理助手。…'`，prompt 模板正确传递。

## cargo test

22 通过 / 0 失败（core 叠加层与既有测试不受影响）。

## CI 同步

ci.yml 冒烟三处 hardcode id 已改（text-stats / weekly-report×2），推送后六 job 待回填。

## 文件清理

移除后 `resources/` 仅剩 5 文件：两个 AI_CODE_AGENT 维护文档 + 三个新示例文件；无空目录残留；用户叠加层 `~/.elwright` 本就不存在（干净）。
