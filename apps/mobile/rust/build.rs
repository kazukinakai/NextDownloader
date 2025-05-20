fn main() {
    // ビルド時の設定
    // 既存のNextDownloaderコアとの連携は一時的に無効化
    // println!("cargo:rustc-link-lib=static=nextdownloader_core");
    // println!("cargo:rustc-link-search=../../../core/target/release");
    
    // ファイル変更時に再ビルドする設定
    println!("cargo:rerun-if-changed=src/api.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
}