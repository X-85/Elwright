# Elwright 发版 SOP

> 本文档是发版的标准流程。每次发版照着走即可。
> 适用对象：项目维护者本人 + 协作者。
> 当前 release pipeline：`.github/workflows/release.yml`，由 `v*` tag 触发。

## 概述

发版流程由 5 步组成：

1. **同步三处版本号**（Cargo.toml / tauri.conf.json / package.json）
2. **本地预检**（`cargo test` + `cargo build --bin elwright` + `npm run build`）—— 避雷
3. **commit + 打 annotated tag + push**（同时触发 ci.yml + release.yml）
4. **监控 GitHub Actions**（ci.yml 必须绿，release.yml 必须产 dmg + msi）
5. **校验 GitHub Release**（assets 齐全、版本号正确、下载链接可用）

---

## 详细步骤

### 1. 同步三处版本号

版本号必须**三处一致**，否则打包脚本/产物文件名会乱套。

```bash
# 1.1 改 src-tauri/Cargo.toml
#   version = "0.1.1"  →  version = "0.2.0"
# 1.2 改 src-tauri/tauri.conf.json
#   "version": "0.1.1"  →  "version": "0.2.0"
# 1.3 改 src/package.json
#   "version": "0.1.1"  →  "version": "0.2.0"
```

**版本号约定**（语义化版本 semver）：

- **MAJOR.MINOR.PATCH**（如 `0.1.1`）
- 当前还在 `0.x.x` 阶段，**MAJOR 永远是 0**（正式 1.0 之前）
- 改 MINOR：新功能（minor release）
- 改 PATCH：仅 bug 修复
- 当前 `0.1.x` 是早期迭代，每次新功能都升 MINOR

**判断升 MINOR 还是 PATCH**：

| 累积改动 | 升 PATCH | 升 MINOR |
|---|---|---|
| 文档归档、changelog 修订、CI 调整 | ✅ | |
| Bug 修复（无新功能） | ✅ | |
| 性能优化（无新功能） | ✅ | |
| 新能力、新 UI 模块、新 IPC 命令 | | ✅ |
| 行为不兼容的改动 | | ✅（少数情况可跳 0.x → 1.0） |

**验证三处一致**：

```bash
grep -E '^version|"version"' src-tauri/Cargo.toml src-tauri/tauri.conf.json src/package.json
```

应输出三行版本号全部相同。

### 2. 本地预检（避雷）

> 这一步是**可选但强烈推荐**。本地跑一遍能挡掉 80% 的 CI 失败。

```bash
# 2.1 Rust 单元测试 + 构建
cd src-tauri
cargo test --lib                      # 单元测试（terminal / registry / llm / invoke 等）
cargo build --bin ew                  # CLI 二进制
cargo build --bin elwright            # 桌面应用二进制（首次编译较慢，~5-10 分钟）

# 2.2 前端构建
cd ../src
npm ci                                # 锁定依赖（首次或 package-lock.json 变了才需要）
npm run build                         # Vite 生产构建
```

**期望结果**：

- `cargo test --lib` 全绿
- `cargo build --bin ew` 成功
- `cargo build --bin elwright` 成功（如果新增了 Rust 代码，必须这个也通过）
- `npm run build` 成功（dist/ 产物生成）

**失败处理**：

- 修复代码 → 重新 commit → 重新跑
- **不要跳过这步直接 push**——CI 跑 10-15 分钟发现失败更浪费

### 3. Commit + 打 tag + push

```bash
# 3.1 commit 版本号 bump（仅这三处，不要把其他未提交内容一起 commit）
cd /path/to/Elwright
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json src/package.json
git commit -m "chore(release): bump version to X.Y.Z

三处版本号同步：src-tauri/Cargo.toml / tauri.conf.json / src/package.json"

# 3.2 打 annotated tag（带 release notes）
git tag -a vX.Y.Z -m "vX.Y.Z: <一句话总结这次发版>

新增：
- <新功能 1>
- <新功能 2>

更新：
- <改进 1>
- <修复 1>

完整 changelog 见 CHANGELOG.md / docs/work/archive/ 各 feature 的 verification.md"

# 3.3 push（同时触发 ci.yml + release.yml）
git push origin main
git push origin vX.Y.Z
```

**注意**：

- tag 名**必须**以 `v` 开头（`v0.1.2`），匹配 `release.yml` 的 `tags: ['v*']` 触发规则
- 用 **annotated tag**（`-a`）而不是 lightweight tag——annotated tag 包含 tagger / date / message，GitHub Release 用它生成 release notes
- tag 消息**至少写三行**：`vX.Y.Z: 一句话总结` + `新增：` + `更新：`

### 4. 监控 GitHub Actions

push 之后两个 workflow 同时触发：

| Workflow | 触发条件 | 跑什么 | 预期时长 |
|---|---|---|---|
| `.github/workflows/ci.yml` | push 到 main / PR / tag | 三平台 `cargo test/build --bin ew` + `npm run build` + `ew` 冒烟 | 10-15 分钟 |
| `.github/workflows/release.yml` | push tag `v*` | macOS dmg + Windows msi + 发 GitHub Release | 20-40 分钟 |

**打开 Actions 页面监控**：

```
https://github.com/X-85/Elwright/actions
```

**判断标准**：

- ci.yml：必须全绿（三平台 cargo + frontend build + smoke 都过）
- release.yml：必须成功产出 dmg + msi 两个 asset

**已知坑**（AGENTS.md 已记录）：

- dmg 第一次打包可能因 Finder AppleScript 超时（AppleScript 错误码 -1712）失败 → release.yml 内置重试，再跑一次即过
- 产物**未签名**——macOS 用户首次打开会被 Gatekeeper 拦，提示「右键 → 打开」才能过；这是预期行为

**如果失败**：

1. 看 Actions 页面里的报错日志（点 job → 点 step）
2. 常见原因：
   - 版本号没同步（dmg/msi 文件名会缺版本号）
   - Rust 代码编不过（`cargo build --bin elwright` 本地没跑）
   - 前端 `npm ci` lockfile 冲突
3. 修好后**重新打 tag**（不要 force-push tag）：

```bash
# 删本地 + 远端 tag
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z

# 修改代码 + commit
git add .
git commit -m "fix(release): <说明>"

# 重新打 tag
git tag -a vX.Y.Z -m "vX.Y.Z: <说明>"
git push origin main
git push origin vX.Y.Z
```

### 5. 校验 GitHub Release

release workflow 跑完后，**自动创建** GitHub Release。

**打开**：

```
https://github.com/X-85/Elwright/releases/tag/vX.Y.Z
```

**检查项**：

- [ ] Tag 名字正确（`vX.Y.Z`）
- [ ] Title 正确（`vX.Y.Z: <一句话总结>`）
- [ ] Release notes 包含 tag message 内容
- [ ] **Assets 至少 2 个**：
  - `Elwright_X.Y.Z_aarch64.dmg`（macOS Apple Silicon）
  - `Elwright_X.Y.Z_x64_en-US.msi`（Windows x64）
- [ ] 资产**文件大小合理**（dmg 通常 5-15 MB，msi 通常 8-20 MB；如果明显偏小说明打包失败）

**下载验证**（可选但推荐）：

- macOS：下载 dmg → 双击挂载 → 拖入 Applications → 启动 → 验证集成终端 v1 等新功能
- Windows：下载 msi → 双击安装 → 启动 → 同样验证

**更新 ROADMAP.md**（AGENTS.md 规定）：

在「当前版本」行更新到新版本：

```markdown
## 当前版本

vX.Y.Z（YYYY-MM-DD，tag `vX.Y.Z`，GitHub Release 附 dmg + msi。自 vX.Y.Z-1 新增：...）
```

---

## 紧急回滚

发版后如果发现严重问题：

```bash
# 1. 删除有问题的 GitHub Release（页面 → Edit → Delete）
# 2. 删除 tag
git push origin :refs/tags/vX.Y.Z
git tag -d vX.Y.Z
# 3. main 仍然保留（有问题的代码 commit 还在）
#    可以用 git revert 单独 revert 那次版本号 bump
git revert <bump-commit-sha>
git push origin main
# 4. 打 hotfix 版本（PATCH bump）
# 5. 修代码 → 走正常发版流程
```

> 不要 force-push main。永远用 revert + 新 commit 保持历史可追溯。

---

## 常见问答

**Q：tag 要不要 push 到 main 之后？**

A：都可以。push 顺序不影响——两个 workflow 独立触发。但推荐先 `git push origin main` 让 CI 先跑（CI 失败的话不发版，省得 dmg 打了再撤）；CI 绿了再 `git push origin vX.Y.Z`。

**Q：能不能本地打 dmg / msi？**

A：能但**不推荐**。本地 macOS 只能打 dmg，Windows 只能打 msi；GitHub Actions 两个平台并行打，节省时间。脚本：

```bash
# macOS dmg（要 macOS + Xcode CLT）
cd src-tauri
../src/node_modules/.bin/tauri build --bundles dmg
# 产物在 src-tauri/target/release/bundle/dmg/

# Windows msi（要 Windows + WiX Toolset）
# 在 Windows 上跑同上命令
```

**Q：版本号回退怎么办？**

A：不允许。版本号只能向前。如果发现新版本有 bug，要么 revert 代码后**重新打 PATCH 版本**，要么直接修代码后**重新打相同版本号**（删 tag + 重建）。

**Q：tag 写错了能改吗？**

A：tag 没 push 之前可以（`git tag -d vX.Y.Z` + 重建）。tag 一旦 push 且 release workflow 触发，就**不能改**——必须删 tag + 重新打（参考步骤 4 失败处理）。

**Q：release.yml 跑了一半卡住/超时怎么办？**

A：可以 cancel 后重新 push tag：

```bash
# 在 Actions 页面点 "Cancel workflow"
# 等几秒确认 status=cancelled
# 然后再次 push（不会自动重跑，需要先删 tag 重建）
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
git tag -a vX.Y.Z -m "vX.Y.Z: <说明>"
git push origin vX.Y.Z
```

---

## 相关文件

- `.github/workflows/ci.yml` —— CI 跑测试 + 构建
- `.github/workflows/release.yml` —— Release 打 dmg + msi
- `docs/ROADMAP.md` —— 当前版本号、已发版里程碑
- `docs/release/llm-setup-guide.md` —— 用户向文档（不是发版流程，但同目录）
- `Elwright架构方案.md` §11 —— 立项时的发版规划（已过时，仅作历史参考）
- `AGENTS.md` —— Agent 工作协议（含「归档由人执行」「不擅自改用户未提交内容」等）
