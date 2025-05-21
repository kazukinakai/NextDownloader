# NextDownloader プロジェクト計画

## プロジェクト目的と背景

NextDownloaderは、Rustの高性能な処理能力とFlutterのクロスプラットフォームUIを組み合わせた次世代のダウンロードマネージャーです。このプロジェクトの目的は、以下の特徴を持つダウンロードマネージャーを実現することです：

1. **高性能**: Rustの並行処理能力を活用した高速なダウンロード処理
2. **使いやすさ**: 直感的なUIとシンプルな操作性
3. **拡張性**: プラグインアーキテクチャによる機能拡張
4. **クロスプラットフォーム**: 複数のプラットフォームでの一貫した体験

## アーキテクチャ設計

### モノレポ構成

```
NextDownloader/
├── core/                   # Rustコアライブラリ（プラットフォーム非依存）
├── ffi/                    # FFIレイヤー（言語間連携）
├── flutter_app/            # Flutterアプリケーション
├── tauri-app/              # Tauriアプリケーション（将来的に実装予定）
├── cli/                    # コマンドラインインターフェース（将来的に実装予定）
└── docs/                   # ドキュメント
```

### クリーンアーキテクチャ

Flutterアプリケーションは、クリーンアーキテクチャに基づいて実装されています：

```
flutter_app/
├── lib/
│   ├── core/               # コア機能
│   │   ├── data/           # データ層
│   │   ├── domain/         # ドメイン層
│   │   └── presentation/   # プレゼンテーション層
│   ├── features/           # 機能モジュール
│   │   ├── download/       # ダウンロード機能
│   │   └── settings/       # 設定機能
│   └── main.dart           # エントリーポイント
```

各機能モジュールは、以下の層に分かれています：

1. **ドメイン層**:
   - エンティティ: ビジネスオブジェクト
   - リポジトリインターフェース: データアクセスの抽象化
   - ユースケース: ビジネスロジック

2. **データ層**:
   - モデル: エンティティに対応するデータモデル
   - リポジトリ実装: データアクセスの実装
   - データソース: データの取得元（Rustブリッジなど）

3. **プレゼンテーション層**:
   - プロバイダー: 状態管理
   - ウィジェット: UI要素
   - 画面: 完全なUI画面

### Flutter-Rust連携

Flutter-Rust連携は、以下の方法で実現されています：

1. **FFIレイヤー**: C言語のFFIを介してRustとDartを連携
2. **flutter_rust_bridge**: 型安全なバインディング生成
3. **非同期処理**: Futureを使用した非同期通信

## 技術スタック

### コアライブラリ（Rust）

- **reqwest**: HTTPクライアント
- **tokio**: 非同期ランタイム
- **serde**: シリアライズ/デシリアライズ
- **anyhow**: エラー処理
- **log**: ロギング

### FFIレイヤー

- **flutter_rust_bridge**: Flutter-Rust連携
- **cbindgen**: C言語バインディング生成

### Flutterアプリケーション

- **flutter_riverpod**: 状態管理
- **freezed**: イミュータブルなデータモデル
- **json_serializable**: JSONシリアライズ/デシリアライズ
- **path_provider**: ファイルシステムアクセス
- **shared_preferences**: 設定の永続化

## ディレクトリ構造

詳細なディレクトリ構造は以下の通りです：

```
NextDownloader/
├── core/
│   ├── src/
│   │   ├── download/       # ダウンロード機能
│   │   ├── youtube/        # YouTube動画処理
│   │   ├── settings/       # 設定管理
│   │   └── lib.rs          # エントリーポイント
│   ├── Cargo.toml          # 依存関係
│   └── README.md           # ドキュメント
├── ffi/
│   ├── src/
│   │   ├── flutter_bridge.rs # Flutterブリッジ
│   │   └── lib.rs          # エントリーポイント
│   ├── Cargo.toml          # 依存関係
│   └── README.md           # ドキュメント
├── flutter_app/
│   ├── lib/
│   │   ├── core/
│   │   │   ├── data/
│   │   │   │   ├── datasources/
│   │   │   │   ├── models/
│   │   │   │   └── repositories/
│   │   │   ├── domain/
│   │   │   │   ├── entities/
│   │   │   │   ├── repositories/
│   │   │   │   └── usecases/
│   │   │   └── presentation/
│   │   │       ├── routes/
│   │   │       └── app.dart
│   │   ├── features/
│   │   │   ├── download/
│   │   │   │   ├── data/
│   │   │   │   ├── domain/
│   │   │   │   └── presentation/
│   │   │   └── settings/
│   │   │       ├── data/
│   │   │       ├── domain/
│   │   │       └── presentation/
│   │   └── main.dart
│   ├── macos/
│   │   ├── Libs/           # Rustライブラリ
│   │   ├── Runner/
│   │   ├── Podfile
│   │   └── rust_build.sh   # Rustビルドスクリプト
│   ├── pubspec.yaml
│   └── README.md
└── docs/
    ├── PLANNING.md         # プロジェクト計画
    ├── TASK.md             # タスク管理
    └── KNOWLEDGE.md        # 知識ベース
```

## コーディング規約とスタイルガイド

### Rust

- **rustfmt**: コードフォーマット
- **clippy**: リンター
- **エラー処理**: `anyhow`と`thiserror`を使用
- **非同期処理**: `async/await`と`tokio`を使用

### Dart/Flutter

- **dartfmt**: コードフォーマット
- **lint**: 厳格なリントルールを適用
- **状態管理**: Riverpodを使用
- **イミュータブルなデータ**: freezedを使用

## テスト戦略

### ユニットテスト

- **Rust**: `cargo test`を使用
- **Dart**: `flutter test`を使用

### 統合テスト

- **Flutter-Rust連携**: FFIの統合テスト
- **UI/UX**: Flutterの統合テスト

### パフォーマンステスト

- **ダウンロード速度**: 様々なネットワーク環境でのテスト
- **メモリ使用量**: 大きなファイルのダウンロード時のメモリ使用量

## デプロイメントフロー

### macOS

1. Rustライブラリのビルド
2. Flutterアプリケーションのビルド
3. アプリケーションの署名とノータリゼーション
4. DMGパッケージの作成

### Windows（将来的に実装予定）

1. Rustライブラリのビルド
2. Flutterアプリケーションのビルド
3. インストーラーの作成

### Linux（将来的に実装予定）

1. Rustライブラリのビルド
2. Flutterアプリケーションのビルド
3. Snapパッケージの作成

## ロードマップとマイルストーン

### マイルストーン1: 基本機能の実装（現在）

- [x] プロジェクト構造の設定
- [x] Rustコアライブラリの実装
- [x] FFIレイヤーの実装
- [x] Flutterアプリケーションの基本UI
- [x] ダウンロード機能の実装
- [x] 設定機能の実装

### マイルストーン2: 拡張機能の実装

- [ ] YouTube動画ダウンロードの実装
- [ ] HLS/DASHストリーミングのサポート
- [ ] ダウンロード履歴の永続化
- [ ] 詳細な統計情報の表示

### マイルストーン3: クロスプラットフォーム対応

- [ ] Windows対応
- [ ] Linux対応
- [ ] Tauriアプリケーションの実装

### マイルストーン4: 高度な機能

- [ ] プラグインシステムの実装
- [ ] バッチダウンロード
- [ ] スケジュールダウンロード
- [ ] クラウド連携