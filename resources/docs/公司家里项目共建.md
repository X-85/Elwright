# 公司 / 家里项目共建指南（Elwright）

> 适用场景：同一套代码，白天在公司机器开发、晚上回家继续开发，两边通过 GitHub 同步。
> 核心思路：**一个远程仓库当真相源，公司和家里都是它的克隆（working copy）。谁改完推上去，另一边拉下来。**

---

## 1. 整体结构

```
       GitHub (真相源)
        /          \
   公司克隆        家里克隆
  (走代理)        (正常网)
```

- 远程：`https://github.com/X-85/Elwright.git`（public）
- 公司机器本地路径：`D:\knowledge-base\toolbox\Elwright\`
- 家里机器：随便一个目录 `git clone` 即可

---

## 2. 公司机器 git 配置（关键）

公司网**无外网直连**，上 GitHub 必须走代理；但公司 GitLab 是内网，不能走代理。所以**代理只配给 github.com，不动全局**，避免把 GitLab 的推送也塞进代理。

在**公司机器上只需配一次**（已写入用户级 `.gitconfig`）：

```bash
# 仅 github.com 走代理（公司 GitLab 直连内网，不受影响）
git config --global http.https://github.com.proxy http://44895:你的域密码@10.182.0.99:9090
# 公司代理做 SSL 拦截（自签证书），github 走它时要关证书校验
git config --global http.https://github.com.sslVerify false
# 关键修复 502：git 默认用 NTLM/协商认证，公司代理只认 Basic
git config --global http.proxyAuthMethod basic
```

> ⚠️ 把域密码写进了用户级 `.gitconfig`（本机），个人机器可接受；嫌明文就改用环境变量或单独凭据助手。

### 凭据隔离（不会串台）

Windows 凭据管理器里，git 凭据**按主机名分开存**：

- `git:https://github.com` → GitHub PAT
- `git:https://yfgitlab.irayple.com` → 公司 GitLab 令牌

推 Elwright（`origin` = github.com）用 PAT；推公司项目（`origin` = GitLab）用 GitLab 令牌。git 按远程 URL 域名挑对应凭据，**绝不串台**。公司 GitLab 那套推送方式完全不用动。

### 远程地址（PAT 写进 URL，最稳）

Elwright 仓库的远程已配成：

```bash
git remote add origin "https://X-85:<你的PAT>@github.com/X-85/Elwright.git"
git branch -M main
git push -u origin main
```

PAT 只存在本机 `.git/config`，**不进公开仓库内容、不进聊天、不影响家里**（家里用自己的凭据 clone）。

---

## 3. 家里机器配置（一次）

家里正常网，直接 clone：

```bash
git clone https://github.com/X-85/Elwright.git
cd Elwright
# 之后正常 add / commit / push，GitHub 家里可直连
```

若家里也想用代理（一般不需要），按公司机器同理配，但把代理地址换成家里的。

---

## 4. 日常协作节奏（极简，符合 pi-mono 哲学）

solo 开发、两台机器，别搞复杂分支流。

**离开一台机器前：**

```bash
cd D:\knowledge-base\toolbox\Elwright   # 公司
# 或 cd <家里 clone 目录>
git add -A
git commit -m "做了啥"
git push
```

**到另一台机器先：**

```bash
git pull
```

### 两边都可能改的情况

如果同一天两边都会动同一批文件，直接在主分支上两边互踢会冲突。开临时分支：

```bash
git checkout -b wip-home     # 家里
# 或 git checkout -b wip-office  # 公司
git push -u origin wip-home
```

另一边拉下来合并后再推主分支。主分支就叫 `main`，别上 `develop`/`release` 那套（开源发布时再按需要拉 `release/*`，后话）。

---

## 5. 凭据安全说明

- **PAT 是给你自己机器上的 git 用的，不是给 AI/任何人用的**——不要贴进聊天框、不要发给别人。
- 生成位置：GitHub 网页右上角头像 → Settings → Developer settings → Personal access tokens → Tokens (classic) → Generate new token。
  - 仓库先私后公开 → 勾 `repo`；直接建 public → 只勾 `public_repo`（最小权限）。
  - Expiration 设 90 天，别选永久。
  - 生成后**只显示一次**，立刻复制存好。
- PAT 泄露/过期时，本地清理：`git remote set-url origin https://github.com/X-85/Elwright.git`（去掉 URL 里的 token）。

---

## 6. 踩坑记录（公司网实测）

| 现象 | 原因 | 解法 |
|------|------|------|
| `git push` 报 `502` | git 默认 NTLM/协商认证，公司代理不认 | `git config --global http.proxyAuthMethod basic` |
| `git ls-remote` 报 `SELF_SIGNED_CERT_IN_CHAIN` | 公司代理做 SSL 拦截（自签证书） | `http.https://github.com.sslVerify false` |
| GitHub API 建仓报 `Problems parsing JSON` | 中文 description 被 shell 编码弄坏 | 改用纯 ASCII description + 无 BOM UTF-8 临时 JSON 文件，`curl --data-binary @文件` |
| 直连 GitHub TCP 不通 | 公司 McAfee 网关拦截 | 走代理 `http://10.182.0.99:9090`（Basic 认证可用） |
| `git push` 报 502 / `Failure receiving data from peer` | 公司代理对较大上传间歇掉线（小请求如 `ls-remote` 正常） | **重试即可**：`for($i=1;$i-le 10;$i++){ git push; if($LASTEXITCODE-eq 0){break}; sleep 5 }`。首次 push 偶发失败是正常的，多试几次必过 |

---

## 7. 兜底方案（公司禁止往公网推代码时）

若公司政策禁止从办公机往公网仓库推代码，改用**桥接法**：

1. 公司只本地提交（`git commit`，不 push）；
2. 下班前 `git bundle create elwright.bundle --all` 生成单文件，丢进 OneDrive / U 盘；
3. 回家 `git clone elwright.bundle` → 推到 GitHub；
4. 之后家里是 GitHub 主推手，公司只拉不推（或继续用 bundle 单向同步）。

这样代码不违规出公司网，家里仍有完整历史。

> 另：`D:\knowledge-base` 整体笔记/工具可用 OneDrive 同步，但**代码仓库别塞进 OneDrive 实时同步**（构建产物 + 冲突风险）。Elwright 走 git，其余笔记走 OneDrive，各管各的。

---

## 8. 一页速查

```bash
# 公司：首次推送（已做过，备忘）
git init && git add -A && git commit -m "init"
git remote add origin "https://X-85:<PAT>@github.com/X-85/Elwright.git"
git branch -M main && git push -u origin main

# 日常
git add -A && git commit -m "..." && git push    # 离开前
git pull                                          # 到另一台先

# 家里首次
git clone https://github.com/X-85/Elwright.git
```

---
*最后更新：2026-08-21（公司机器已成功 push，远程 main 与本地首提交一致）*
