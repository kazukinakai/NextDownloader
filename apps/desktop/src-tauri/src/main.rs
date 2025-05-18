//! # NextDownloader デスクトップアプリケーション
//! 
//! Tauri 2.0を使用したクロスプラットフォームデスクトップアプリケーションです。

use std::sync::Arc;

use log::{debug, error, info, warn};
use tauri::Manager;

mod commands;
mod state;

fn main() {
    // ロギングの初期化
    env_logger::init();
    info!("NextDownloader デスクトップアプリケーションを起動しています...");
    
    // NextDownloader FFIの初期化
    nextdownloader_ffi::initialize();
    
    // Tauriアプリケーションの構築と実行
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::init())
        .plugin(tauri_plugin_window::init())
        .setup(|app| {
            // アプリケーションの状態を初期化
            let app_state = state::AppState::new();
            app.manage(app_state);
            
            // 開発モードの場合はDevtoolsを開く
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            
            Ok(())
        })
        // コマンドの登録
        .invoke_handler(tauri::generate_handler![
            commands::download::start_download,
            commands::download::get_download_progress,
            commands::download::cancel_download,
            commands::download::get_downloads,
            commands::dependency::check_dependencies,
            commands::config::get_config,
            commands::config::save_config,
            commands::utils::select_directory,
            commands::utils::get_version,
        ])
        .run(tauri::generate_context!())
        .expect("Tauriアプリケーションの実行中にエラーが発生しました");
}