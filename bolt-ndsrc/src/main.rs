use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

mod download;
mod server;
mod analysis;
mod storage;
mod config;
mod error;

use crate::server::start_server;
use crate::config::Config;
use crate::storage::Storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration
    let config = Config::load().await?;
    
    // Initialize storage
    let storage = Arc::new(Mutex::new(Storage::new(&config).await?));
    
    // Start the server
    start_server(config, storage).await?;
    
    Ok(())
}