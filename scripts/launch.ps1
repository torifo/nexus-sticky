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
# ※ bash の || や | を Windows cmd に横取りされないよう、呼び出しを分割する
$wslAppPath = ""

# 1. リリースビルド（最優先）
if (-not $wslAppPath) {
    $p = (wsl -d $WslDist -- bash -c "ls ~/devops/nexus-sticky/src-tauri/target/release/$AppBin 2>/dev/null") 2>$null
    if ($p) { $wslAppPath = $p.Trim() }
}

# 2. システムインストール済み（make install 後）
if (-not $wslAppPath) {
    $p = (wsl -d $WslDist -- bash -c "command -v $AppBin 2>/dev/null") 2>$null
    if ($p) { $wslAppPath = $p.Trim() }
}

# 3. デバッグビルド（フォールバック）
if (-not $wslAppPath) {
    $p = (wsl -d $WslDist -- bash -c "ls ~/devops/nexus-sticky/src-tauri/target/debug/$AppBin 2>/dev/null") 2>$null
    if ($p) { $wslAppPath = $p.Trim() }
}

if (-not $wslAppPath) {
    $msg  = "nexus-sticky が WSL '$WslDist' 内に見つかりません。`n`n"
    $msg += "ビルドしてください:`n"
    $msg += "  wsl -d $WslDist -- bash -c `"source ~/.cargo/env && cd ~/devops/nexus-sticky/src-tauri && cargo build --release`""
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.MessageBox]::Show($msg, "Nexus Sticky", "OK", "Error") | Out-Null
    exit 1
}

# ── バックグラウンドで起動 ────────────────────────────────────────────────
# --norc --noprofile を使わず、フル bash 環境で DISPLAY/WAYLAND_DISPLAY を継承する
$launchCmd = "WEBKIT_DISABLE_COMPOSITING_MODE=1 RUST_LOG=warn nohup $wslAppPath >/tmp/nexus-sticky.log 2>&1 &"
wsl -d $WslDist -- bash -c $launchCmd
