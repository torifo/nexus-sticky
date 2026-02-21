# nexus-sticky Makefile
# WSL2環境でのビルド・デプロイを管理する

CARGO     := ~/.cargo/bin/cargo
APP_NAME  := nexus-sticky
TAURI_CLI := ~/.cargo/bin/cargo-tauri

# WSL2 ネイティブパス（このMakefileはWSL2内で実行する）
WSL_SRC   := $(shell pwd)

# Windows側の出力先
WIN_BIN_DIR := /mnt/c/Users/Swimm/.local/bin
WIN_APP_DIR := /mnt/c/Users/Swimm/AppData/Local/nexus-sticky

.PHONY: all setup build dev test check clean install help

## デフォルト: ビルド
all: build

## 初回セットアップ: 依存パッケージとTauri CLIのインストール
setup:
	@echo "=== セットアップ ==="
	@echo "--- システム依存パッケージのインストール ---"
	sudo apt-get update -qq
	sudo apt-get install -y build-essential pkg-config libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev libayatana-appindicator3-1 librsvg2-dev libsoup-3.0-dev
	@echo "--- Rust/Cargo の確認 ---"
	. ~/.cargo/env && rustup update stable
	@echo "--- Tauri CLI のインストール ---"
	. ~/.cargo/env && cargo install tauri-cli --version "^2" --locked
	@echo "--- アイコン生成 ---"
	. ~/.cargo/env && cargo tauri icon src-tauri/icons/icon.png 2>/dev/null || \
		echo "アイコン生成をスキップ（icon.pngが存在しません）"
	@echo "セットアップ完了！"

## ビルド（dev/debug）
build:
	@echo "=== ビルド (debug) ==="
	. ~/.cargo/env && cd src-tauri && $(CARGO) build 2>&1
	@echo "ビルド完了: src-tauri/target/debug/$(APP_NAME)"

## ビルド（release）
build-release:
	@echo "=== ビルド (release) ==="
	. ~/.cargo/env && cd src-tauri && $(CARGO) build --release 2>&1
	@echo "ビルド完了: src-tauri/target/release/$(APP_NAME)"

## Tauri アプリ全体をビルド（バンドル含む）
bundle:
	@echo "=== Tauri バンドル ==="
	. ~/.cargo/env && cargo tauri build 2>&1

## 型チェックのみ（コンパイルしない）
check:
	@echo "=== cargo check ==="
	. ~/.cargo/env && cd src-tauri && $(CARGO) check 2>&1

## テスト実行
test:
	@echo "=== テスト ==="
	. ~/.cargo/env && cd src-tauri && $(CARGO) test -- --nocapture 2>&1

## 開発サーバー起動（HMR）
dev:
	@echo "=== 開発モード ==="
	. ~/.cargo/env && cargo tauri dev 2>&1

## WSL2グローバルにインストール
install: build-release
	@echo "=== WSL2へインストール ==="
	sudo cp src-tauri/target/release/$(APP_NAME) /usr/local/bin/
	@echo "インストール完了: /usr/local/bin/$(APP_NAME)"

## ビルド成果物を削除
clean:
	@echo "=== クリーン ==="
	. ~/.cargo/env && cd src-tauri && $(CARGO) clean

## ヘルプ
help:
	@echo "利用可能なコマンド:"
	@echo "  make setup         - 初回セットアップ（依存パッケージ＋Tauri CLI）"
	@echo "  make check         - 型チェック（高速）"
	@echo "  make build         - デバッグビルド"
	@echo "  make build-release - リリースビルド"
	@echo "  make bundle        - Tauriバンドル（.deb/.AppImage など）"
	@echo "  make test          - テスト実行"
	@echo "  make dev           - 開発サーバー起動"
	@echo "  make install       - WSL2グローバルにインストール"
	@echo "  make clean         - ビルド成果物を削除"
