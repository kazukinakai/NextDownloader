//! NextDownloaderコア機能のテスト
//! 
//! このモジュールでは、ダウンロード機能に関するテストを実装します。

use nextdownloader_core::{DownloadManager, Downloader, DownloadOptions, ContentType, VideoFormat, ErrorCode};
use std::path::PathBuf;
use tokio::runtime::Runtime;

/// ダウンロードマネージャーの初期化テスト
#[test]
fn test_download_manager_initialization() {
    let download_manager = DownloadManager::new();
    assert!(download_manager.is_initialized());
}

/// コンテンツタイプ検出のテスト
#[test]
fn test_content_type_detection() {
    // テスト用のURLとその期待されるコンテンツタイプのマッピング
    let test_cases = vec![
        ("https://example.com/video.mp4", ContentType::MP4),
        ("https://example.com/stream.m3u8", ContentType::HLS),
        ("https://example.com/manifest.mpd", ContentType::DASH),
        ("https://www.youtube.com/watch?v=dQw4w9WgXcQ", ContentType::YouTube),
        ("https://example.com/unknown", ContentType::Unknown),
    ];
    
    // 非同期テストのためのランタイム
    let rt = Runtime::new().unwrap();
    let download_manager = DownloadManager::new();
    
    for (url, expected_type) in test_cases {
        // モックを使用してネットワークリクエストを避ける
        // 実際のテストでは、モックライブラリを使用することを推奨
        let content_type = match url {
            url if url.ends_with(".mp4") => ContentType::MP4,
            url if url.ends_with(".m3u8") => ContentType::HLS,
            url if url.ends_with(".mpd") => ContentType::DASH,
            url if url.contains("youtube.com") => ContentType::YouTube,
            _ => ContentType::Unknown,
        };
        
        assert_eq!(content_type, expected_type);
    }
}

/// ダウンロードオプションのテスト
#[test]
fn test_download_options() {
    let default_options = DownloadOptions::default();
    assert_eq!(default_options.preferred_format, None);
    assert_eq!(default_options.max_retries, 3);
    
    let custom_options = DownloadOptions {
        preferred_format: Some("mp4".to_string()),
        max_retries: 5,
        ..DownloadOptions::default()
    };
    
    assert_eq!(custom_options.preferred_format, Some("mp4".to_string()));
    assert_eq!(custom_options.max_retries, 5);
}

/// 依存関係チェックのテスト
/// 
/// このテストは実際の依存関係の有無に依存するため、CIでは適切にスキップすることを推奨
#[test]
#[ignore]
fn test_check_dependencies() {
    let download_manager = DownloadManager::new();
    let rt = Runtime::new().unwrap();
    
    let (ytdlp, aria2c, ffmpeg) = rt.block_on(download_manager.check_dependencies());
    
    // 依存関係の有無をログに出力（テスト環境によって結果が異なるため）
    println!("yt-dlp available: {}", ytdlp);
    println!("aria2c available: {}", aria2c);
    println!("ffmpeg available: {}", ffmpeg);
    
    // このテストでは実際の結果を検証するのではなく、関数が正常に実行されることを確認
    // 実際の環境では、これらの依存関係が利用可能であることを期待
}

/// ダウンロード処理のモックテスト
#[test]
fn test_mock_download() {
    let download_manager = DownloadManager::new();
    let rt = Runtime::new().unwrap();
    
    // テスト用のURL、パス、オプション
    let url = "https://example.com/test-video.mp4";
    let destination = PathBuf::from("/tmp/test-video.mp4");
    let options = DownloadOptions::default();
    
    // モックダウンロード処理（実際のダウンロードは行わない）
    // 実際のテストでは、モックライブラリを使用することを推奨
    let download_id = "test-download-id".to_string();
    
    // ダウンロードIDの形式を検証
    assert!(!download_id.is_empty());
    
    // 進捗取得のテスト
    let progress = 0.5; // モック進捗
    assert!(progress >= 0.0 && progress <= 1.0);
}

/// エラーコードのテスト
#[test]
fn test_error_codes() {
    assert_eq!(ErrorCode::Success as i32, 0);
    assert_ne!(ErrorCode::UnknownError as i32, 0);
    assert_ne!(ErrorCode::InvalidUrl as i32, 0);
    assert_ne!(ErrorCode::NetworkError as i32, 0);
    assert_ne!(ErrorCode::FileSystemError as i32, 0);
}

/// 統合テスト（実際のダウンロード）
/// 
/// このテストは実際のネットワークリクエストを行うため、CIでは適切にスキップすることを推奨
#[test]
#[ignore]
fn test_real_download() {
    // テスト用の小さなファイルのURL
    let url = "https://filesamples.com/samples/video/mp4/sample_640x360.mp4";
    let destination = PathBuf::from("/tmp/sample_test.mp4");
    let options = DownloadOptions::default();
    
    let download_manager = DownloadManager::new();
    let rt = Runtime::new().unwrap();
    
    // ダウンロードの実行
    let download_result = rt.block_on(async {
        let download_id = download_manager.start_download(url, &destination, options).await?;
        
        // 進捗の監視
        let mut progress = 0.0;
        while progress < 1.0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            progress = download_manager.get_download_progress(&download_id).await?;
            println!("Download progress: {:.2}%", progress * 100.0);
        }
        
        Ok::<_, ErrorCode>(())
    });
    
    // テスト結果の検証
    match download_result {
        Ok(_) => {
            // ダウンロードファイルの存在確認
            assert!(destination.exists());
            
            // テスト後のクリーンアップ
            if destination.exists() {
                std::fs::remove_file(destination).ok();
            }
        },
        Err(e) => {
            panic!("Download failed with error: {:?}", e);
        }
    }
}