# NextDownloader

NextDownloaderは、Rustの高性能なダウンロードエンジンとFlutterのクロスプラットフォームUIを組み合わせた、次世代のダウンロードマネージャーです。

## 主な機能

- **高速ダウンロード**: Rustの並行処理能力を活用した高速なダウンロード
- **YouTube動画のダウンロード**: 様々な品質オプションに対応
- **ダウンロード管理**: 一時停止、再開、キャンセル機能
- **プログレス追跡**: リアルタイムのダウンロード進捗表示
- **カスタマイズ可能な設定**: ダウンロードパス、同時ダウンロード数などの設定
- **クロスプラットフォーム**: macOS, Windows, Linux対応予定

## インストール方法

### 開発環境のセットアップ

1. **Rustのインストール**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Flutterのインストール**:
   [Flutter公式サイト](https://flutter.dev/docs/get-started/install)の手順に従ってインストール

3. **依存関係のインストール**:
   ```bash
   cd flutter_app
   flutter pub get
   ```

4. **Rustライブラリのビルド**:
   ```bash
   cd ffi
   cargo build --release
   ```

### アプリケーションの実行

```bash
cd flutter_app
flutter run -d macos
```

## プロジェクト構成

```
NextDownloader/
├── core/                   # Rustコアライブラリ
├── ffi/                    # FFIレイヤー
├── flutter_app/            # Flutterアプリケーション
│   ├── lib/                # Dartコード
│   │   ├── core/           # コア機能
│   │   ├── features/       # 機能モジュール
│   │   └── main.dart       # エントリーポイント
│   └── macos/              # macOS固有の設定
├── docs/                   # ドキュメント
└── README.md               # プロジェクト概要
```

## 開発環境

- **Rust**: 安定版 (1.70.0以上)
- **Flutter**: 3.10.0以上
- **Dart**: 3.0.0以上
- **macOS**: 10.14以上

## ライセンス

MIT License