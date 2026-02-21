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
echo "[1/4] Tauri システム依存パッケージをインストール中..."
sudo apt-get update -qq
sudo apt-get install -y build-essential pkg-config libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev libayatana-appindicator3-1 librsvg2-dev libsoup-3.0-dev
echo "✓ システムパッケージ完了"

# ─── 2. Rust / Cargo の確認 ─────────────────────────────────────────
echo ""
echo "[2/4] Rust の確認..."
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
echo "[3/4] Tauri CLI のインストール..."
if ! command -v cargo-tauri &>/dev/null && ! cargo tauri --version &>/dev/null 2>&1; then
    cargo install tauri-cli --version "^2" --locked
fi
echo "✓ $(cargo tauri --version)"

# ─── 4. アイコン生成 ─────────────────────────────────────────────────
echo ""
echo "[4/4] アイコン確認..."
cd "$PROJ_DIR"
if [ ! -f "src-tauri/icons/icon.ico" ] && [ -f "src-tauri/icons/icon.png" ]; then
    echo "アイコン生成中..."
    cargo tauri icon src-tauri/icons/icon.png || echo "アイコン生成失敗（スキップ）"
fi
echo "✓ アイコン準備完了"

echo ""
echo "================================================"
echo "✓ セットアップ完了！"
echo ""
echo "ビルドコマンド:"
echo "  cd ~/devops/nexus-sticky"
echo "  make check          # 型チェック（高速）"
echo "  make build          # デバッグビルド"
echo "  make test           # テスト実行"
echo "================================================"
