# NextDownloader プロジェクト計画

## プロジェクト目的と背景

NextDownloaderは、さまざまなコンテンツタイプ（動画、音声、ストリーミングなど）をダウンロードするためのクロスプラットフォームアプリケーションです。このプロジェクトの主な目的は以下の通りです：

1. 高速で信頼性の高いダウンロード機能の提供
2. 複数のプラットフォーム（デスクトップ、モバイル）への対応
3. 使いやすく直感的なユーザーインターフェースの実現
4. 拡張性と柔軟性を備えたアーキテクチャの構築

## アーキテクチャ設計

NextDownloaderは、以下のコンポーネントからなるモノレポ構造を採用しています：

```
NextDownloader/
├── core/                   # Rustコアライブラリ
├── ffi/                    # FFIレイヤー（C FFI、UniFFI）
├── apps/                   # アプリケーション
│   ├── desktop/            # デスクトップアプリ（Tauri 2.0）
│   ├── mobile/             # モバイルアプリ（将来的に実装）
├── cli/                    # コマンドラインインターフェース
├── packages/               # 共通パッケージ
├── plugins/                # Tauriプラグイン
└── docs/                   # ドキュメント
```

### コアライブラリ（`core/`）

プラットフォームに依存しない中核機能を実装します：

- ダウンロードマネージャー
- コンテンツタイプ検出
- 依存関係チェック
- ユーティリティ関数
- エラー処理

### FFIレイヤー（`ffi/`）

異なる言語からRustコアを呼び出すためのインターフェースを提供します：

- C FFI：安定性が高く、広く採用されているアプローチ
- UniFFI：多言語バインディングを自動生成（Swift、Kotlin、Python、JavaScript）

### アプリケーション（`apps/`）

各プラットフォーム向けのアプリケーション実装：

- デスクトップ（Tauri 2.0）：Windows、macOS、Linux
- モバイル（将来的に実装）：iOS、Android

### クロスプラットフォーム戦略

短期的には、Tauri 2.0を使用してデスクトップアプリケーションを実装します。将来的には、モバイルプラットフォームにも対応する予定です。

## 技術スタック

### バックエンド

- **Rust**：コアライブラリとバックエンド実装
- **Tauri 2.0**：デスクトップアプリケーションフレームワーク
- **tokio**：非同期ランタイム
- **reqwest**：HTTPクライアント
- **serde**：シリアライズ/デシリアライズ

### フロントエンド

- **React**：UIライブラリ
- **TypeScript**：型安全な開発
- **Chakra UI**：UIコンポーネントライブラリ
- **React Router**：ルーティング

### 開発ツール

- **Cargo**：Rustパッケージマネージャー
- **npm/yarn**：JavaScriptパッケージマネージャー
- **Vite**：フロントエンドビルドツール
- **cbindgen**：C/C++ヘッダー生成
- **UniFFI**：多言語バインディング生成

## ディレクトリ構造と意図

```
NextDownloader/
├── core/                       # Rustコアライブラリ
│   ├── src/
│   │   ├── lib.rs              # ライブラリのエントリポイント
│   │   ├── downloader.rs       # ダウンロードマネージャー
│   │   ├── content_type.rs     # コンテンツタイプ検出
│   │   ├── error.rs            # エラー処理
│   │   ├── config.rs           # 設定管理
│   │   └── utils.rs            # ユーティリティ関数
│   └── Cargo.toml
├── ffi/                        # FFIレイヤー
│   ├── src/
│   │   ├── lib.rs              # FFIのエントリポイント
│   │   ├── c_ffi.rs            # C FFIバインディング
│   │   ├── uniffi.rs           # UniFFIバインディング
│   │   └── nextdownloader.udl  # UniFFIインターフェース定義
│   ├── build.rs                # ビルドスクリプト
│   └── Cargo.toml
├── apps/
│   ├── desktop/                # デスクトップアプリ
│   │   ├── src/                # フロントエンド
│   │   │   ├── main.tsx        # エントリポイント
│   │   │   ├── App.tsx         # メインアプリコンポーネント
│   │   │   ├── components/     # UIコンポーネント
│   │   │   ├── pages/          # ページコンポーネント
│   │   │   ├── api/            # APIクライアント
│   │   │   └── types/          # 型定義
│   │   ├── src-tauri/          # Tauriバックエンド
│   │   │   ├── src/
│   │   │   │   ├── main.rs     # バックエンドのエントリポイント
│   │   │   │   ├── commands/   # Tauriコマンド
│   │   │   │   └── state.rs    # アプリケーション状態
│   │   │   ├── Cargo.toml
│   │   │   └── tauri.conf.json # Tauri設定
│   │   ├── package.json
│   │   └── vite.config.ts
│   └── mobile/                 # モバイルアプリ（将来的に実装）
├── cli/                        # コマンドラインインターフェース
├── docs/                       # ドキュメント
│   ├── PLANNING.md             # プロジェクト計画
│   ├── TASK.md                 # タスク管理
│   └── KNOWLEDGE.md            # 知識ベース
└── README.md                   # プロジェクト概要
```

## コーディング規約とスタイルガイド

### Rust

- Rustの公式スタイルガイドに従う
- `cargo fmt`と`cargo clippy`を使用してコードの品質を維持
- 適切なドキュメンテーションコメントを追加
- エラー処理は`anyhow`と`thiserror`を使用

### TypeScript/React

- ESLintとPrettierを使用してコードの品質を維持
- コンポーネントはReact関数コンポーネントとして実装
- 状態管理はReact HooksとContext APIを使用
- 型安全性を確保するため、適切な型定義を作成

## テスト戦略

### 単体テスト

- Rustコードは`cargo test`を使用してテスト
- React/TypeScriptコードはJestとReact Testing Libraryを使用

### 統合テスト

- FFIレイヤーのテスト
- Tauriコマンドのテスト

### E2Eテスト

- Tauriアプリケーションのテスト（Playwright）

## デプロイメントフロー

### ビルド

- Rustコードは`cargo build`でビルド
- フロントエンドは`vite build`でビルド
- Tauriアプリケーションは`tauri build`でビルド

### パッケージング

- Windows：MSIインストーラー
- macOS：DMGパッケージ
- Linux：AppImage、Debパッケージ

### CI/CD

- GitHub Actionsを使用して自動ビルドとテスト
- リリースパッケージの自動生成

## ロードマップとマイルストーン

### フェーズ1：基本機能の実装（現在）

- プロジェクト構造の再編成
- Rustコアライブラリの実装
- FFIレイヤーの実装
- Tauriデスクトップアプリの実装

### フェーズ2：機能拡張

- YouTube動画ダウンロード機能の実装
- HLS/DASHストリーミングダウンロード機能の実装
- ダウンロード履歴と設定の永続化
- テストの実装とCI/CDパイプラインの構築

### フェーズ3：クロスプラットフォーム展開

- モバイルアプリ（iOS/Android）の実装
- ブラウザ拡張機能の開発
- クラウド連携機能の実装

### フェーズ4：高度な機能

- プラグインシステムの実装
- 高度なダウンロードスケジューリング
- AIを活用したコンテンツ分析と推奨