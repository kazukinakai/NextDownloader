# NextDownloader

NextDownloaderは、さまざまなコンテンツタイプ（動画、音声、ストリーミングなど）をダウンロードするための高速で使いやすいクロスプラットフォームアプリケーションです。

## 主な機能

- **多様なコンテンツタイプ対応**: MP4、HLS、DASH、YouTubeなど様々な形式に対応
- **高速ダウンロード**: 並列ダウンロードによる高速化
- **直感的なUI**: 使いやすく美しいユーザーインターフェース
- **クロスプラットフォーム**: Windows、macOS、Linuxで動作
- **カスタマイズ可能**: 様々な設定オプションでニーズに合わせた調整が可能

## インストール方法

### 必要条件

- **依存ツール**:
  - yt-dlp: YouTubeなどの動画サイトからのダウンロード
  - aria2c: 高速ダウンロード
  - ffmpeg: メディア処理

### バイナリからのインストール

1. [リリースページ](https://github.com/kazukinakai/NextDownloader/releases)から最新のバイナリをダウンロード
2. お使いのプラットフォームに合わせたインストーラーを実行

### ソースからのビルド

```bash
# リポジトリのクローン
git clone https://github.com/kazukinakai/NextDownloader.git
cd NextDownloader

# 依存関係のインストール
cd apps/desktop
npm install

# 開発サーバーの起動
npm run tauri dev

# ビルド
npm run tauri build
```

## 使用方法

1. アプリケーションを起動
2. ダウンロードしたいコンテンツのURLを入力
3. 保存先ディレクトリを選択
4. 必要に応じてフォーマットを選択
5. 「ダウンロード開始」ボタンをクリック

## 開発環境のセットアップ

### 必要なツール

- Rust 1.76.0以上
- Node.js 18以上
- npm 9以上

### セットアップ手順

```bash
# Rustのインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.jsとnpmのインストール（お使いのプラットフォームに合わせて）

# Tauriの開発に必要なシステム依存関係のインストール
# macOSの場合
xcode-select --install

# Linuxの場合（Ubuntuの例）
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Windowsの場合
# Visual Studio 2019以上とRust toolchainをインストール

# リポジトリのクローンと依存関係のインストール
git clone https://github.com/kazukinakai/NextDownloader.git
cd NextDownloader
cd apps/desktop
npm install
```

## 利用可能なスクリプト

- `npm run tauri dev`: 開発サーバーの起動
- `npm run tauri build`: アプリケーションのビルド
- `npm run lint`: コードリントの実行
- `npm run test`: テストの実行

## プロジェクト構造

```
NextDownloader/
├── core/                   # Rustコアライブラリ
├── ffi/                    # FFIレイヤー
├── apps/                   # アプリケーション
│   ├── desktop/            # デスクトップアプリ（Tauri 2.0）
│   ├── mobile/             # モバイルアプリ（将来的に実装）
├── cli/                    # コマンドラインインターフェース
├── packages/               # 共通パッケージ
├── plugins/                # Tauriプラグイン
└── docs/                   # ドキュメント
```

## 貢献方法

1. このリポジトリをフォーク
2. 新しいブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add some amazing feature'`)
4. ブランチをプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

## ライセンス

MITライセンスの下で配布されています。詳細は[LICENSE](LICENSE)ファイルを参照してください。