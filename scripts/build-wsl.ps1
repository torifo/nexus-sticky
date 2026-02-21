# build-wsl.ps1
# 同期→WSL2ビルドを一括実行するスクリプト
# 使い方: powershell -ExecutionPolicy Bypass -File scripts\build-wsl.ps1 [check|build|test|setup]

param(
    [string]$Target = "check"
)

$ErrorActionPreference = "Stop"

$WIN_SRC   = "C:\Users\Swimm\develop\opus\nexus-sticky"
$WSL_USER  = "toriforiumu"
$WSL_DST   = "/home/$WSL_USER/devops/nexus-sticky"
$WSL_DIST  = "Ubuntu"

Write-Host "=== nexus-sticky ビルドパイプライン ===" -ForegroundColor Cyan
Write-Host "ターゲット: $Target" -ForegroundColor Yellow

# ─── Step 1: WSL2にコピー ───────────────────────────────────────────
Write-Host "`n[1/3] ソースをWSL2ネイティブFSへコピー中..." -ForegroundColor Cyan

$EXCLUDES = @(".git", "target", "dist", "node_modules")

function Sync-ToWSL {
    param($Src, $Dst)

    $rsyncCmd = "rsync -av --delete " + ($EXCLUDES | ForEach-Object { "--exclude='$_' " }) + `
        "/mnt/c/Users/Swimm/develop/opus/nexus-sticky/ $Dst/"

    wsl -d $WSL_DIST -- bash --norc --noprofile -c $rsyncCmd 2>&1
}

# rsync が使えるか確認、なければ直接コピー
$rsyncAvail = wsl -d $WSL_DIST -- bash --norc --noprofile -c "which rsync 2>/dev/null"
if ($LASTEXITCODE -eq 0 -and $rsyncAvail) {
    Write-Host "rsync を使用して同期します..."
    $excludeArgs = ($EXCLUDES | ForEach-Object { "--exclude='$_'" }) -join " "
    $cmd = "rsync -a --delete $excludeArgs /mnt/c/Users/Swimm/develop/opus/nexus-sticky/ $WSL_DST/"
    wsl -d $WSL_DIST -- bash --norc --noprofile -c "mkdir -p $WSL_DST && $cmd"
} else {
    Write-Host "rsync が見つからないため cp でコピーします..."
    $cmd = "mkdir -p $WSL_DST && cp -r /mnt/c/Users/Swimm/develop/opus/nexus-sticky/. $WSL_DST/"
    wsl -d $WSL_DIST -- bash --norc --noprofile -c $cmd
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "コピー失敗" -ForegroundColor Red
    exit 1
}
Write-Host "コピー完了" -ForegroundColor Green

# ─── Step 2: WSL2でビルド ────────────────────────────────────────────
Write-Host "`n[2/3] WSL2でビルド: make $Target ..." -ForegroundColor Cyan

$buildCmd = "cd $WSL_DST && . ~/.cargo/env && make $Target"
wsl -d $WSL_DIST -- bash --norc --noprofile -c $buildCmd

if ($LASTEXITCODE -ne 0) {
    Write-Host "ビルド失敗" -ForegroundColor Red
    exit 1
}

Write-Host "`n[3/3] 完了！" -ForegroundColor Green

if ($Target -eq "build-release") {
    Write-Host "`nバイナリの場所 (WSL2):" -ForegroundColor Yellow
    Write-Host "  $WSL_DST/src-tauri/target/release/nexus-sticky"
    Write-Host "`nWindows側から参照:"
    Write-Host "  \\wsl.localhost\Ubuntu\home\$WSL_USER\devops\nexus-sticky\src-tauri\target\release\"
}
