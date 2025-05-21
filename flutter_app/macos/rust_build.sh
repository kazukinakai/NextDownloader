#!/bin/bash
# rust_build.sh
# Rustライブラリをビルドし、Flutterアプリケーションにリンクするスクリプト

set -e

# プロジェクトのルートディレクトリを取得
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
FFI_DIR="$PROJECT_ROOT/ffi"
TARGET_DIR="$PROJECT_ROOT/target"  # ルートのtargetディレクトリを使用
LIBS_DIR="$PROJECT_ROOT/flutter_app/macos/Libs"
FRAMEWORKS_DIR="$PROJECT_ROOT/flutter_app/build/macos/Build/Products/Debug/NextDownloader.app/Contents/Frameworks"

# CPUアーキテクチャを検出
ARCH="$(uname -m)"
if [ "$ARCH" = "arm64" ]; then
  # Apple Silicon (M1/M2)
  echo "Apple Silicon (ARM64)アーキテクチャを検出しました"
  # ユニバーサルバイナリをビルドするためのターゲット
  RUST_TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")
  PRIMARY_TARGET="aarch64-apple-darwin"
  BUILD_UNIVERSAL=true
else
  # Intel Mac
  echo "Intel (x86_64)アーキテクチャを検出しました"
  RUST_TARGETS=("x86_64-apple-darwin")
  PRIMARY_TARGET="x86_64-apple-darwin"
  BUILD_UNIVERSAL=false
fi

LIB_NAME="next_downloader_ffi"
DYLIB_NAME="lib$LIB_NAME.dylib"

echo "NextDownloader Rustライブラリビルドスクリプト"
echo "プロジェクトルート: $PROJECT_ROOT"
echo "FFIディレクトリ: $FFI_DIR"
echo "ターゲットディレクトリ: $TARGET_DIR"
echo "ライブラリディレクトリ: $LIBS_DIR"
echo "フレームワークディレクトリ: $FRAMEWORKS_DIR"

# Libsディレクトリが存在しない場合は作成
mkdir -p "$LIBS_DIR"
mkdir -p "$FRAMEWORKS_DIR"

# Homebrewでインストールされた場合のパスを追加
export PATH="/opt/homebrew/bin:$PATH"

# ワークスペースのルートディレクトリでcargoを実行
cd "$PROJECT_ROOT"
echo "Rustワークスペースのルートディレクトリでのビルドを実行します..."

# 各ターゲット用にRustライブラリをビルド
for TARGET in "${RUST_TARGETS[@]}"; do
  echo "Rustライブラリを $TARGET 用にビルドしています..."
  # ターゲットがインストールされているか確認
  if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
      echo "$TARGET ターゲットをインストールしています..."
      rustup target add "$TARGET" || {
        echo "警告: rustupターゲット追加に失敗しました。標準ターゲットでビルドを試みます。"
      }
    fi
    # rustupが利用可能な場合はターゲットを指定してビルド
    cargo build --release -p next_downloader_ffi --target "$TARGET" || {
      echo "警告: ターゲット指定ビルドに失敗しました。標準ターゲットでビルドを試みます..."
      cargo build --release -p next_downloader_ffi
      cp "$TARGET_DIR/release/$DYLIB_NAME" "$LIBS_DIR/"
      BUILD_UNIVERSAL=false
      break
    }
  else
    echo "rustupが見つかりません。標準ターゲットでビルドを試みます..."
    cargo build --release -p next_downloader_ffi
    cp "$TARGET_DIR/release/$DYLIB_NAME" "$LIBS_DIR/"
    BUILD_UNIVERSAL=false
    break
  fi
done

# ユニバーサルバイナリを作成（Apple Siliconの場合）
if [ "$BUILD_UNIVERSAL" = true ] && [ ${#RUST_TARGETS[@]} -gt 1 ]; then
  echo "ユニバーサルバイナリを作成しています..."
  TEMP_DIR="$LIBS_DIR/temp"
  mkdir -p "$TEMP_DIR"
  
  # 各アーキテクチャのライブラリをコピー
  for TARGET in "${RUST_TARGETS[@]}"; do
    cp "$TARGET_DIR/$TARGET/release/$DYLIB_NAME" "$TEMP_DIR/$DYLIB_NAME.$TARGET"
  done
  
  # lipoを使ってユニバーサルバイナリを作成
  lipo -create -output "$LIBS_DIR/$DYLIB_NAME" "$TEMP_DIR/$DYLIB_NAME.aarch64-apple-darwin" "$TEMP_DIR/$DYLIB_NAME.x86_64-apple-darwin"
  
  # 一時ディレクトリを削除
  rm -rf "$TEMP_DIR"
else
  # 単一アーキテクチャの場合は直接コピー
  echo "ビルドされたライブラリをコピーしています..."
  if [ -f "$TARGET_DIR/$PRIMARY_TARGET/release/$DYLIB_NAME" ]; then
    cp "$TARGET_DIR/$PRIMARY_TARGET/release/$DYLIB_NAME" "$LIBS_DIR/"
  elif [ -f "$TARGET_DIR/release/$DYLIB_NAME" ]; then
    cp "$TARGET_DIR/release/$DYLIB_NAME" "$LIBS_DIR/"
  else
    echo "エラー: ビルドされたライブラリが見つかりません"
    exit 1
  fi
fi

# インストール名ツールでライブラリのインストール名を変更
echo "ライブラリのインストール名を変更しています..."
install_name_tool -id "@executable_path/../Frameworks/$DYLIB_NAME" "$LIBS_DIR/$DYLIB_NAME"

# フレームワークディレクトリにライブラリをコピー
echo "ライブラリをフレームワークディレクトリにコピーしています..."
cp "$LIBS_DIR/$DYLIB_NAME" "$FRAMEWORKS_DIR/"

echo "ビルドが完了しました: $LIBS_DIR/$DYLIB_NAME"
echo "フレームワークにコピーしました: $FRAMEWORKS_DIR/$DYLIB_NAME"