#!/usr/bin/env bash
# setup-wsl.sh
# WSL2 Ubuntu での初回セットアップスクリプト
# 実行方法（WSL端末で）:
#   bash ~/devops/nexus-sticky/scripts/setup-wsl.sh

set -euo pipefail

PROJ_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "=== nexus-sticky WSL2 セットアップ ==="
echo "プロジェクト: $PROJ_DIR"

# ─── 1. システム依存パッケージ ───────────────────────────────────────
echo ""
echo "[1/6] Tauri システム依存パッケージをインストール中..."
sudo apt-get update -qq
sudo apt-get install -y build-essential pkg-config libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev libayatana-appindicator3-1 librsvg2-dev libsoup-3.0-dev
echo "✓ システムパッケージ完了"

# ─── 2. Rust / Cargo の確認 ─────────────────────────────────────────
echo ""
echo "[2/6] Rust の確認..."
if ! command -v cargo &>/dev/null; then
    if [ -f "$HOME/.cargo/bin/cargo" ]; then
        source "$HOME/.cargo/env"
    else
        echo "Rust をインストール中..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
fi
echo "✓ $(cargo --version)"
echo "✓ $(rustc --version)"

# ─── 3. Tauri CLI のインストール ─────────────────────────────────────
echo ""
echo "[3/6] Tauri CLI のインストール..."
if ! command -v cargo-tauri &>/dev/null && ! cargo tauri --version &>/dev/null 2>&1; then
    cargo install tauri-cli --version "^2" --locked
fi
echo "✓ $(cargo tauri --version)"

# ─── 4. アイコン生成 ─────────────────────────────────────────────────
echo ""
echo "[4/6] アイコン確認..."
cd "$PROJ_DIR"
if [ ! -f "src-tauri/icons/icon.ico" ] && [ -f "src-tauri/icons/icon.png" ]; then
    echo "アイコン生成中..."
    cargo tauri icon src-tauri/icons/icon.png || echo "アイコン生成失敗（スキップ）"
fi
echo "✓ アイコン準備完了"

# ─── 5. GitHub CLI (gh) のインストール ──────────────────────────────────
echo ""
echo "[5/6] GitHub CLI のインストール..."
if ! command -v gh &>/dev/null; then
    sudo apt-get install -y gh
fi
echo "✓ $(gh --version | head -1)"

# ─── 6. 起動の便利設定 ──────────────────────────────────────────────────
echo ""
echo "[6/6] 起動の便利設定..."

# シェルエイリアス（nexus-sticky コマンドで直接起動できるようにする）
RELEASE_BIN="$HOME/devops/nexus-sticky/src-tauri/target/release/nexus-sticky"
ALIAS_LINE="alias nexus-sticky='WEBKIT_DISABLE_COMPOSITING_MODE=1 RUST_LOG=warn $RELEASE_BIN'"
if ! grep -q "alias nexus-sticky" "$HOME/.bashrc"; then
    echo "" >> "$HOME/.bashrc"
    echo "# Nexus Sticky" >> "$HOME/.bashrc"
    echo "$ALIAS_LINE" >> "$HOME/.bashrc"
    echo "✓ ~/.bashrc にエイリアスを追加しました (nexus-sticky)"
else
    echo "✓ エイリアスは既に設定済みです"
fi

# WSLg .desktop エントリ（Windows スタートメニューに表示されるようになる）
DESKTOP_DIR="$HOME/.local/share/applications"
DESKTOP_FILE="$DESKTOP_DIR/nexus-sticky.desktop"
mkdir -p "$DESKTOP_DIR"
cat > "$DESKTOP_FILE" <<DESKTOP
[Desktop Entry]
Name=Nexus Sticky
Comment=Multi-window sticky note app
Exec=bash -c 'WEBKIT_DISABLE_COMPOSITING_MODE=1 RUST_LOG=warn $RELEASE_BIN'
Icon=$HOME/devops/nexus-sticky/src-tauri/icons/icon.png
Terminal=false
Type=Application
Categories=Utility;
DESKTOP
echo "✓ WSLg .desktop エントリを作成しました"
echo "  → Windows スタートメニューに「Nexus Sticky」が表示されます"

echo ""
echo "================================================"
echo "✓ セットアップ完了！"
echo ""
echo "起動方法:"
echo "  [WSL端末]  nexus-sticky              # エイリアスで起動"
echo "  [Windows]  scripts\\launch.bat        # ダブルクリック or スタートアップ登録"
echo ""
echo "ビルドコマンド:"
echo "  cd ~/devops/nexus-sticky"
echo "  make check          # 型チェック（高速）"
echo "  make build          # デバッグビルド"
echo "  make build-release  # リリースビルド（推奨）"
echo "  make test           # テスト実行"
echo "================================================"
