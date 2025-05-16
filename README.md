# NextDownloader-Rust

NextDownloader はマルチプラットフォーム対応の高速動画ダウンローダーです。macOS, Windows, Linux で同一コードベースで実行できます。

## 特徴

- 複数の接続を使用した高速ダウンロード（aria2c）
- YouTube や他のサイトからのダウンロードをサポート（yt-dlp）
- HLS, DASH, 直接MP4ダウンロードなど複数のフォーマットに対応
- クロスプラットフォーム（Windows, macOS, Linux）
- コマンドラインインターフェース（CLI）とグラフィカルユーザーインターフェース（GUI）の両方を提供

## インストール方法

### 依存関係

以下のツールが必要です：

- yt-dlp
- aria2c
- ffmpeg

#### macOS:

```bash
brew install yt-dlp aria2 ffmpeg
```

#### Ubuntu/Debian:

```bash
sudo apt update
sudo apt install aria2 ffmpeg
pip install yt-dlp
```

#### Windows:

```bash
winget install yt-dlp
winget install aria2
winget install ffmpeg
```

### ビルド方法

```bash
# CLIアプリケーションをビルド
cargo build --release -p nextdownloader-cli

# GUIアプリケーションをビルド
cd gui
npm install
npm run tauri build
```

## 使用方法

### CLIモード

```bash
# URLから動画をダウンロード
nextdownloader-cli download --url https://example.com/video.mp4 --output ~/Downloads --filename myvideo

# システム状態をチェック
nextdownloader-cli check
```

### GUIモード

アプリケーションを起動し、URLを入力してダウンロードボタンをクリックするだけです。

## ライセンス

MIT

## 開発者

NextDownloader Team
