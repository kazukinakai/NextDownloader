# NextDownloader 知識ベース

## 技術スタックの最新情報

### Tauri 2.0（2025年5月更新）

Tauri 2.0は、Rustベースのクロスプラットフォームアプリケーションフレームワークで、デスクトップだけでなくiOSとAndroidにも対応しています。

#### 主な特徴

- **クロスプラットフォーム**: デスクトップ（Windows, macOS, Linux）とモバイル（iOS, Android）に対応
- **軽量**: Electronと比較して大幅に小さいバイナリサイズ
- **セキュア**: セキュリティを重視した設計
- **高性能**: Rustで実装されたバックエンドによる高いパフォーマンス
- **ネイティブUI**: OSネイティブのWebViewを使用

#### アーキテクチャ

- **コアエンジン**: Rustで実装されたバックエンド
- **WebView**: OSネイティブのWebView（WKWebView, WebView2, WebKitGTK）
- **IPC**: フロントエンドとバックエンド間の通信メカニズム
- **プラグインシステム**: 機能拡張のためのプラグイン機構

#### モバイル対応

Tauri 2.0では、iOS/Androidのサポートが追加されました。

- **iOS**: WKWebViewを使用
- **Android**: WebViewを使用
- **共通API**: デスクトップとモバイルで同じAPIを使用可能

#### プラグインシステム

Tauri 2.0では、プラグインシステムが強化され、より柔軟な機能拡張が可能になりました。

```rust
// プラグインの定義
pub struct MyPlugin<R: Runtime> {
    // プラグインの状態
}

// プラグインの初期化
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("my-plugin")
        .setup(|app| {
            // セットアップ処理
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            my_command
        ])
        .build()
}

// コマンドの実装
#[tauri::command]
async fn my_command() -> Result<String, String> {
    // コマンドの処理
    Ok("Hello from plugin".into())
}
```

### Rust-FFI連携（2025年5月更新）

#### UniFFIの活用

UniFFIは、Rustコードから複数の言語（Swift、Kotlin、Python、JavaScript）へのバインディングを自動生成するツールです。従来のC FFIと比較して、以下の利点があります：

- 型安全性の向上
- 複数言語への対応が容易
- コードの保守性の向上
- エラーハンドリングの改善

#### 実装方法

1. UDLファイル（`.udl`）でインターフェースを定義
2. Rustでインターフェースを実装
3. バインディングを自動生成
4. 各言語から生成されたバインディングを使用

#### 使用例（Swift）

```swift
import NextDownloaderFFI

let downloadManager = DownloadManager()
let dependencies = try downloadManager.checkDependencies()

if dependencies.ytdlp && dependencies.aria2c && dependencies.ffmpeg {
    print("All dependencies are available")
} else {
    print("Some dependencies are missing")
}

let contentType = try downloadManager.detectContentType(url: "https://example.com/video.mp4")
```

#### 使用例（Kotlin）

```kotlin
import com.nextdownloader.ffi.*

val downloadManager = DownloadManager()
val dependencies = downloadManager.checkDependencies()

if (dependencies.ytdlp && dependencies.aria2c && dependencies.ffmpeg) {
    println("All dependencies are available")
} else {
    println("Some dependencies are missing")
}

val contentType = downloadManager.detectContentType("https://example.com/video.mp4")
```

## 発見したベストプラクティス

### モバイル対応のベストプラクティス

1. **レスポンシブデザイン**: 様々な画面サイズに対応するUIデザイン
2. **プラットフォーム固有の最適化**: 各OSの機能を活用
3. **権限管理**: 必要な権限を適切にリクエスト
4. **オフライン対応**: ネットワーク接続が不安定な環境でも動作するよう設計
5. **バッテリー消費の最適化**: バックグラウンド処理の最適化

### FFIレイヤーの設計

1. **関心事の分離**: コア機能とFFIレイヤーを明確に分離する
2. **型安全性**: 言語間の型変換を明示的に行い、エラーを防ぐ
3. **非同期処理**: 非同期関数を同期的に呼び出す場合は、専用のランタイムを用意する
4. **エラーハンドリング**: エラーを適切に変換し、呼び出し側に伝える

### モノレポ構成

1. **ワークスペースの活用**: Cargo Workspacesを使用して依存関係を一元管理
2. **独立したクレート**: 機能ごとに独立したクレートとして実装
3. **バージョン管理**: 各クレートを独立してバージョン管理

## トラブルシューティングと解決策

### Tauriのモバイルビルドエラー

問題: iOS/Androidビルド時のエラー

解決策:
```bash
# iOSビルドの前提条件
xcode-select --install
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim

# Androidビルドの前提条件
rustup target add armv7-linux-androideabi
rustup target add aarch64-linux-android
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

### UniFFIのビルドエラー

問題: `uniffi-bindgen`コマンドが見つからない

解決策:
```bash
cargo install uniffi-bindgen
```

### バインディング生成時のエラー

問題: UDLファイルのパスが見つからない

解決策:
```bash
# 正しいパスを指定
cargo run --bin uniffi-bindgen generate /path/to/your.udl --language swift
```

## 重要な設計判断とその理由

### Tauri 2.0の採用

- **理由**: クロスプラットフォーム対応（デスクトップ・モバイル）を単一のコードベースで実現するため
- **メリット**: 開発効率の向上、保守性の向上、一貫したユーザー体験
- **デメリット**: モバイル対応が発展途上、一部のネイティブ機能へのアクセスが制限される可能性

### UniFFIの採用

- **理由**: 複数言語へのバインディング生成を自動化し、保守性を向上させるため
- **メリット**: 型安全性の向上、コードの重複削減、エラーハンドリングの改善
- **デメリット**: 学習コストの増加、既存のC FFIからの移行コスト

### Tauriプラグインシステムの活用

- **理由**: 機能を独立したモジュールとして実装し、再利用性を高めるため
- **メリット**: コードの分離、テスト容易性の向上、機能の拡張性
- **デメリット**: 設計の複雑化、初期実装コストの増加

## 学習リソースと参考資料

- [Tauri 2.0公式ドキュメント](https://tauri.app/v2/docs/)
- [Tauri Mobile Guide](https://tauri.app/v2/guides/mobile/)
- [UniFFI公式ドキュメント](https://mozilla.github.io/uniffi-rs/)
- [Rustのクロスプラットフォーム開発ガイド](https://rust-lang.org/learn)
- [React + TypeScriptベストプラクティス](https://react-typescript-cheatsheet.netlify.app/)