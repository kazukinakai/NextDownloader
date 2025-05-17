# NextDownloader

NextDownloaderは、様々な形式のコンテンツを簡単かつ高速にダウンロードするためのクロスプラットフォームアプリケーションです。

## 主な機能

- **多様なコンテンツタイプ対応**: MP4、HLS、DASH、YouTubeなど様々な形式に対応
- **高速ダウンロード**: 並列ダウンロードとレジューム機能による高速な処理
- **クロスプラットフォーム**: デスクトップ（macOS、Windows、Linux）とモバイル（iOS、Android）に対応
- **使いやすいUI**: モダンで直感的なユーザーインターフェース
- **プラグイン機能**: 拡張可能なプラグインシステム

## インストールと実行方法

### 必要条件

- Rust 1.70以上
- Node.js 18以上
- 依存ツール: yt-dlp, aria2c, ffmpeg

### デスクトップアプリのビルドと実行

```bash
# リポジトリのクローン
git clone https://github.com/yourusername/NextDownloader.git
cd NextDownloader

# 依存関係のインストール
cargo build
cd tauri-app
npm install

# 開発モードで実行
npm run tauri dev

# ビルド
npm run tauri build
```

### モバイルアプリのビルドと実行

```bash
# iOS向けビルド
cd apps/mobile
npm install
npm run tauri ios dev

# Android向けビルド
cd apps/mobile
npm install
npm run tauri android dev
```

## 開発環境のセットアップ

### Rust環境

```bash
# Rustのインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 必要なツールのインストール
cargo install uniffi-bindgen
cargo install tauri-cli
```

### フロントエンド環境

```bash
# Node.jsの依存関係
cd tauri-app
npm install

# 開発サーバーの起動
npm run dev
```

### FFIバインディングの生成

```bash
# UniFFIバインディングの生成
cd ffi/src/uniffi
chmod +x generate_bindings.sh
./generate_bindings.sh
```

## 利用可能なスクリプト

- `npm run dev`: フロントエンド開発サーバーの起動
- `npm run build`: フロントエンドのビルド
- `npm run tauri dev`: Tauriアプリの開発モードでの実行
- `npm run tauri build`: Tauriアプリのビルド
- `./ffi/src/uniffi/generate_bindings.sh`: FFIバインディングの生成

## プロジェクト構造

```
NextDownloader/
├── apps/                     # アプリケーション
│   ├── desktop/              # デスクトップアプリ（Tauri）
│   └── mobile/               # モバイルアプリ（Tauri 2.0対応）
├── cli/                      # コマンドラインインターフェース
├── core/                     # Rustコア機能
├── ffi/                      # FFIレイヤー
├── packages/                 # 共通パッケージ
├── platforms/                # プラットフォーム固有実装
├── plugins/                  # Tauriプラグイン
└── docs/                     # ドキュメント
```

## 貢献方法

1. このリポジトリをフォークします
2. 新しいブランチを作成します (`git checkout -b feature/amazing-feature`)
3. 変更をコミットします (`git commit -m 'Add some amazing feature'`)
4. ブランチにプッシュします (`git push origin feature/amazing-feature`)
5. プルリクエストを作成します

## ライセンス

MITライセンスの下で配布されています。詳細は[LICENSE](LICENSE)ファイルをご覧ください。