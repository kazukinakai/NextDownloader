# NextDownloader 知識ベース

## 技術スタックの最新情報

### Swift/SwiftUI
- **Swift バージョン**: 5.9+
- **SwiftUI**: iOS 17/macOS 14 以降の機能を活用
- **最新の変更点**:
  - Swift Concurrency の完全対応
  - SwiftUI の新しいライフサイクル管理
  - マクロを活用した開発効率の向上

### Rust
- **Rust バージョン**: 最新安定版（2023年以降）
- **主要クレート**:
  - tokio: 非同期ランタイム
  - reqwest: HTTP クライアント
  - serde: シリアライゼーション/デシリアライゼーション
  - clap: コマンドライン引数パーサー
  - tauri: GUI フレームワーク（将来的なクロスプラットフォーム対応）

### 外部ツール
- **yt-dlp**: YouTube-DL の改良版、より多くのサイトに対応し、積極的に開発されている
- **aria2c**: 高速並列ダウンロードツール、HTTP/HTTPS/FTP/BitTorrent に対応
- **ffmpeg**: 動画処理の業界標準ツール、トランスコードや結合に使用

## 発見したベストプラクティス

### アーキテクチャ設計
- **クリーンアーキテクチャ**:
  - ドメイン層、アプリケーション層、インフラストラクチャ層の明確な分離
  - 依存関係の方向は内側に向ける（依存性逆転の原則）
  - インターフェースを活用した疎結合設計

- **MVVM パターン**:
  - SwiftUI と相性の良い MVVM アーキテクチャの採用
  - ビューとビジネスロジックの分離
  - テスト可能性の向上

### Swift/SwiftUI 開発
- **SwiftUI のパフォーマンス最適化**:
  - 不必要な再描画を避けるための `@State` と `@Binding` の適切な使用
  - 大きなビューの分割と再利用
  - `LazyVStack`/`LazyHStack` の活用

- **Swift Concurrency**:
  - `async/await` パターンの活用
  - `Task` と `TaskGroup` による並行処理
  - アクターモデルによる安全な状態管理

### Rust 開発
- **エラー処理**:
  - `thiserror` と `anyhow` クレートの活用
  - 適切なエラー型の定義と伝播

- **非同期処理**:
  - tokio エコシステムの活用
  - Future と Stream の適切な使用
  - 非同期コンテキストの伝播

### 外部ツール連携
- **プロセス管理**:
  - 非同期プロセス実行
  - 標準出力/エラー出力のストリーミング処理
  - シグナルハンドリングによる安全な終了

- **バイナリ管理**:
  - アプリケーションバンドル内への埋め込み
  - 自動更新機構
  - バージョン互換性の確保

## トラブルシューティングと解決策

### HLS/DASH ダウンロード
- **問題**: セグメントの一部がダウンロードできない
  - **解決策**: User-Agent の偽装、リトライ機構の実装、複数 CDN からの取得

- **問題**: DRM 保護コンテンツへのアクセス
  - **解決策**: ブラウザセッションの活用、認証情報の適切な伝達

### 外部ツール連携
- **問題**: 外部ツールの実行権限
  - **解決策**: macOS の公証とセキュリティ対策、適切な権限要求

- **問題**: 外部ツールの出力解析
  - **解決策**: 構造化されたログ形式の活用、正規表現による解析

### パフォーマンス
- **問題**: 大量のセグメントダウンロード時のメモリ使用量
  - **解決策**: ストリーミング処理、一時ファイルの活用

- **問題**: UI の応答性低下
  - **解決策**: バックグラウンドスレッドでの処理、進捗更新の最適化

## 重要な設計判断とその理由

### Rust コアの採用
- **判断**: ダウンロードエンジンのコア部分を Rust で実装
- **理由**:
  - パフォーマンスと安全性の両立
  - クロスプラットフォーム対応の容易さ
  - メモリ効率の高い並行処理

### Chrome 拡張と Native アプリの連携
- **判断**: Native Messaging API を使用した連携
- **理由**:
  - ブラウザのセキュリティモデルとの互換性
  - 安定した双方向通信の実現
  - ユーザー体験の向上

### 外部ツールの統合
- **判断**: 既存の高性能ツール（yt-dlp, aria2c, ffmpeg）の活用
- **理由**:
  - 車輪の再発明を避ける
  - 広範なサイト対応と継続的な更新
  - 専門的な機能の活用

## 学習リソースと参考資料

### Swift/SwiftUI
- [Apple Developer Documentation](https://developer.apple.com/documentation/)
- [Swift.org](https://swift.org/)
- [Hacking with Swift](https://www.hackingwithswift.com/)
- [Swift by Sundell](https://www.swiftbysundell.com/)

### Rust
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)

### HLS/DASH ストリーミング
- [HLS Specification](https://datatracker.ietf.org/doc/html/rfc8216)
- [DASH Industry Forum](https://dashif.org/)
- [FFmpeg Documentation](https://ffmpeg.org/documentation.html)

### Chrome 拡張開発
- [Chrome Extensions Documentation](https://developer.chrome.com/docs/extensions/)
- [Native Messaging API](https://developer.chrome.com/docs/extensions/mv3/nativeMessaging/)

### 並列ダウンロード
- [aria2 Documentation](https://aria2.github.io/)
- [HTTP Range Requests](https://datatracker.ietf.org/doc/html/rfc7233)
