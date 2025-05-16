// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use nextdownloader_core::{DownloadManager, Downloader};

// Tauriアプリケーションのエントリーポイント
fn main() {
    // アプリケーションの初期化
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            // バックグラウンドで依存関係チェック
            tokio::spawn(async move {
                let downloader = DownloadManager::new();
                let status = downloader.system_status().await;
                if !status.is_ready() {
                    // 依存関係不足の警告ダイアログ
                    let _ = app_handle.dialog()
                        .message(format!(
                            "NextDownloaderの実行に必要なツールが不足しています。\n\n{}\n\n必要なツールをインストールしてください。",
                            status.description()
                        ))
                        .title("依存関係エラー")
                        .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                        .blocking_show();
                }
            });
            Ok(())
        })
        // コマンドハンドラー登録
        .invoke_handler(tauri::generate_handler![
            commands::download_url,
            commands::detect_content_type,
            commands::check_system_status
        ])
        .run(tauri::generate_context!())
        .expect("アプリケーションの起動中にエラーが発生しました");
}
