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
$running = (wsl -d $WslDist pgrep -x $AppBin 2>$null)
if ($running) { exit 0 }

# ── バイナリを探す（release → /usr/local/bin → debug の順）──────────────
$wslAppPath = ""

if (-not $wslAppPath) {
    $p = (wsl -d $WslDist bash -c "ls ~/devops/nexus-sticky/src-tauri/target/release/$AppBin 2>/dev/null") 2>$null
    if ($p) { $wslAppPath = $p.Trim() }
}
if (-not $wslAppPath) {
    $p = (wsl -d $WslDist bash -c "command -v $AppBin 2>/dev/null") 2>$null
    if ($p) { $wslAppPath = $p.Trim() }
}
if (-not $wslAppPath) {
    $p = (wsl -d $WslDist bash -c "ls ~/devops/nexus-sticky/src-tauri/target/debug/$AppBin 2>/dev/null") 2>$null
    if ($p) { $wslAppPath = $p.Trim() }
}

if (-not $wslAppPath) {
    Add-Type -AssemblyName System.Windows.Forms
    $msg  = "nexus-sticky が WSL '$WslDist' 内に見つかりません。`n`n"
    $msg += "ビルドしてください:`n"
    $msg += "wsl -d $WslDist bash -c `"source ~/.cargo/env && cd ~/devops/nexus-sticky/src-tauri && cargo build --release`""
    [System.Windows.Forms.MessageBox]::Show($msg, "Nexus Sticky", "OK", "Error") | Out-Null
    exit 1
}

# ── WScript.Shell.Run で WSL を完全非表示起動 ─────────────────────────────
# Start-Process -WindowStyle Hidden は WSL に必要なコンソールを提供できないため
# WScript.Shell.Run(cmd, 0, false) を使う（0 = 非表示、false = 非同期）
$bashCmd = "WEBKIT_DISABLE_COMPOSITING_MODE=1 RUST_LOG=warn $wslAppPath"
$wsh = New-Object -ComObject WScript.Shell
$wsh.Run("wsl -d $WslDist bash -c ""$bashCmd""", 0, $false)
