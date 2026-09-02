# Verification — 设置中心 模型档案

> 与 `plan.md` / `checklist.md` 对应，实施完成后逐条打勾 + 写证据
> 状态：待实施（占位）

## 单元 / Mock runtime 验证

（实施时回填：列出单测文件 + 例数 + 通过情况）

- `core::llm` 新单测：
  - 例 1：profile 解析合法 / 例数：
  - 例 2：activeProfile 命中走 profile / 例数：
  - 例 3：activeProfile 未命中回退 flat / 例数：
  - 例 4：env 永远高优 / 例数：
  - 例 5：注册表默认兜底 / 例数：
- `bin/ew.rs` `ew config profile` 子命令：
  - （手工或脚本化验证；CI 中跑通的关键 case）
- mock runtime IPC 测试（`tests/llm_profiles_ipc.rs`）：
  - list / use / save / delete / 旧 flat 兼容 各一例
- vitest `profileSwitch.test.ts`：4+ 例

## 本地五道闸（实施完成后跑一次）

- [ ] `cargo test`（≥87 + 新增）全绿
- [ ] strict `cargo clippy --all-targets -- -D warnings` 0 警告
- [ ] `cargo fmt --check` 无差异
- [ ] `npm run test`（≥47 + 新增）全绿
- [ ] `npm run build` 成功

## CI 验证

- [ ] PR 推送后 CI 7/7（mac/win/linux clippy+fmt + 三平台 cargo + 前端 + dmg + msi）全绿

## 文档验证

- [ ] `docs/features/settings-center/behavior.md`：模型设置章节含"档案管理"段落
- [ ] `docs/features/settings-center/architecture.md`：架构图含 `profiles` / `activeProfile`；含"配置解析顺序"小节
- [ ] `docs/features/settings-center/changelog.md`：本期条目
- [ ] `docs/features/settings-center/README.md`：分类与阶段表更新
- [ ] `docs/ROADMAP.md` §V2「设置中心一期延伸」首条标记完成

## 端点验证（待真机点验）

- [ ] 建 2 个 profile，ChatView 头部模型名随之变化
- [ ] 切换 profile → 流式首字延迟、invoke skill 走新配置
- [ ] 旧 flat 配置（无 profiles 字段）继续生效
- [ ] key 脱敏（仅后四位展示）
- [ ] 删除 profile（当前激活时优雅提示"将回退 flat"）

## 实施期间追加

（实施中发现的新验证点直接追加到这里）