#!/bin/bash
# NextDownloader ビルドスクリプト
# プロジェクト全体をビルドする統合スクリプト

set -e

# プロジェクトルートディレクトリ
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
echo "プロジェクトルート: $PROJECT_ROOT"

# 引数の解析
CLEAN=false
RELEASE=false
TARGET_PLATFORM="macos"

for arg in "$@"; do
  case $arg in
    --clean)
      CLEAN=true
      shift
      ;;
    --release)
      RELEASE=true
      shift
      ;;
    --platform=*)
      TARGET_PLATFORM="${arg#*=}"
      shift
      ;;
    *)
      # 不明な引数
      ;;
  esac
done

# ビルドタイプ
BUILD_TYPE="debug"
if [ "$RELEASE" = true ]; then
  BUILD_TYPE="release"
fi

echo "ビルドタイプ: $BUILD_TYPE"
echo "ターゲットプラットフォーム: $TARGET_PLATFORM"

# クリーンビルドの場合
if [ "$CLEAN" = true ]; then
  echo "クリーンビルドを実行します..."
  # Cargoのクリーン
  cargo clean
  # Flutterのクリーン
  cd "$PROJECT_ROOT/flutter_app"
  flutter clean
  cd "$PROJECT_ROOT"
fi

# Rustライブラリのビルド
echo "Rustライブラリをビルドしています..."
if [ "$RELEASE" = true ]; then
  cargo build --release -p next_downloader_ffi
else
  cargo build -p next_downloader_ffi
fi

# Flutterの依存関係を取得
echo "Flutterの依存関係を取得しています..."
cd "$PROJECT_ROOT/flutter_app"
flutter pub get

# macOS固有のビルド処理
if [ "$TARGET_PLATFORM" = "macos" ]; then
  echo "macOS向けのライブラリをコピーしています..."
  bash "$PROJECT_ROOT/flutter_app/macos/rust_build.sh"
fi

# Flutterアプリのビルド
echo "Flutterアプリをビルドしています..."
if [ "$RELEASE" = true ]; then
  flutter build "$TARGET_PLATFORM" --release
else
  flutter build "$TARGET_PLATFORM" --debug
fi

echo "ビルドが完了しました！"

# macOSの場合、アプリケーションの場所を表示
if [ "$TARGET_PLATFORM" = "macos" ]; then
  if [ "$RELEASE" = true ]; then
    APP_PATH="$PROJECT_ROOT/flutter_app/build/macos/Build/Products/Release/NextDownloader.app"
  else
    APP_PATH="$PROJECT_ROOT/flutter_app/build/macos/Build/Products/Debug/NextDownloader.app"
  fi
  
  echo "アプリケーションパス: $APP_PATH"
  echo "アプリケーションを実行するには: open $APP_PATH"
fi 