# 一键安装 Elwright 桌面版

> 不去 GitHub Release 翻 dmg / msi，直接在终端用一条命令拉最新版本装上。

## macOS

```bash
curl -fsSL https://raw.githubusercontent.com/X-85/Elwright/main/install.sh | bash
```

会做：
1. 探测架构（Apple 芯片 / Intel）
2. 从 GitHub `releases/latest` API 拿最新 tag
3. 下载对应 dmg（`Elwright_<ver>_aarch64.dmg` 或 `Elwright_<ver>_x64.dmg`）
4. 挂载 dmg → 用 `ditto` 复制 `Elwright.app` 到 `/Applications`（保留 extended attributes）
5. 卸载 dmg、清理临时目录

**装完位置**：`/Applications/Elwright.app`

**首次打开**（未签名应用过 Gatekeeper 的标准流程）：
1. 在「应用程序」找到 Elwright
2. 右键 → 打开 → 弹出框里再点「打开」
3. 之后双击就能直接打开

**卸载**：`rm -rf /Applications/Elwright.app`

## Windows

PowerShell（普通权限就行，安装时会弹 UAC 提权）：

```powershell
irm https://raw.githubusercontent.com/X-85/Elwright/main/install.ps1 | iex
```

会做：
1. 按 ProductCode 查注册表，已装就跳过（幂等）
2. 没装就拉 latest tag，下载 `Elwright_<ver>_x64_en-US.msi`
3. `msiexec /i <msi> /quiet /norestart` 静默安装
4. **首次启动**：开始菜单搜「Elwright」打开；若被 SmartScreen 拦，点「更多信息」→「仍要运行」

**卸载**：控制面板 → 程序与功能 → 卸载 Elwright

## 可调参数

两个脚本都吃环境变量：

| 变量 | 默认 | 作用 |
|---|---|---|
| `ELWRIGHT_VERSION` | latest | 指定版本号（带不带 `v` 前缀都行） |
| `ELWRIGHT_REPO` | `X-85/Elwright` | 指向 fork 或私有仓库 |
| `ELWRIGHT_INSTALL_DIR`（仅 mac） | `/Applications` | 装到用户目录可免 sudo |

### 例子

```bash
# 装指定版本
curl -fsSL https://raw.githubusercontent.com/X-85/Elwright/main/install.sh | ELWRIGHT_VERSION=v0.1.3 bash
```

```bash
# mac 装到用户目录（不需 sudo）
curl -fsSL https://raw.githubusercontent.com/X-85/Elwright/main/install.sh | ELWRIGHT_INSTALL_DIR=~/Applications bash
```

```powershell
# Windows 装指定版本
$env:ELWRIGHT_VERSION='v0.1.3'; irm https://raw.githubusercontent.com/X-85/Elwright/main/install.ps1 | iex
```

## 故障排查

### mac：`✗ 挂载 dmg 失败`

通常是 Gatekeeper 给 dmg 本身打 quarantine 标签了，先 `xattr -dr com.apple.quarantine <dmg-path>` 再跑脚本，或直接双击 dmg 走 Finder 拖装。

### mac：`✗ 复制失败`

大概率是 `/Applications` 没写权限。两种处理：
- 装到用户目录：`ELWRIGHT_INSTALL_DIR=~/Applications bash install.sh`
- 走 sudo：`sudo bash install.sh`

### Windows：SmartScreen 红色拦截

脚本走的是 `msiexec /quiet`，正常情况下不会有 UI。**首次启动 .exe** 时 SmartScreen 可能拦——按上面"首次打开"流程点过就行。

### Windows：脚本说不支持

`install.ps1` 顶部硬编码了 **ProductCode `{653FB0F8-53C0-448C-99E7-AEBC37F313B5}`**（v0.1.3 msi 的产品 ID）。每发一版要把这个 ID 同步更新——查法：`curl -L -o x.msi <msi-url> && file x.msi`，看输出里 `Revision Number: {GUID}`。

## 跟「手动从 GitHub Release 下载」什么关系？

脚本就是「帮你在终端里做手动那几件事」——不做任何额外的事。**手动下载仍然有效**，跟脚本并存。脚本的优势：

- 不打开浏览器（远程 ssh 进机器也能装）
- 不需要双击 / 拖拽（自动化、CI、可复制粘贴）
- 默认装 latest，永远是最新版（手动容易下到老版本）

## 脚本本身在哪里？

- mac：`install.sh`（仓库根，executable bit 已设）
- Windows：`install.ps1`（仓库根）

两个脚本**走 GitHub raw 域名分发**——commit 到 main 后立刻生效，不需要再发版。

升级时同步改一下 `install.ps1` 的 `ProductCode` 即可（AGENTS.md 第 54 行附近有提醒）。
