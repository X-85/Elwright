# 阶段 4：打包与发布（实施 plan，基于同目录 design.md）

## 本轮范围（家里 macOS 可完成）

1. **core 三段式资源根解析**（design §2）：`ELWRIGHT_ROOT` 环境变量覆盖 > cwd 上溯（现行为）> 调用方探测目录（Tauri 传 `Contents/Resources`，CLI 传 exe 上溯）。core 不依赖 tauri 类型，只收 `&[PathBuf]`。
2. **tauri.conf.json bundle.resources**（map 形式）：`../capabilities.json → capabilities.json`、`../resources/**/* → resources/`，进包后资源落在资源根下，与第 3 档探测对齐。
3. **两壳接入**：ew.rs 改用 `resolve_root(&[])`；main.rs setup 里经 `app.path().resource_dir()` 解析一次存入 `OnceLock`，IPC 命令读取。
4. **本机构建验证**：`cargo test`（新增解析单测）→ `tauri build` 出 dmg → 检查 .app 内资源落位（`Contents/Resources/capabilities.json` 与 `resources/` 存在）。
5. 文档回填 + 提交推送过 CI。

## 不在本轮（记录到 checklist）

- Windows msi（等公司机 MSVC，清单在 design §3）。
- GitHub Release 发布与版本号三处同步（发版时做，design §5）。
- 签名/公证（design §4：暂不做，README 写右键打开）。

## 风险

- Tauri resources map 形式对 `../` 外部路径的处理需实测（若产生 `_up_` 层级则调整映射目标）。
- release 全量编译约数分钟。
- 打包后 GUI 打开验证受限，以 bundle 结构 + 资源存在 + 二进制链接成功为准，GUI 交互留用户复验。

## 验证方式

- 单测：env 覆盖、cwd 上溯、探测目录三级各有覆盖。
- `tauri build` 成功且 dmg 存在；`Contents/Resources/` 下能看到 capabilities.json 与 resources/docs/*.md。
- CI 全绿（现有门禁）。
