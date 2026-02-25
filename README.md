# Nexus Sticky

デスクトップに常駐するマルチウィンドウ付箋アプリ。
Rust (Tauri 2.0) バックエンドが複数の付箋ウィンドウを一元管理し、リアルタイム同期・透過率制御・ワークスペース自動保存を実現します。

**Windows ネイティブ (.exe) と WSL2/Linux の両方で動作し、履歴・ワークスペースをクロスプラットフォームで共有します。**

---

## 特徴

- **マルチウィンドウ** — 付箋を何枚でも追加・削除
- **フレームレス UI** — OS の標準枠なし、カスタムタイトルバーでドラッグ移動
- **常に最前面 (ピン留め)** — 他のウィンドウに隠れず情報を確認
- **透過率制御** — 20〜100% をスライダーで調整
- **8 色プリセット** — 付箋を色で分類
- **ワークスペース永続化** — 終了時に位置・内容・設定を JSON 保存、起動時に自動復元
- **履歴ウィンドウ** — 閉じた付箋を最大 20 件記録、クリックで復元
- **システムトレイ統合** — 全付箋の一括表示/非表示、新規作成、終了
- **共有データパス** — Windows .exe と WSL Linux ビルドが同一ディレクトリを参照

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

| プラットフォーム | 要件 |
|---|---|
| **Windows 11** | WebView2 内蔵済み。`.exe` をダブルクリックするだけで起動 |
| **Linux** | Ubuntu 22.04 以降。Tauri システムライブラリが必要 |
| **WSL2 + WSLg** | Ubuntu 22.04 以降、WSLg 対応 (Windows 11)。Linux バイナリを Windows デスクトップに表示 |

---

## ダウンロード（推奨）

[GitHub Releases](https://github.com/torifo/nexus-sticky/releases) からお使いの環境に合わせたバイナリをダウンロードしてください。

| ファイル | 対象 |
|---|---|
| `nexus-sticky.exe` | Windows 11 ネイティブ（ダブルクリックで起動） |
| `nexus-sticky` | Linux / WSL2 (Ubuntu) |

---

## セットアップ

### Windows ネイティブ (.exe)

1. [Releases](https://github.com/torifo/nexus-sticky/releases) から `nexus-sticky.exe` をダウンロード
2. 任意のフォルダに配置してダブルクリック

**スタートアップ登録（ログイン時に自動起動）:**
```
Win + R → shell:startup → nexus-sticky.exe のショートカットを配置
```

---

### WSL2 / Linux

```bash
git clone https://github.com/torifo/nexus-sticky.git ~/devops/nexus-sticky
cd ~/devops/nexus-sticky
bash scripts/setup-wsl.sh
```

`setup-wsl.sh` が以下を自動で行います:
- Tauri 必要システムライブラリのインストール
- Rust / Cargo の確認・インストール
- Tauri CLI のインストール
- `mingw-w64` のインストール（Windows .exe クロスコンパイル用）
- `~/.bashrc` への `nexus-sticky` エイリアス登録
- Windows スタートメニュー用 `.desktop` エントリの作成

**初回ビルド:**
```bash
cd ~/devops/nexus-sticky
make build-release
```

---

## 起動方法

### Windows (.exe)

```
nexus-sticky.exe            ← ダブルクリック or コマンドで起動
```

### WSL2 端末

```bash
nexus-sticky                # 通常起動（前回の付箋を自動復元）
nexus-sticky --resume       # 履歴ウィンドウを先に表示して付箋を選択
```

> `setup-wsl.sh` 実行後に `~/.bashrc` へ自動登録されます。
> 新しい端末を開くか `source ~/.bashrc` を実行してから使用してください。

**デバッグモード:**
```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 RUST_LOG=debug \
  ~/devops/nexus-sticky/src-tauri/target/release/nexus-sticky
```

**WSLg スタートメニューから起動:**
`setup-wsl.sh` 実行後、Windows のスタートメニューに「Nexus Sticky」が表示されます。

**cron / 自動起動（WSL2）:**
```bash
# ログイン時に自動起動する場合
echo '@reboot WEBKIT_DISABLE_COMPOSITING_MODE=1 ~/devops/nexus-sticky/src-tauri/target/release/nexus-sticky &' | crontab -
```

---

## データ保存先

Windows .exe と WSL Linux ビルドは **同一ディレクトリ** を参照し、履歴・ワークスペースを共有します。

| 環境 | パス |
|---|---|
| Windows .exe | `%APPDATA%\com.nexus.sticky\` |
| WSL2 (自動検出) | `/mnt/c/Users/{user}/AppData/Roaming/com.nexus.sticky/` |
| Linux ネイティブ | `~/.local/share/com.nexus.sticky/` |

```
com.nexus.sticky/
├── workspace.json   # 現在開いている付箋（位置・内容・設定）
└── history.json     # 最近閉じた付箋の履歴（最大 20 件）
```

> 環境変数 `NEXUS_DATA_DIR` を設定すると任意のパスに変更できます。

---

## 使い方

### システムトレイ

起動するとタスクバー右端にアイコンが表示されます。

| メニュー | 動作 |
|---|---|
| 新規付箋作成 | 付箋ウィンドウを追加 |
| 最近の付箋 | 閉じた付箋の履歴ウィンドウを表示 |
| 全て表示 | 非表示の付箋を全て表示 |
| 全て非表示 | 全ての付箋を画面から隠す |
| 最後に閉じた付箋を復元 | 直前に閉じた付箋を再表示 |
| 終了 | 配置を保存してアプリ終了 |

### 付箋ウィンドウ

| 操作 | 説明 |
|---|---|
| タイトルバーをドラッグ | ウィンドウ移動 |
| 📌 ピン留め | 常に最前面 ON/OFF |
| 🎨 色変更 | 8 色のプリセットから選択 |
| 👁 透過率 | スライダーで 20〜100% を調整 |
| — 最小化 | ウィンドウを最小化 |
| ✕ 閉じる | この付箋を閉じる（履歴に追加） |
| テキストエリア | 最大 10,000 文字 |

### 履歴ウィンドウ

トレイ → 「最近の付箋」または `nexus-sticky --resume` で開きます。

- 閉じた付箋を最大 20 件、新しい順で一覧表示
- 色ドット・プレビューテキスト・経過時間を表示
- クリックで新しい付箋として復元

---

## 開発

### ビルド

```bash
# Linux バイナリ
make build-release

# Windows .exe（WSL2 クロスコンパイル）
cd src-tauri
cargo build --release --target x86_64-pc-windows-gnu
# → target/x86_64-pc-windows-gnu/release/nexus-sticky.exe
```

### コマンド一覧

```bash
make check          # 型チェック（高速）
make test           # テスト実行（8 件）
make build          # デバッグビルド
make build-release  # リリースビルド
make clean          # ビルド成果物を削除
```

### プロジェクト構成

```
nexus-sticky/
├── src/                        # フロントエンド
│   ├── sticky.html             # 付箋ウィンドウ
│   ├── sticky.css
│   ├── sticky.js
│   └── history.html            # 履歴ウィンドウ
├── src-tauri/
│   ├── src/
│   │   ├── main.rs             # エントリーポイント・Tauri コマンド
│   │   ├── nexus.rs            # NexusManager（ウィンドウ管理・履歴永続化）
│   │   ├── event.rs            # EventBus（クロスウィンドウ同期）
│   │   ├── workspace.rs        # JSON 永続化・共有パス解決
│   │   ├── validator.rs        # 位置・サイズ制約
│   │   ├── tray.rs             # システムトレイ
│   │   └── error.rs            # エラー型
│   ├── .cargo/
│   │   └── config.toml         # Windows クロスコンパイル用 linker 設定
│   ├── icons/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/
│   ├── setup-wsl.sh            # WSL 初回セットアップ（全自動）
│   ├── launch.bat              # （旧）Windows WSL 経由起動スクリプト
│   └── launch.ps1              # （旧）PowerShell 起動スクリプト
├── Makefile
└── LICENSE
```

---

## トラブルシューティング

| 症状 | 対処 |
|---|---|
| ウィンドウが表示されない (WSL) | `WEBKIT_DISABLE_COMPOSITING_MODE=1` を付けて起動 |
| `GTK-CRITICAL` 警告 | WSLg の既知の問題。動作には影響なし |
| 履歴が共有されない | `NEXUS_DATA_DIR` を Windows AppData パスに設定 |
| プロセス強制終了 (WSL) | `pkill nexus-sticky` |
| プロセス強制終了 (Windows) | タスクマネージャーで `nexus-sticky.exe` を終了 |

---

## ライセンス

[MIT License](LICENSE)
