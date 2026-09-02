# 实施清单（Checklist）

## ADR 阶段

- [x] ADR-001：阶段④范围（暂不引入 Java LSP，受控补丁编辑边界）

## 后端 core/patch.rs

- [ ] `parse_unified_diff`：合法 diff 单测
- [ ] `parse_unified_diff`：缺路径 / 二进制 / 多文件边界单测
- [ ] `apply_hunks_to_content`：匹配 / 上下文漂移 / 空文件
- [ ] `is_sensitive_path`：.env / .ssh / node_modules / target 四例
- [ ] `PatchSnapshot` 持久化 round-trip

## IPC

- [ ] `apply_patch_preview` 命令
- [ ] `apply_patch_apply` 命令（含敏感路径拒收 + 指纹校验）
- [ ] `apply_patch_revert` 命令
- [ ] `main.rs` invoke_handler 注册三个命令
- [ ] mock runtime IPC 测试三例

## 前端

- [ ] `bridge.ts`：applyPatchPreview/Apply/Revert 三个方法 + 浏览器降级
- [ ] `lib/patch.ts`：diff 文本前端解析（识别入口用，权威在后端）
- [ ] `components/PatchPreviewDialog.vue`：三栏布局 + 接受/拒绝复选 + 写入按钮
- [ ] `ChatView.vue`：助手消息 ```diff ``` 块识别 → 渲染预览按钮
- [ ] `CodeBrowserView.vue`：「最近应用」列表入口
- [ ] vitest：patch.ts 解析 3 例 + diff 块识别 2 例

## 文档

- [ ] `docs/features/code-browser/behavior.md`：新增阶段④节
- [ ] `docs/features/code-browser/architecture.md`：新增 PatchPreviewDialog / apply_patch_* 组件与命令
- [ ] `docs/features/code-browser/changelog.md`：阶段④条目
- [ ] `docs/features/chat/behavior.md`/`architecture.md`/`changelog.md`：补阶段③④（发版后遗漏，本次随阶段④一并回填）
- [ ] ADR-001 验证节回填

## 验收

- [ ] 本地五道闸：cargo 全部 / 严格 clippy / fmt / vitest / build 全绿
- [ ] PR 开出、CI 7/7 全绿、合并 main
- [ ] 任务目录归档（active→archive，由用户确认上线后执行）
- [ ] 真机点验项加入 PENDING-REAL-MACHINE-CHECKLIST.md

## 不做（本阶段明确外）

- Java Language Server 集成
- 内置编辑器（多光标 / 撤销栈）
- 自动保存 / git 集成
- 三向合并
- 行级 hunk 的语法校验