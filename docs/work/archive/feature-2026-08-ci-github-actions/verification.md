# 验证记录（2026-08-21，家里 macOS）

## 静态验证

- YAML 语法解析通过（ruby YAML.load_file；系统 python 无 yaml 模块）。
- 逐条复核 workflow 命令与 main（`f21ede7`）状态一致：cargo test（阶段 2 通过 3 例）、npm ci/build（阶段 3 通过，lock 已提交）、`ew` 三条冒烟。

## 冒烟断言本地复现（与 CI 完全相同的命令与 grep，用既有 target/debug/ew 二进制）

| CI 断言 | 结果 |
|---|---|
| `ew ls | grep doc-keyword-search` | ✅ |
| `ew ls | tail -1 | grep 项能力` | ✅（未硬编码数量，防注册表增项后误红） |
| `ew invoke tech-grill | grep 八层拷问 · 离线 SOP` | ✅ |
| mock LLM（127.0.0.1:18434）`ew invoke tech-grill CI | grep 模拟LLM回复` | ✅ |

本地未重跑 cargo/npm：避免与并行进行的阶段 3b（Codex）争用 `src-tauri/target/` 与 `src/node_modules/`；对应命令在本会话对同一代码内容均已验证过。

## 未验证项（如实记录）

- GitHub Actions 真实运行未发生——workflow 随本次提交推送后才首跑；三平台 runner 环境（尤其 Windows bash + `.exe` 后缀处理）待首跑确认。**下次 push 后应查看 Actions 页并回填结果。**
- 平台特有风险已做防御：冒烟步骤显式 `shell: bash`，Windows 用 `format()` 补 `.exe`；ubuntu 的 TLS 依赖（reqwest 0.13 默认 rustls 系 + runner 预装 libssl-dev）双重覆盖。

## 结论

可合并。workflow 为纯新增（`.github/` 与任务目录），与阶段 3b 在制改动零交集。

---

## Actions 真实运行回填（2026-08-21）

| Run | 提交 | 结果 | 说明 |
|---|---|---|---|
| #1 `add2287` | docs 集成 | ❌ 启动失败（0 job） | 教训：`runner` 上下文写进了 job 级 env，GitHub 校验直接拒绝。修复 `169b9f1`：EW 挪到 step 级。 |
| #2 `169b9f1` | CI 修复 | ❌ 2/4 job | Frontend/Windows 绿；ubuntu `cargo test` exit 101（Tauri Linux 系统库缺失）、macos 冒烟挂（ew Broken pipe panic，产品 bug）。 |
| #3 `d53b50c` | 双修复 | ✅ **4/4 全绿** | ubuntu 加 apt 装 webkit2gtk 等；ew 输出容错宏（另有 bugfix 任务目录）。 |

结论：CI 门禁生效。三平台 cargo test/build + ew 冒烟（含 mock LLM 回归）+ 前端构建全部通过。
