# Nexus Sticky — PowerShell 起動スクリプト
# WSL2 (Ubuntu) 経由でアプリを起動する
#
# 使い方:
#   powershell -ExecutionPolicy Bypass -File scripts\launch.ps1
#   スタートアップ登録: Win+R → shell:startup → launch.bat のショートカットを配置

param(
    [string]$WslDist = "Ubuntu",
    [string]$AppBin  = "nexus-sticky"
)

# ── 多重起動チェック ────────────────────────────────────────────────────
$running = (wsl -d $WslDist -- pgrep -x $AppBin 2>$null)
if ($running) { exit 0 }

# ── バイナリを探す（release → /usr/local/bin → debug の順）──────────────
# bash の || や | を Windows に横取りされないよう呼び出しを分割する
$wslAppPath = ""

if (-not $wslAppPath) {
    $p = (wsl -d $WslDist -- bash -c "ls ~/devops/nexus-sticky/src-tauri/target/release/$AppBin 2>/dev/null") 2>$null
    if ($p) { $wslAppPath = $p.Trim() }
}
if (-not $wslAppPath) {
    $p = (wsl -d $WslDist -- bash -c "command -v $AppBin 2>/dev/null") 2>$null
    if ($p) { $wslAppPath = $p.Trim() }
}
if (-not $wslAppPath) {
    $p = (wsl -d $WslDist -- bash -c "ls ~/devops/nexus-sticky/src-tauri/target/debug/$AppBin 2>/dev/null") 2>$null
    if ($p) { $wslAppPath = $p.Trim() }
}

if (-not $wslAppPath) {
    Add-Type -AssemblyName System.Windows.Forms
    $msg  = "nexus-sticky が WSL '$WslDist' 内に見つかりません。`n`n"
    $msg += "ビルドしてください:`n"
    $msg += "  wsl -d $WslDist bash -c `"source ~/.cargo/env && cd ~/devops/nexus-sticky/src-tauri && cargo build --release`""
    [System.Windows.Forms.MessageBox]::Show($msg, "Nexus Sticky", "OK", "Error") | Out-Null
    exit 1
}

# ── Start-Process で WSL を独立プロセスとして起動 ─────────────────────────
# nohup + & ではなく、WSL プロセス自体を PowerShell から切り離す
# これにより起動スクリプト終了後もアプリが生き続ける
$bashCmd = "WEBKIT_DISABLE_COMPOSITING_MODE=1 RUST_LOG=warn $wslAppPath >/tmp/nexus-sticky.log 2>&1"
Start-Process -FilePath "wsl.exe" `
    -ArgumentList @("-d", $WslDist, "--", "bash", "-c", $bashCmd) `
    -WindowStyle Hidden
