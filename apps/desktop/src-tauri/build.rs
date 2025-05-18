// Tauriアプリケーションのビルドスクリプト
// NextDownloader - 高性能ダウンローダーアプリケーション

fn main() {
    // Tauriビルドプロセスの実行
    tauri_build::build();
    
    // ここに追加のビルド手順を記述できます
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=capabilities/");
}