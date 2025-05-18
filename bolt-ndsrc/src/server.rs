use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;

use crate::config::Config;
use crate::storage::Storage;
use crate::download::{DownloadManager, DownloadStatus};

pub async fn start_server(
    config: Config,
    storage: Arc<Mutex<Storage>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let download_manager = Arc::new(DownloadManager::new(config.clone(), storage.clone()));
    
    // Status updates channel
    let (tx, _) = broadcast::channel(100);
    let tx = Arc::new(tx);

    // WebSocket handler
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_download_manager(download_manager.clone()))
        .and(with_tx(tx.clone()))
        .map(|ws: warp::ws::Ws, dm, tx| {
            ws.on_upgrade(move |socket| handle_ws_client(socket, dm, tx))
        });

    // HTTP routes
    let download = warp::path("download")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_download_manager(download_manager.clone()))
        .and_then(handle_download);

    let routes = ws_route
        .or(download)
        .with(warp::cors().allow_any_origin());

    println!("Server starting on {}:{}", config.server.host, config.server.port);
    
    warp::serve(routes)
        .run((config.server.host.parse()?, config.server.port))
        .await;

    Ok(())
}

async fn handle_ws_client(
    ws: warp::ws::WebSocket,
    dm: Arc<DownloadManager>,
    tx: Arc<broadcast::Sender<DownloadStatus>>,
) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut status_rx = tx.subscribe();

    // Forward download status updates to WebSocket
    tokio::spawn(async move {
        while let Ok(status) = status_rx.recv().await {
            let msg = warp::ws::Message::text(serde_json::to_string(&status).unwrap());
            if let Err(e) = ws_tx.send(msg).await {
                eprintln!("WebSocket send error: {}", e);
                break;
            }
        }
    });

    // Handle incoming WebSocket messages
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    println!("Received message: {}", text);
                    // Handle WebSocket commands
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
}

async fn handle_download(
    payload: serde_json::Value,
    dm: Arc<DownloadManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Handle download request
    Ok(warp::reply::json(&"Download started"))
}

fn with_download_manager(
    dm: Arc<DownloadManager>,
) -> impl Filter<Extract = (Arc<DownloadManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || dm.clone())
}

fn with_tx(
    tx: Arc<broadcast::Sender<DownloadStatus>>,
) -> impl Filter<Extract = (Arc<broadcast::Sender<DownloadStatus>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tx.clone())
}