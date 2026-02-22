# Contributing to Nexus Sticky

## 開発環境の準備

### 必要なもの
- WSL2 (Ubuntu 22.04 以降) または Linux
- Rust 1.77 以降
- システムライブラリ（`scripts/setup-wsl.sh` で一括インストール可）

### 初回セットアップ

```bash
git clone https://github.com/torifo/nexus-sticky.git
cd nexus-sticky
bash scripts/setup-wsl.sh   # 依存パッケージ + Tauri CLI を自動インストール
```

## ビルドとテスト

```bash
make check          # 型チェック（PR 前に必ず実行）
make test           # ユニットテスト・プロパティテスト（8 件）
make build          # デバッグビルド
make build-release  # リリースビルド
```

## Windows 環境での開発フロー（WSL2 ビルド）

Windows 上でコードを編集し、WSL2 でビルドする場合:

```powershell
# ソースを WSL2 に同期してビルド・チェック
powershell -ExecutionPolicy Bypass -File scripts\build-wsl.ps1 check
powershell -ExecutionPolicy Bypass -File scripts\build-wsl.ps1 build
```

`scripts/build-wsl.ps1` は内部的に `rsync` または `cp` でソースを WSL2 ネイティブ
ファイルシステムへコピーしてからビルドします（I/O パフォーマンス最適化）。

## アーキテクチャ概要

```
NexusManager          中央制御。ウィンドウの生成・削除・状態管理
EventBus              Tauri Events を使ったクロスウィンドウ同期
WorkspaceManager      serde_json による永続化
WindowValidator       画面境界・最小/最大サイズの制約
SystemTray            OS トレイアイコンとメニュー
```

イベントフロー（コンテンツ編集の例）:

```
JS (200ms debounce)
  → invoke('update_content')
  → NexusManager::update_content()
  → EventBus::emit_to_all(ContentChanged)
  → 全付箋の JS が listen で受信 → UI 更新
```

## アイコンの再生成

`src-tauri/icons/` に 256×256 以上の RGBA PNG を用意してから:

```bash
cargo tauri icon src-tauri/icons/icon.png
```

## コードスタイル

```bash
cargo fmt       # フォーマット
cargo clippy    # Lint
```

PR は `cargo fmt` と `cargo clippy` をパスした状態で送ってください。
