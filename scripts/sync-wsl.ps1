# sync-wsl.ps1
# Windows側ソースをWSL2のネイティブファイルシステムへ同期する
# 使い方: powershell -ExecutionPolicy Bypass -File scripts\sync-wsl.ps1

$ErrorActionPreference = "Stop"

$SRC = "C:\Users\Swimm\develop\opus\nexus-sticky"
$DST = "\\wsl.localhost\Ubuntu\home\toriforiumu\devops\nexus-sticky"

# 除外パターン
$EXCLUDES = @(
    ".git",
    "target",
    "dist",
    "node_modules",
    "*.exe",
    "scripts"
)

Write-Host "=== nexus-sticky WSL2 同期 ===" -ForegroundColor Cyan
Write-Host "コピー元: $SRC"
Write-Host "コピー先: $DST"

# コピー先ディレクトリを作成
if (-not (Test-Path $DST)) {
    New-Item -ItemType Directory -Path $DST -Force | Out-Null
    Write-Host "コピー先ディレクトリを作成しました: $DST" -ForegroundColor Green
}

# ファイルを同期
$items = Get-ChildItem -Path $SRC -Recurse -Force

foreach ($item in $items) {
    # 除外チェック
    $skip = $false
    foreach ($exc in $EXCLUDES) {
        if ($item.FullName -like "*\$exc\*" -or $item.FullName -like "*\$exc") {
            $skip = $true
            break
        }
        if ($item.Name -like $exc) {
            $skip = $true
            break
        }
    }
    if ($skip) { continue }

    # 相対パスを計算してコピー先を決定
    $rel = $item.FullName.Substring($SRC.Length).TrimStart('\')
    $dest = Join-Path $DST $rel

    if ($item.PSIsContainer) {
        if (-not (Test-Path $dest)) {
            New-Item -ItemType Directory -Path $dest -Force | Out-Null
        }
    } else {
        $destDir = Split-Path $dest -Parent
        if (-not (Test-Path $destDir)) {
            New-Item -ItemType Directory -Path $destDir -Force | Out-Null
        }
        Copy-Item -Path $item.FullName -Destination $dest -Force
    }
}

Write-Host "同期完了！" -ForegroundColor Green
Write-Host ""
Write-Host "次のステップ (WSL2内でビルド):" -ForegroundColor Yellow
Write-Host "  wsl -d Ubuntu -- bash -c 'cd ~/devops/nexus-sticky && make build'"
