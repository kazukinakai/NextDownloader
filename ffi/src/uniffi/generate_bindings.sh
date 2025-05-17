#!/bin/bash

# UniFFIバインディングを生成するスクリプト

# ビルドディレクトリ
BUILD_DIR="../../build/uniffi"
mkdir -p $BUILD_DIR

# Swiftバインディングの生成
echo "Generating Swift bindings..."
cargo run --bin uniffi-bindgen generate src/nextdownloader.udl --language swift --out-dir $BUILD_DIR

# Kotlinバインディングの生成
echo "Generating Kotlin bindings..."
cargo run --bin uniffi-bindgen generate src/nextdownloader.udl --language kotlin --out-dir $BUILD_DIR

# Pythonバインディングの生成（オプション）
echo "Generating Python bindings..."
cargo run --bin uniffi-bindgen generate src/nextdownloader.udl --language python --out-dir $BUILD_DIR

# JavaScriptバインディングの生成（オプション）
echo "Generating JavaScript bindings..."
cargo run --bin uniffi-bindgen generate src/nextdownloader.udl --language javascript --out-dir $BUILD_DIR

echo "Bindings generated successfully in $BUILD_DIR"