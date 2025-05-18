# NextDownloader 知識ベース

## 技術スタックの最新情報

### Rust

- **バージョン**: 1.76.0以上を推奨
- **非同期処理**: tokio 1.36.0が最新で安定
- **エラー処理**: anyhow 1.0.79とthiserror 1.0.57の組み合わせが効果的
- **シリアライズ**: serde 1.0.197が最新

### Tauri

- **バージョン**: 2.0.0-rc.8が最新のリリース候補
- **プラグイン**: 2.0.0-rc.4シリーズが最新
- **特徴**: 
  - WebViewベースのクロスプラットフォームアプリケーションフレームワーク
  - デスクトップとモバイルの両方をサポート
  - セキュリティ重視の設計

### React/TypeScript

- **React**: 18.2.0が最新の安定版
- **TypeScript**: 5.3.3以上を推奨
- **ビルドツール**: Vite 5.0.0以上が高速で効率的

### FFI技術

- **C FFI**: cbindgen 0.26.0が最新
- **UniFFI**: 0.25.3が最新で多言語バインディングを自動生成
- **対応言語**: Swift、Kotlin、Python、JavaScript

## 発見したベストプラクティス

### Rustコード構造

- **モジュール分割**: 機能ごとに明確に分離されたモジュール構造
- **エラー処理**: 専用のエラータイプとエラーコードの定義
- **非同期API**: すべての長時間実行操作は非同期APIとして実装
- **テスト**: 各モジュールに単体テストを含める

### FFIレイヤー

- **C FFI**: 低レベルな相互運用性のための標準的なアプローチ
- **UniFFI**: より高レベルで使いやすいバインディング
- **メモリ管理**: 所有権の明確な移譲と適切なリソース解放
- **エラー処理**: エラーコードとエラーメッセージの適切な変換

### Tauriアプリケーション

- **状態管理**: アプリケーション状態の集中管理
- **コマンド**: バックエンドとフロントエンド間の明確なインターフェース
- **プラグイン**: 機能拡張のためのプラグインシステムの活用
- **セキュリティ**: 最小権限の原則に基づいた設定

### フロントエンド

- **コンポーネント設計**: 再利用可能な小さなコンポーネント
- **型安全**: TypeScriptの厳格なタイプチェック
- **状態管理**: React HooksとContext APIの活用
- **スタイリング**: Chakra UIによる一貫したデザインシステム

## トラブルシューティングと解決策

### Rust/FFI関連

- **問題**: C FFIでの文字列処理
  - **解決策**: CStringとCStrの適切な使用、所有権の明確な管理

- **問題**: 非同期APIのFFI経由での呼び出し
  - **解決策**: tokio::runtime::Runtimeを使用して非同期コードをブロッキング呼び出しに変換

### Tauri関連

- **問題**: Tauriコマンドでの非同期処理
  - **解決策**: async/awaitを使用し、適切なエラーハンドリングを実装

- **問題**: アプリケーション状態の管理
  - **解決策**: Arc<Mutex<T>>やArc<RwLock<T>>を使用した共有状態

### フロントエンド関連

- **問題**: TypeScriptの型定義
  - **解決策**: インターフェースの明確な定義と共有

- **問題**: 非同期APIの呼び出し
  - **解決策**: try-catchブロックでのエラーハンドリングと適切なローディング状態の管理

## 重要な設計判断とその理由

### モノレポ構造の採用

- **判断**: プロジェクトをモノレポ構造で組織化
- **理由**: コード共有の促進、依存関係の一元管理、一貫したビルドプロセス

### FFIレイヤーの分離

- **判断**: コアライブラリとFFIを分離
- **理由**: 関心事の明確な分離、テスト容易性の向上、メンテナンス性の向上

### Tauri 2.0の採用

- **判断**: Tauri 2.0を使用してデスクトップアプリを実装
- **理由**: 軽量、高性能、セキュア、将来的なモバイル対応

### Chakra UIの採用

- **判断**: UIライブラリとしてChakra UIを選択
- **理由**: アクセシビリティ、カスタマイズ性、テーマ対応、コンポーネントの豊富さ

## 学習リソースと参考資料

### Rust

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)

### FFI

- [Rust FFI Guide](https://michael-f-bryan.github.io/rust-ffi-guide/)
- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/)

### Tauri

- [Tauri 2.0 Documentation](https://tauri.app/v2/guides/)
- [Tauri API Reference](https://tauri.app/v2/api/)

### React/TypeScript

- [React Documentation](https://react.dev/learn)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [Chakra UI Documentation](https://chakra-ui.com/docs/getting-started)