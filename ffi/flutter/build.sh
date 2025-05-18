#!/bin/bash
set -e

# ディレクトリ設定
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
FLUTTER_APP_DIR="$SCRIPT_DIR/../../apps/flutter/next_downloader"
RUST_INPUT="$SCRIPT_DIR/src/lib.rs"
DART_OUTPUT="$FLUTTER_APP_DIR/lib/ffi/bridge_generated.dart"
RUST_OUTPUT="$SCRIPT_DIR/src/bridge_generated.rs"
DART_CLASS_NAME="DownloaderFFI"

# 必要なディレクトリを作成
mkdir -p "$FLUTTER_APP_DIR/lib/ffi"

# flutter_rust_bridge_codegen を使用してバインディングを生成
flutter_rust_bridge_codegen \
  --rust-input "$RUST_INPUT" \
  --dart-output "$DART_OUTPUT" \
  --class-name "$DART_CLASS_NAME" \
  --rust-output "$RUST_OUTPUT"

echo "FFI バインディングの生成が完了しました。"

# プラットフォーム固有のビルドスクリプトを実行
case "$(uname -s)" in
  Darwin)
    # macOS/iOS 向けビルド
    echo "macOS/iOS 向けビルドを実行します..."
    # TODO: XCFramework のビルドスクリプトを追加
    ;;
  Linux)
    # Linux 向けビルド
    echo "Linux 向けビルドを実行します..."
    # TODO: Linux 向けビルドスクリプトを追加
    ;;
  MINGW*|MSYS*|CYGWIN*)
    # Windows 向けビルド
    echo "Windows 向けビルドを実行します..."
    # TODO: Windows 向けビルドスクリプトを追加
    ;;
  *)
    echo "未対応のプラットフォームです: $(uname -s)"
    exit 1
    ;;
esac

echo "ビルドが完了しました。"