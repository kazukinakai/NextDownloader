#!/bin/bash
# NextDownloader ビルドスクリプト
# プロジェクト全体をビルドする統合スクリプト

set -e

# 色付き出力用の関数
print_info() {
  echo -e "\033[0;34m[INFO] $1\033[0m"
}

print_success() {
  echo -e "\033[0;32m[SUCCESS] $1\033[0m"
}

print_error() {
  echo -e "\033[0;31m[ERROR] $1\033[0m"
}

print_warning() {
  echo -e "\033[0;33m[WARNING] $1\033[0m"
}

check_dependencies() {
  print_info "依存関係を確認しています..."
  
  # yt-dlpの確認
  if command -v yt-dlp >/dev/null 2>&1; then
    print_success "yt-dlp が見つかりました: $(which yt-dlp)"
    YTDLP_VERSION=$(yt-dlp --version)
    print_info "yt-dlp バージョン: $YTDLP_VERSION"
  else
    print_warning "yt-dlp が見つかりません。インストールすることをお勧めします。"
    print_info "インストール方法: pip install yt-dlp"
  fi
  
  # aria2cの確認
  if command -v aria2c >/dev/null 2>&1; then
    print_success "aria2c が見つかりました: $(which aria2c)"
    ARIA2C_VERSION=$(aria2c --version | head -n 1)
    print_info "aria2c バージョン: $ARIA2C_VERSION"
  else
    print_warning "aria2c が見つかりません。インストールすることをお勧めします。"
    print_info "インストール方法: brew install aria2"
  fi
  
  # FFmpegの確認
  if command -v ffmpeg >/dev/null 2>&1; then
    print_success "FFmpeg が見つかりました: $(which ffmpeg)"
    FFMPEG_VERSION=$(ffmpeg -version | head -n 1)
    print_info "FFmpeg バージョン: $FFMPEG_VERSION"
  else
    print_warning "FFmpeg が見つかりません。インストールすることをお勧めします。"
    print_info "インストール方法: brew install ffmpeg"
  fi
}

# プロジェクトルートディレクトリ
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
print_info "プロジェクトルート: $PROJECT_ROOT"

# 引数の解析
CLEAN=false
RELEASE=false
TARGET_PLATFORM="macos"
FEATURES="external-tools"
VERBOSE=false
INSTALL_DEPS=false

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
    --features=*)
      FEATURES="${arg#*=}"
      shift
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    --install-deps)
      INSTALL_DEPS=true
      shift
      ;;
    --help)
      echo "使用法: $0 [オプション]"
      echo "オプション:"
      echo "  --clean              クリーンビルドを実行する"
      echo "  --release            リリースビルドを実行する"
      echo "  --platform=PLATFORM  ターゲットプラットフォームを指定する (デフォルト: macos)"
      echo "  --features=FEATURES  Rustの機能フラグを指定する (デフォルト: external-tools)"
      echo "                       有効な値: external-tools"
      echo "  --verbose            詳細な出力を表示する"
      echo "  --install-deps       依存関係を自動的にインストールする"
      echo "  --help               このヘルプメッセージを表示する"
      exit 0
      ;;
    *)
      # 不明な引数
      print_warning "不明な引数: $arg (無視されます)"
      ;;
  esac
done

# 依存関係のインストール
if [ "$INSTALL_DEPS" = true ]; then
  print_info "依存関係をインストールしています..."
  
  # macOSの場合
  if [ "$(uname)" = "Darwin" ]; then
    # Homebrewのインストール確認
    if ! command -v brew >/dev/null 2>&1; then
      print_error "Homebrewがインストールされていません。インストールしてください。"
      print_info "インストール方法: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
      exit 1
    fi
    
    # Pythonのインストール（yt-dlp用）
    if ! command -v python3 >/dev/null 2>&1; then
      print_info "Pythonをインストールしています..."
      brew install python
    fi
    
    # yt-dlpのインストール
    if ! command -v yt-dlp >/dev/null 2>&1; then
      print_info "yt-dlpをインストールしています..."
      pip3 install yt-dlp
    fi
    
    # aria2cのインストール
    if ! command -v aria2c >/dev/null 2>&1; then
      print_info "aria2cをインストールしています..."
      brew install aria2
    fi
    
    # FFmpegのインストール
    if ! command -v ffmpeg >/dev/null 2>&1; then
      print_info "FFmpegをインストールしています..."
      brew install ffmpeg
    fi
  else
    print_error "このスクリプトはmacOSでのみ依存関係の自動インストールをサポートしています。"
    print_info "手動でyt-dlp、aria2c、FFmpegをインストールしてください。"
    exit 1
  fi
  
  check_dependencies
  exit 0
fi

# ビルドタイプ
BUILD_TYPE="debug"
if [ "$RELEASE" = true ]; then
  BUILD_TYPE="release"
fi

print_info "ビルドタイプ: $BUILD_TYPE"
print_info "ターゲットプラットフォーム: $TARGET_PLATFORM"
print_info "機能フラグ: $FEATURES"

# クリーンビルドの場合
if [ "$CLEAN" = true ]; then
  print_info "クリーンビルドを実行します..."
  # Cargoのクリーン
  cargo clean
  rm -f Cargo.lock
  # Flutterのクリーン
  cd "$PROJECT_ROOT/flutter_app"
  flutter clean
  cd "$PROJECT_ROOT"
fi

# 依存関係の確認
check_dependencies

# Rustライブラリのビルド
print_info "Rustライブラリをビルドしています..."

# ビルドコマンドの構築
CARGO_CMD="cargo build"
if [ "$RELEASE" = true ]; then
  CARGO_CMD="$CARGO_CMD --release"
fi

if [ -n "$FEATURES" ]; then
  CARGO_CMD="$CARGO_CMD --features $FEATURES"
fi

if [ "$VERBOSE" = true ]; then
  CARGO_CMD="$CARGO_CMD --verbose"
fi

# Apple Silicon (M1/M2)チェック
ARCH="$(uname -m)"
if [ "$ARCH" = "arm64" ]; then
  print_info "Apple Silicon (ARM64)アーキテクチャを検出しました"
  # ARM64用にビルド
  CARGO_CMD="$CARGO_CMD --target aarch64-apple-darwin"
else
  print_info "Intel (x86_64)アーキテクチャを検出しました"
  # x86_64用にビルド
  CARGO_CMD="$CARGO_CMD --target x86_64-apple-darwin"
fi

# Cargoビルドの実行
eval "$CARGO_CMD"

if [ $? -ne 0 ]; then
  print_error "Rustライブラリのビルドに失敗しました。"
  exit 1
fi

print_success "Rustライブラリのビルドが完了しました。"

# Flutterアプリのビルド
print_info "Flutterアプリをビルドしています..."

cd "$PROJECT_ROOT/flutter_app"

# Flutterパッケージの更新
flutter pub get

if [ $? -ne 0 ]; then
  print_error "Flutterパッケージの取得に失敗しました。"
  exit 1
fi

# Flutterアプリのビルド
FLUTTER_CMD="flutter build $TARGET_PLATFORM"
if [ "$RELEASE" = true ]; then
  FLUTTER_CMD="$FLUTTER_CMD --release"
else
  FLUTTER_CMD="$FLUTTER_CMD --debug"
fi

if [ "$VERBOSE" = true ]; then
  FLUTTER_CMD="$FLUTTER_CMD -v"
fi

# Flutterビルドの実行
print_info "コマンド実行: $FLUTTER_CMD"
eval "$FLUTTER_CMD"

if [ $? -ne 0 ]; then
  print_error "Flutterアプリのビルドに失敗しました。"
  exit 1
fi

print_success "Flutterアプリのビルドが完了しました。"

# ビルド完了後のパス情報
if [ "$TARGET_PLATFORM" = "macos" ]; then
  if [ "$RELEASE" = true ]; then
    APP_PATH="$PROJECT_ROOT/flutter_app/build/macos/Build/Products/Release/NextDownloader.app"
  else
    APP_PATH="$PROJECT_ROOT/flutter_app/build/macos/Build/Products/Debug/NextDownloader.app"
  fi
  
  if [ -d "$APP_PATH" ]; then
    print_success "アプリケーションのビルドが完了しました: $APP_PATH"
    print_info "アプリを実行するには: open \"$APP_PATH\""
  else
    print_warning "ビルドは完了しましたが、アプリケーションが予期された場所に見つかりません。"
  fi
fi

print_success "NextDownloaderのビルドが完了しました！" 