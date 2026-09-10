param(
    [string]$SkillRoot,
    [string]$SkillName = "topic-ledger"
)

$ErrorActionPreference = "Stop"
$source = Split-Path -Parent $MyInvocation.MyCommand.Path
$userHome = [Environment]::GetFolderPath("UserProfile")

if ([string]::IsNullOrWhiteSpace($SkillRoot)) {
    $candidates = @(
        (Join-Path $userHome ".zcode\skills"),
        (Join-Path $userHome ".codex\skills"),
        (Join-Path $userHome ".config\zcode\skills")
    )
    $SkillRoot = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
    if ([string]::IsNullOrWhiteSpace($SkillRoot)) {
        throw "找不到 ZCode/Codex skills 目录。请通过 -SkillRoot 指定，例如 $userHome\.zcode\skills。"
    }
}

New-Item -ItemType Directory -Force -Path $SkillRoot | Out-Null
$target = Join-Path $SkillRoot $SkillName
if (Test-Path $target) {
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $backup = "$target.backup-$stamp"
    Move-Item -Path $target -Destination $backup
    Write-Host "已备份旧 Skill：$backup"
}

New-Item -ItemType Directory -Force -Path $target | Out-Null
Get-ChildItem -Path $source -Force | Where-Object { $_.Name -notin @("install.ps1") } | ForEach-Object {
    Copy-Item -Path $_.FullName -Destination $target -Recurse -Force
}
if ($SkillName -eq "session-ledger") {
    $skillFile = Join-Path $target "SKILL.md"
    $skillText = Get-Content -Raw -Path $skillFile
    $skillText = $skillText -replace "(?m)^name: topic-ledger$", "name: session-ledger"
    [System.IO.File]::WriteAllText($skillFile, $skillText, (New-Object System.Text.UTF8Encoding($false)))
}
Write-Host "已安装 $SkillName 到 $target"
Write-Host "重新打开 ZCode 后使用 `$${SkillName}，或在请求中说明维护当前 Topic 台账。"
