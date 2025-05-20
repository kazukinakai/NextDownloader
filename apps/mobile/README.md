# NextDownloader Flutter アプリケーション

NextDownloaderのFlutterバージョンです。Rustコアとの連携により、高速で安定したダウンロード機能を提供します。

## 主な機能

- 様々なURLからのファイルダウンロード
- YouTube動画のダウンロード（予定）
- HLS/DASHストリーミングのダウンロード（予定）
- ダウンロード履歴の管理
- 複数同時ダウンロード
- ダウンロード進捗の表示
- プラットフォーム固有の最適化（macOS, Windows, Linux）

## インストールと実行

### 前提条件

- Flutter SDK（3.19.0以上）
- Rust（1.75.0以上）
- Cargo（Rustのパッケージマネージャー）
- 依存ツール（yt-dlp, aria2c, ffmpeg）

### セットアップ

1. リポジトリをクローン

```bash
git clone https://github.com/yourusername/NextDownloader.git
cd NextDownloader/apps/flutter
```

2. Flutterの依存関係をインストール

```bash
flutter pub get
```

3. Rustの依存関係をビルド

```bash
cd rust
cargo build --release
cd ..
```

4. Flutter-Rustブリッジのコード生成（オプション）

```bash
flutter pub run build_runner build
```

5. アプリケーションの実行

```bash
flutter run -d macos  # macOSの場合
flutter run -d windows  # Windowsの場合
flutter run -d linux  # Linuxの場合
```

## 開発環境のセットアップ

1. Flutter SDKのインストール
2. Rustのインストール
3. 依存ツールのインストール
4. VSCodeまたはAndroid Studioの設定

## 利用可能なスクリプト

- `flutter run`: 開発モードでアプリを実行
- `flutter build macos`: macOS向けにビルド
- `flutter build windows`: Windows向けにビルド
- `flutter build linux`: Linux向けにビルド
- `flutter test`: テストを実行

## 貢献方法

1. このリポジトリをフォーク
2. 新しいブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add some amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

## ライセンス

MITライセンスの下で配布されています。詳細は[LICENSE](../../LICENSE)ファイルを参照してください。