//! # 設定モジュール
//! 
//! NextDownloaderの設定を管理するモジュールです。

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::error::DownloaderError;
use crate::DownloadManagerConfig;

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// ダウンロードマネージャーの設定
    pub download_manager: DownloadManagerConfig,
    /// デフォルトのダウンロードディレクトリ
    pub default_download_dir: PathBuf,
    /// テーマ（"light", "dark", "system"）
    pub theme: String,
    /// 言語（"ja", "en"）
    pub language: String,
    /// ダウンロード完了時に通知する
    pub notify_on_completion: bool,
    /// アーカイブを自動的に解凍する
    pub auto_extract_archives: bool,
    /// 依存ツールのパス
    pub tool_paths: ToolPaths,
}

/// 依存ツールのパス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPaths {
    /// yt-dlpのパス
    pub ytdlp: Option<PathBuf>,
    /// aria2cのパス
    pub aria2c: Option<PathBuf>,
    /// ffmpegのパス
    pub ffmpeg: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_download_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("./downloads"));
        
        Self {
            download_manager: DownloadManagerConfig::default(),
            default_download_dir,
            theme: "system".to_string(),
            language: "ja".to_string(),
            notify_on_completion: true,
            auto_extract_archives: false,
            tool_paths: ToolPaths {
                ytdlp: None,
                aria2c: None,
                ffmpeg: None,
            },
        }
    }
}

impl AppConfig {
    /// 設定ファイルからアプリケーション設定を読み込みます
    pub fn load(path: &Path) -> Result<Self, DownloaderError> {
        if !path.exists() {
            debug!("設定ファイルが存在しないため、デフォルト設定を使用します: {:?}", path);
            return Ok(Self::default());
        }
        
        let config_str = fs::read_to_string(path)
            .map_err(|e| DownloaderError::FileSystemError(format!("設定ファイルの読み込みに失敗しました: {}", e)))?;
        
        let config: AppConfig = serde_json::from_str(&config_str)
            .map_err(|e| DownloaderError::UnknownError(format!("設定ファイルのパースに失敗しました: {}", e)))?;
        
        Ok(config)
    }
    
    /// アプリケーション設定を設定ファイルに保存します
    pub fn save(&self, path: &Path) -> Result<(), DownloaderError> {
        // 親ディレクトリが存在しない場合は作成
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| DownloaderError::FileSystemError(format!("設定ディレクトリの作成に失敗しました: {}", e)))?;
            }
        }
        
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| DownloaderError::UnknownError(format!("設定のシリアライズに失敗しました: {}", e)))?;
        
        fs::write(path, config_str)
            .map_err(|e| DownloaderError::FileSystemError(format!("設定ファイルの保存に失敗しました: {}", e)))?;
        
        Ok(())
    }
    
    /// デフォルトの設定ファイルパスを取得します
    pub fn default_config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        
        config_dir.join("nextdownloader").join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.theme, "system");
        assert_eq!(config.language, "ja");
        assert_eq!(config.notify_on_completion, true);
        assert_eq!(config.auto_extract_archives, false);
    }
    
    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        
        let mut config = AppConfig::default();
        config.theme = "dark".to_string();
        config.language = "en".to_string();
        
        // 設定を保存
        config.save(&config_path).unwrap();
        
        // 設定を読み込み
        let loaded_config = AppConfig::load(&config_path).unwrap();
        
        assert_eq!(loaded_config.theme, "dark");
        assert_eq!(loaded_config.language, "en");
    }
    
    #[test]
    fn test_load_nonexistent_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("nonexistent.json");
        
        // 存在しない設定ファイルを読み込むとデフォルト設定が返される
        let config = AppConfig::load(&config_path).unwrap();
        assert_eq!(config.theme, "system");
    }
}