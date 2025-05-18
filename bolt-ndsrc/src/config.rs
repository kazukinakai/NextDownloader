use serde::{Deserialize, Serialize};
use tokio::fs;
use directories::ProjectDirs;

use crate::error::{DownloadError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub download: DownloadConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub chunk_size: usize,
    pub max_concurrent: usize,
    pub update_interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_path: String,
    pub db_path: String,
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if let Ok(content) = fs::read_to_string(&config_path).await {
            toml::from_str(&content).map_err(|e| DownloadError::ConfigError(e.to_string()))
        } else {
            let config = Self::default();
            let content = toml::to_string_pretty(&config)
                .map_err(|e| DownloadError::ConfigError(e.to_string()))?;
            
            fs::create_dir_all(config_path.parent().unwrap()).await
                .map_err(|e| DownloadError::FileError(e.to_string()))?;
            
            fs::write(&config_path, content).await
                .map_err(|e| DownloadError::FileError(e.to_string()))?;
            
            Ok(config)
        }
    }

    fn get_config_path() -> Result<std::path::PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "nextdownloader", "NextDownloader")
            .ok_or_else(|| DownloadError::ConfigError("Could not determine config directory".into()))?;
        
        Ok(proj_dirs.config_dir().join("config.toml"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 8080,
                host: "127.0.0.1".into(),
            },
            download: DownloadConfig {
                chunk_size: 1024 * 1024 * 5, // 5MB
                max_concurrent: 3,
                update_interval: 250,
            },
            storage: StorageConfig {
                base_path: dirs::download_dir()
                    .unwrap_or_else(|| std::env::current_dir().unwrap())
                    .join("NextDownloader")
                    .to_string_lossy()
                    .into(),
                db_path: ProjectDirs::from("com", "nextdownloader", "NextDownloader")
                    .unwrap()
                    .data_dir()
                    .join("downloads.db")
                    .to_string_lossy()
                    .into(),
            },
        }
    }
}