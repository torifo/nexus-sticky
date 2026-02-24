# Nexus Sticky — PowerShell 起動スクリプト
# WSL2 (Ubuntu) 経由でアプリを起動する
#
# 使い方:
#   powershell -ExecutionPolicy Bypass -File scripts\launch.ps1
#   スタートアップ登録: Win+R → shell:startup → このファイルのショートカットを配置

param(
    [string]$WslDist = "Ubuntu",
    [string]$AppBin  = "nexus-sticky"
)

# WSL 内でバイナリパスを自動検索（release → debug の順で優先）
$wslAppPath = wsl -d $WslDist -- bash --norc --noprofile -c `
    "which $AppBin 2>/dev/null || find ~/devops -path '*/release/$AppBin' -type f 2>/dev/null | head -1 || find ~/devops -path '*/debug/$AppBin' -type f 2>/dev/null | head -1"

if (-not $wslAppPath) {
    Write-Error "$AppBin not found in WSL '$WslDist'."
    Write-Host "Build first: wsl -d $WslDist -- bash -c 'cd ~/devops/nexus-sticky && make build'"
    exit 1
}

# 多重起動を防止
$pid = wsl -d $WslDist -- pgrep -x $AppBin
if ($pid) {
    Write-Host "Nexus Sticky is already running (PID: $pid)"
    exit 0
}

# バックグラウンドで起動
$cmd = "WEBKIT_DISABLE_COMPOSITING_MODE=1 RUST_LOG=warn nohup $wslAppPath > /tmp/nexus-sticky.log 2>&1 &"
wsl -d $WslDist -- bash --norc --noprofile -c $cmd

Write-Host "Nexus Sticky started."
Write-Host "Log: wsl -d $WslDist -- cat /tmp/nexus-sticky.log"
