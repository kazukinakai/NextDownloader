//! # ユーティリティモジュール
//! 
//! NextDownloaderで使用する共通ユーティリティ関数を提供します。

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use log::{debug, error, info, warn};

use crate::error::DownloaderError;

/// ファイルサイズを人間が読みやすい形式に変換します
pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size < TB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else {
        format!("{:.2} TB", size as f64 / TB as f64)
    }
}

/// 時間を人間が読みやすい形式に変換します
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}秒", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{}分{}秒", minutes, secs)
    } else {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{}時間{}分{}秒", hours, minutes, secs)
    }
}

/// コマンドが存在するかどうかを確認します
pub fn command_exists(command: &str) -> bool {
    #[cfg(target_os = "windows")]
    let output = Command::new("where")
        .arg(command)
        .output();
    
    #[cfg(not(target_os = "windows"))]
    let output = Command::new("which")
        .arg(command)
        .output();
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// ファイルの拡張子を取得します
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// ファイル名から拡張子を除いた部分を取得します
pub fn get_file_stem(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_string())
}

/// ディレクトリが存在しない場合は作成します
pub fn ensure_directory_exists(path: &Path) -> Result<(), DownloaderError> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| DownloaderError::FileSystemError(format!("ディレクトリの作成に失敗しました: {}", e)))?;
    }
    Ok(())
}

/// URLからファイル名を抽出します
pub fn extract_filename_from_url(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|url| {
            url.path_segments()
                .and_then(|segments| segments.last())
                .map(|last_segment| last_segment.to_string())
        })
}

/// 一時ファイルのパスを生成します
pub fn generate_temp_path(original_path: &Path, temp_dir: Option<&Path>) -> PathBuf {
    let file_name = original_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    
    let temp_file_name = format!("{}.part", file_name);
    
    if let Some(dir) = temp_dir {
        dir.join(temp_file_name)
    } else {
        original_path.with_file_name(temp_file_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1023), "1023 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30秒");
        assert_eq!(format_duration(90), "1分30秒");
        assert_eq!(format_duration(3600), "1時間0分0秒");
        assert_eq!(format_duration(3661), "1時間1分1秒");
    }
    
    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension(Path::new("test.mp4")), Some("mp4".to_string()));
        assert_eq!(get_file_extension(Path::new("test.MP4")), Some("mp4".to_string()));
        assert_eq!(get_file_extension(Path::new("test")), None);
    }
    
    #[test]
    fn test_get_file_stem() {
        assert_eq!(get_file_stem(Path::new("test.mp4")), Some("test".to_string()));
        assert_eq!(get_file_stem(Path::new("path/to/test.mp4")), Some("test".to_string()));
        assert_eq!(get_file_stem(Path::new("test")), Some("test".to_string()));
    }
    
    #[test]
    fn test_extract_filename_from_url() {
        assert_eq!(
            extract_filename_from_url("https://example.com/path/to/file.mp4"),
            Some("file.mp4".to_string())
        );
        assert_eq!(
            extract_filename_from_url("https://example.com/"),
            None
        );
    }
    
    #[test]
    fn test_generate_temp_path() {
        let path = Path::new("/path/to/file.mp4");
        assert_eq!(
            generate_temp_path(path, None),
            PathBuf::from("/path/to/file.mp4.part")
        );
        
        let temp_dir = Path::new("/tmp");
        assert_eq!(
            generate_temp_path(path, Some(temp_dir)),
            PathBuf::from("/tmp/file.mp4.part")
        );
    }
}