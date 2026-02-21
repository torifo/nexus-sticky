# Nexus Sticky

デスクトップに常駐するマルチウィンドウ付箋アプリ。
Rust (Tauri 2.0) バックエンドが複数の付箋ウィンドウを一元管理し、リアルタイム同期・透過率制御・ワークスペース自動保存を実現します。

---

## 特徴

- **マルチウィンドウ** — 付箋を何枚でも追加・削除
- **フレームレス UI** — OS の標準枠なし、カスタムタイトルバーでドラッグ移動
- **常に最前面 (ピン留め)** — 他のウィンドウに隠れず情報を確認
- **透過率制御** — 20〜100% をスライダーで調整
- **8 色プリセット** — 付箋を色で分類
- **ワークスペース永続化** — 終了時に位置・内容・設定を JSON 保存、起動時に自動復元
- **システムトレイ統合** — 全付箋の一括表示/非表示、新規作成、終了

---

## 技術スタック

| レイヤー | 技術 |
|---|---|
| バックエンド | [Rust](https://www.rust-lang.org/) + [Tauri 2.0](https://tauri.app/) |
| フロントエンド | HTML / CSS / Vanilla JS (Webview) |
| IPC | Tauri Commands + Tauri Events |
| 状態管理 | `Arc<Mutex<AppState>>` |
| 永続化 | `serde_json` (JSON) |
| エラー型 | `thiserror` |

---

## 動作環境

| | 要件 |
|---|---|
| **Linux / WSL2** | Ubuntu 22.04 以降、WSLg 対応 (Windows 11) |
| **Rust** | 1.77 以降 (`rustup` でインストール) |
| システムライブラリ | 後述 |

> Windows ネイティブビルド（`.exe`）は今後対応予定です。
> 現在は **WSL2 上でビルド・実行**し、WSLg 経由で Windows デスクトップに表示します。

---

## セットアップ

### 1. リポジトリのクローン

```bash
git clone https://github.com/YOUR_USERNAME/nexus-sticky.git
cd nexus-sticky
```

### 2. システム依存パッケージのインストール（Ubuntu / WSL2）

```bash
sudo apt-get install -y build-essential pkg-config \
    libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev \
    libayatana-appindicator3-dev libayatana-appindicator3-1 \
    librsvg2-dev libsoup-3.0-dev
```

### 3. Rust のインストール（未インストールの場合）

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 4. ビルド

```bash
make build          # デバッグビルド
make build-release  # リリースビルド
```

---

## 起動方法

### WSL 端末から

```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 ./src-tauri/target/debug/nexus-sticky
```

### Windows から（WSL2 経由）

```bat
scripts\launch.bat
```

または PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\launch.ps1
```

**Windows スタートアップ登録**（自動起動）:
`Win + R` → `shell:startup` → `launch.bat` のショートカットを配置

---

## 使い方

### システムトレイ

起動するとタスクバー右端にアイコンが表示されます。

| メニュー | 動作 |
|---|---|
| 新規付箋作成 | 付箋ウィンドウを追加 |
| 全て表示 | 非表示の付箋を全て表示 |
| 全て非表示 | 全ての付箋を画面から隠す |
| 終了 | 配置を保存してアプリ終了 |

### 付箋ウィンドウ

| 操作 | 説明 |
|---|---|
| タイトルバーをドラッグ | ウィンドウ移動 |
| 📌 ピン留め | 常に最前面 ON/OFF |
| 🎨 色変更 | 8 色のプリセットから選択 |
| 👁 透過率 | スライダーで 20〜100% を調整 |
| — 最小化 | ウィンドウを最小化 |
| ✕ 閉じる | この付箋を削除（終了時に自動保存） |
| テキストエリア | 最大 10,000 文字 |

### データ保存先

```
~/.local/share/com.nexus.sticky/workspace.json
```

---

## 開発

```bash
make check          # 型チェック（高速）
make test           # テスト実行
make build-release  # 最適化ビルド
make clean          # ビルド成果物を削除
```

### プロジェクト構成

```
nexus-sticky/
├── src/                        # フロントエンド
│   ├── sticky.html
│   ├── sticky.css
│   └── sticky.js
├── src-tauri/
│   ├── src/
│   │   ├── main.rs             # エントリーポイント・Tauri コマンド
│   │   ├── nexus.rs            # NexusManager（ウィンドウ管理）
│   │   ├── event.rs            # EventBus（クロスウィンドウ同期）
│   │   ├── workspace.rs        # JSON 永続化
│   │   ├── validator.rs        # 位置・サイズ制約
│   │   ├── tray.rs             # システムトレイ
│   │   └── error.rs            # エラー型
│   ├── capabilities/
│   ├── icons/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/
│   ├── launch.bat              # Windows 起動スクリプト
│   ├── launch.ps1              # PowerShell 起動スクリプト
│   └── setup-wsl.sh            # WSL 初回セットアップ
├── Makefile
└── LICENSE
```

---

## トラブルシューティング

| 症状 | 対処 |
|---|---|
| ウィンドウが表示されない | `WEBKIT_DISABLE_COMPOSITING_MODE=1` を付けて起動 |
| `GTK-CRITICAL` 警告 | WSLg の既知の問題。動作には影響なし |
| ログ確認 | `cat /tmp/nexus-sticky.log` |
| プロセス強制終了 | `pkill nexus-sticky` |

---

## ライセンス

[MIT License](LICENSE)
