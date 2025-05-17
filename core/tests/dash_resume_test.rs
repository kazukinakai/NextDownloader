use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use tokio::sync::Mutex;
use nextdownloader_core::tools::dash::{
    DashDownloadTool, DownloadOptions, DownloadState, ProgressCallback, ProgressInfo, DownloadError
};
use std::collections::HashMap;
use std::ops::DerefMut;

#[tokio::test]
async fn test_resume_download() -> Result<(), Box<dyn std::error::Error>> {
    // テスト用の一時ディレクトリを作成
    let temp_dir = tempdir()?;
    let output_path = temp_dir.path().join("test_output.mp4");
    
    // テスト用のDASHツールを作成
    let dash_tool = DashDownloadTool::new();
    
    // テスト用のダウンロードオプション
    let options = DownloadOptions {
        max_concurrent_downloads: Some(2),
        // 他のオプションはデフォルト値を使用
        ..Default::default()
    };
    
    // 進捗を追跡するための変数
    let progress_counter = Arc::new(Mutex::new(0));
    let progress_counter_clone = progress_counter.clone();
    
    // 進捗コールバック
    let progress_callback: ProgressCallback = Box::new(move |_progress: ProgressInfo| {
        let mut counter = progress_counter_clone.blocking_lock();
        *counter += 1;
    });
    
    // テスト用の状態を作成（途中までダウンロードされた状態をシミュレート）
    let state = DownloadState {
        source_url: "http://example.com/test.mpd".to_string(),
        output_path: output_path.to_str().unwrap().to_string(),
        completed_segments: vec![0, 1],  // 最初の2セグメントは完了済み
        total_segments: 5,
        segment_mapping: HashMap::new(),
        options: options.clone(),
    };
    
    // 状態ファイルを保存
    let state_file = temp_dir.path().join("test_output.json");
    state.save(&state_file).await?;
    
    // 状態をロードして検証
    let loaded_state = DownloadState::load(&state_file).await?;
    assert_eq!(loaded_state.completed_segments, vec![0, 1]);
    assert_eq!(loaded_state.total_segments, 5);
    
    // セグメントが完了済みとしてマークされているか確認
    assert!(loaded_state.is_segment_completed(0));
    assert!(loaded_state.is_segment_completed(1));
    assert!(!loaded_state.is_segment_completed(2));
    
    // セグメントを完了としてマークして保存
    let mut state = loaded_state;
    state.mark_segment_completed(2);
    state.save(&state_file).await?;
    
    // 再度ロードして検証
    let loaded_state = DownloadState::load(&state_file).await?;
    assert_eq!(loaded_state.completed_segments, vec![0, 1, 2]);
    
    // テスト完了
    Ok(())
}

#[tokio::test]
async fn test_download_state_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // テスト用の状態を作成
    let mut state = DownloadState {
        source_url: "http://example.com/test.mpd".to_string(),
        output_path: "/path/to/output.mp4".to_string(),
        completed_segments: vec![0, 1, 2],
        total_segments: 5,
        segment_mapping: {
            let mut map = HashMap::new();
            map.insert("segment1".to_string(), "http://example.com/seg1".to_string());
            map
        },
        options: DownloadOptions {
            max_concurrent_downloads: Some(4),
            ..Default::default()
        },
    };
    
    // 一時ファイルに保存
    let temp_dir = tempfile::tempdir()?;
    let state_file = temp_dir.path().join("state.json");
    state.save(&state_file).await?;
    
    // 保存されたファイルを読み込んで検証
    let loaded_state = DownloadState::load(&state_file).await?;
    
    assert_eq!(loaded_state.source_url, state.source_url);
    assert_eq!(loaded_state.output_path, state.output_path);
    assert_eq!(loaded_state.completed_segments, state.completed_segments);
    assert_eq!(loaded_state.total_segments, state.total_segments);
    assert_eq!(loaded_state.segment_mapping, state.segment_mapping);
    assert_eq!(loaded_state.options.max_concurrent_downloads, state.options.max_concurrent_downloads);
    
    Ok(())
}
