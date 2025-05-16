#!/bin/bash

# NextDownloader Rustコアビルドスクリプト
# このスクリプトは、RustコアをビルドしてmacOSアプリで使用できるようにします

# 現在のディレクトリを取得
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CORE_DIR="$SCRIPT_DIR/../core"
OUTPUT_DIR="$SCRIPT_DIR/NextDownloaderCore/lib"

# 出力ディレクトリの作成
mkdir -p "$OUTPUT_DIR"

echo "Rustコアのビルドを開始します..."

# Rustコアのビルド
cd "$CORE_DIR" || exit 1
cargo build --release --features ffi

# ビルド成果物のコピー
echo "ビルド成果物をコピーしています..."
cp "$CORE_DIR/target/release/libnextdownloader_core.a" "$OUTPUT_DIR/"
cp "$CORE_DIR/target/release/libnextdownloader_core.dylib" "$OUTPUT_DIR/"

echo "ヘッダーファイルの生成..."
# cbindgenがインストールされていない場合はインストール
if ! command -v cbindgen &> /dev/null; then
    echo "cbindgenをインストールしています..."
    cargo install cbindgen
fi

# ヘッダーファイルの生成
cd "$CORE_DIR" || exit 1
cbindgen --config cbindgen.toml --crate nextdownloader-core --output "$OUTPUT_DIR/nextdownloader_core.h"

echo "Rustコアのビルドが完了しました！"
