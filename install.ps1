<#
install.ps1 — 从 GitHub Release 拉最新 Elwright Windows msi 静默安装。

用法（PowerShell）：

    irm https://raw.githubusercontent.com/X-85/Elwright/main/install.ps1 | iex

参数（可经环境变量覆盖）：

    $env:ELWRIGHT_VERSION = "v0.1.3"   # 指定版本，默认 latest
    $env:ELWRIGHT_REPO    = "X-85/Elwright"

设计原则：
  - 零依赖：只走 PowerShell 5.1+ 自带 cmdlet + Windows Installer
  - 幂等：先检测已装版本（按 ProductCode），命中就跳过 / 否则覆盖
  - 安静：默认 /quiet + /norestart（不弹 UI、不自动重启）
  - 不动 SmartScreen：未签名应用首次启动仍需「更多信息 → 仍要运行」
#>

$ErrorActionPreference = 'Stop'

$Repo        = $env:ELWRIGHT_REPO
if (-not $Repo) { $Repo = 'X-85/Elwright' }
$AppName     = 'Elwright'
$ProductCode = '{6635434E-978D-43C7-87D5-8C8507FA0543}'  # v0.1.6 的 MSI ProductCode，升级版本时需同步改
$WorkDir     = Join-Path $env:TEMP "elwright-install-$([guid]::NewGuid().ToString('N').Substring(0,8))"

# ---- 1. 探测已装版本（按 ProductCode 命中即视为已装）----
$existing = Get-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*',
                              'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*',
                              'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*' -ErrorAction SilentlyContinue |
            Where-Object { $_.DisplayName -eq $AppName } |
            Select-Object -First 1
if ($existing) {
    Write-Host "✓ $AppName 已安装（版本：$($existing.DisplayVersion)）；如需升级请手动卸载或调 `--force`（TODO）"
    exit 0
}

# ---- 2. 决定要装的版本（默认 latest）----
if ($env:ELWRIGHT_VERSION) {
    $tag = $env:ELWRIGHT_VERSION -replace '^v', ''
    $tag = "v$tag"
} else {
    Write-Host '→ 查询最新版本 ...'
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ Accept = 'application/vnd.github+json' }
    $tag = $release.tag_name
}
$version = $tag.TrimStart('v')
$msiName = "${AppName}_${version}_x64_en-US.msi"
$url     = "https://github.com/$Repo/releases/download/$tag/$msiName"
Write-Host "→ 准备安装 $AppName $tag（x64）"

# ---- 3. 下载 ----
New-Item -ItemType Directory -Path $WorkDir -Force | Out-Null
$msiPath = Join-Path $WorkDir $msiName
try {
    Write-Host "→ 下载 $url"
    Invoke-WebRequest -Uri $url -OutFile $msiPath -UseBasicParsing
} catch {
    Write-Host "✗ 下载失败：$_" -ForegroundColor Red
    exit 1
}

# ---- 4. 静默安装 ----
# /quiet  = 无 UI
# /norestart = 不自动重启（让用户自己决定）
# /l*v = 详细日志到文件，失败时方便排查
$logPath = Join-Path $WorkDir 'install.log'
Write-Host '→ 安装中（首次需要管理员权限，会弹 UAC）...'
$proc = Start-Process -FilePath 'msiexec.exe' `
    -ArgumentList "/i `"$msiPath`" /quiet /norestart /l*v `"$logPath`"" `
    -Verb RunAs -Wait -PassThru
if ($proc.ExitCode -ne 0) {
    Write-Host "✗ 安装失败（退出码 $($proc.ExitCode)），日志：$logPath" -ForegroundColor Red
    exit $proc.ExitCode
}

# ---- 5. 完成提示 ----
@"
✓ $AppName $tag 已装好

启动：在开始菜单搜 "$AppName"，或去「$env:ProgramFiles\$AppName」找 $AppName.exe
首次打开被 SmartScreen 拦截时：点「更多信息」→「仍要运行」（未签名应用过 SmartScreen 的标准流程）

卸载：控制面板 → 程序与功能 → 卸载 $AppName
"@ | Write-Host
