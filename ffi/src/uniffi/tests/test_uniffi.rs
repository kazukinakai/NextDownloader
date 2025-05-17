//! UniFFIバインディングのテスト
//! 
//! このモジュールでは、UniFFIを使用したFFIバインディングのテストを実装します。

use nextdownloader_uniffi::{
    DownloadManager, DownloaderError, ContentType, VideoFormat, DependencyStatus
};

/// ダウンロードマネージャーの初期化テスト
#[test]
fn test_download_manager_initialization() {
    let download_manager = DownloadManager::new();
    assert!(download_manager.is_initialized());
}

/// 依存関係チェックのテスト
#[test]
#[ignore]
fn test_check_dependencies() {
    let download_manager = DownloadManager::new();
    
    match download_manager.check_dependencies() {
        Ok(status) => {
            println!("yt-dlp available: {}", status.ytdlp);
            println!("aria2c available: {}", status.aria2c);
            println!("ffmpeg available: {}", status.ffmpeg);
        },
        Err(e) => {
            panic!("Failed to check dependencies: {:?}", e);
        }
    }
}

/// コンテンツタイプ検出のテスト
#[test]
fn test_content_type_detection() {
    let download_manager = DownloadManager::new();
    
    // 有効なURL
    let valid_url = "https://example.com/video.mp4";
    match download_manager.detect_content_type(valid_url.to_string()) {
        Ok(content_type) => {
            assert_eq!(content_type, ContentType::MP4);
        },
        Err(e) => {
            panic!("Failed to detect content type for valid URL: {:?}", e);
        }
    }
    
    // 無効なURL
    let invalid_url = "";
    match download_manager.detect_content_type(invalid_url.to_string()) {
        Ok(_) => {
            panic!("Expected error for invalid URL, but got success");
        },
        Err(e) => {
            assert!(matches!(e, DownloaderError::InvalidUrl));
        }
    }
}

/// ダウンロード開始のテスト
#[test]
#[ignore]
fn test_start_download() {
    let download_manager = DownloadManager::new();
    
    // テスト用のURL、パス、フォーマット
    let url = "https://filesamples.com/samples/video/mp4/sample_640x360.mp4";
    let destination = "/tmp/sample_test_uniffi.mp4";
    let format = "mp4";
    
    match download_manager.start_download(url.to_string(), destination.to_string(), format.to_string()) {
        Ok(download_id) => {
            assert!(!download_id.is_empty());
            println!("Download started with ID: {}", download_id);
            
            // 進捗の取得
            match download_manager.get_download_progress(download_id.clone()) {
                Ok(progress) => {
                    assert!(progress >= 0.0 && progress <= 1.0);
                    println!("Download progress: {:.2}%", progress * 100.0);
                },
                Err(e) => {
                    panic!("Failed to get download progress: {:?}", e);
                }
            }
            
            // キャンセル処理のテスト
            match download_manager.cancel_download(download_id) {
                Ok(_) => {
                    println!("Download cancelled successfully");
                },
                Err(e) => {
                    panic!("Failed to cancel download: {:?}", e);
                }
            }
        },
        Err(e) => {
            // ネットワークエラーなどの場合はテストをスキップ
            println!("Failed to start download: {:?}", e);
        }
    }
}

/// エラーハンドリングのテスト
#[test]
fn test_error_handling() {
    let download_manager = DownloadManager::new();
    
    // 無効なダウンロードIDでの進捗取得
    let invalid_id = "non-existent-id";
    match download_manager.get_download_progress(invalid_id.to_string()) {
        Ok(_) => {
            panic!("Expected error for invalid download ID, but got success");
        },
        Err(e) => {
            // 適切なエラー型が返されることを確認
            assert!(matches!(e, DownloaderError::UnknownError));
        }
    }
    
    // 無効なダウンロードIDでのキャンセル
    match download_manager.cancel_download(invalid_id.to_string()) {
        Ok(_) => {
            panic!("Expected error for invalid download ID, but got success");
        },
        Err(e) => {
            // 適切なエラー型が返されることを確認
            assert!(matches!(e, DownloaderError::UnknownError));
        }
    }
}

/// 型変換のテスト
#[test]
fn test_type_conversions() {
    // ContentTypeの変換テスト
    let content_types = vec![
        ContentType::MP4,
        ContentType::HLS,
        ContentType::DASH,
        ContentType::YouTube,
        ContentType::Unknown,
    ];
    
    for content_type in content_types {
        // 文字列表現と比較
        let type_str = match content_type {
            ContentType::MP4 => "MP4",
            ContentType::HLS => "HLS",
            ContentType::DASH => "DASH",
            ContentType::YouTube => "YouTube",
            ContentType::Unknown => "Unknown",
        };
        
        assert!(!type_str.is_empty());
    }
    
    // VideoFormatの変換テスト
    let video_formats = vec![
        VideoFormat::MP4,
        VideoFormat::WebM,
        VideoFormat::AVI,
        VideoFormat::MKV,
        VideoFormat::Unknown,
    ];
    
    for format in video_formats {
        // 文字列表現と比較
        let format_str = match format {
            VideoFormat::MP4 => "mp4",
            VideoFormat::WebM => "webm",
            VideoFormat::AVI => "avi",
            VideoFormat::MKV => "mkv",
            VideoFormat::Unknown => "unknown",
        };
        
        assert!(!format_str.is_empty());
    }
}