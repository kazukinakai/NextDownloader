// Tauriアプリケーションのメインエントリーポイント
// NextDownloader - 高性能ダウンローダーアプリケーション

// 必要なクレートをインポート
use tauri::Manager;
use log::{info, error};

// アプリケーションの状態
#[derive(Default)]
struct AppState {
    config_path: std::path::PathBuf,
}

fn main() {
    // ロギングの初期化
    env_logger::init();
    info!("NextDownloader Tauriアプリケーションを起動しています...");

    // Tauriアプリケーションの構築と実行
    tauri::Builder::default()
        // NextDownloaderプラグインを登録
        .plugin(nextdownloader_plugin_download::init())
        // アプリケーションのセットアップ
        .setup(|app| {
            // アプリケーションの状態を初期化
            let app_handle = app.handle();
            let config_dir = app_handle.path().app_config_dir().unwrap_or_default();
            
            // 設定ディレクトリが存在しない場合は作成
            if !config_dir.exists() {
                std::fs::create_dir_all(&config_dir).map_err(|e| {
                    error!("設定ディレクトリの作成に失敗しました: {:?}", e);
                    e
                })?;
            }
            
            // アプリケーションの状態を管理
            app.manage(AppState {
                config_path: config_dir,
            });
            
            info!("NextDownloaderアプリケーションのセットアップが完了しました");
            Ok(())
        })
        // アプリケーションのイベントハンドラ
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    info!("ウィンドウが閉じられようとしています");
                    // ここでクリーンアップ処理を行うことができます
                },
                _ => {}
            }
        })
        // Tauriアプリケーションの実行
        .run(tauri::generate_context!())
        .expect("Tauriアプリケーションの実行中にエラーが発生しました");
}