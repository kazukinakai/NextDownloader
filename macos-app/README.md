# NextDownloader macOS アプリケーション

NextDownloaderのmacOS版アプリケーションです。Rustで実装されたコア機能を使用して、HLS動画の高速ダウンロードを実現します。

## 機能

- Chrome拡張機能と連携したHLS動画のダウンロード
- yt-dlp、aria2c、ffmpegを内蔵・管理
- 高速なダウンロード処理（Rustコア使用）
- 使いやすいSwiftUIインターフェース

## 開発環境

- macOS 12.0以上
- Xcode 14.0以上
- Rust 1.70.0以上

## ビルド方法

1. Rustコアのビルド

```bash
cd ../core
cargo build --release
```

2. Swiftアプリのビルド

```bash
cd ../macos-app
xcodebuild -scheme NextDownloader -configuration Release
```

## プロジェクト構造

- `NextDownloader/`: Swiftアプリケーションのメインコード
- `NextDownloaderCore/`: RustコアとのFFIブリッジ
- `Resources/`: アプリケーションリソース
