use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use sha2::{Sha256, Digest};
use url::Url;

use crate::config::Config;
use crate::storage::Storage;
use crate::error::{DownloadError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStatus {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub progress: f32,
    pub speed: u64,
    pub state: DownloadState,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadState {
    Queued,
    Downloading,
    Paused,
    Completed,
    Error,
}

pub struct DownloadManager {
    config: Config,
    storage: Arc<Mutex<Storage>>,
    client: Client,
}

impl DownloadManager {
    pub fn new(config: Config, storage: Arc<Mutex<Storage>>) -> Self {
        Self {
            config,
            storage,
            client: Client::new(),
        }
    }

    pub async fn start_download(&self, url: &str) -> Result<String> {
        let url = Url::parse(url)
            .map_err(|e| DownloadError::ParseError(e.to_string()))?;
        
        // Generate unique ID
        let id = generate_download_id(&url);
        
        // Create download entry
        let download = DownloadStatus {
            id: id.clone(),
            url: url.to_string(),
            filename: get_filename_from_url(&url),
            progress: 0.0,
            speed: 0,
            state: DownloadState::Queued,
            error: None,
        };

        // Save to storage
        self.storage.lock().await.add_download(&download).await?;

        Ok(id)
    }

    pub async fn pause_download(&self, id: &str) -> Result<()> {
        // Implementation
        Ok(())
    }

    pub async fn resume_download(&self, id: &str) -> Result<()> {
        // Implementation
        Ok(())
    }

    pub async fn cancel_download(&self, id: &str) -> Result<()> {
        // Implementation
        Ok(())
    }
}

fn generate_download_id(url: &Url) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_str().as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_filename_from_url(url: &Url) -> String {
    url.path_segments()
        .and_then(|segments| segments.last())
        .unwrap_or("download")
        .to_string()
}