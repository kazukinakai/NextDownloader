use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;
use futures::stream;
use reqwest::Client;
use crate::types::{DownloadError, ProgressCallback, DownloadOptions, ProgressInfo};
use crate::tools::{YtDlpTool, Aria2cTool, FFmpegTool};
use crate::utils::ensure_dir_exists;

/// HLSダウンロードを扱うための構造体
pub struct HlsDownloadTool {
    ytdlp: YtDlpTool,
    aria2c: Aria2cTool,
    ffmpeg: FFmpegTool,
    client: Client,
    active_downloads: Arc<Mutex<std::collections::HashMap<String, bool>>>,
}

impl HlsDownloadTool {
    /// 新しいHlsDownloadToolを作成
    pub fn new() -> Self {
        Self {
            ytdlp: YtDlpTool::new(),
            aria2c: Aria2cTool::new(),
            ffmpeg: FFmpegTool::new(),
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            active_downloads: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    /// HLSマニフェストを解析してセグメントURLを取得
    pub async fn parse_manifest(&self, url: &str) -> Result<Vec<String>, DownloadError> {
        // まず、URLがm3u8ファイルを直接指しているか確認
        if url.ends_with(".m3u8") {
            return self.parse_m3u8_directly(url).await;
        }
        
        // yt-dlpを使用してURLを展開
        let result = tokio::process::Command::new(self.ytdlp.executable_path())
            .args(&["--get-url", "--no-warnings", url])
            .output()
            .await
            .map_err(|e| DownloadError::ProcessFailed(format!("yt-dlpの実行に失敗しました: {}", e)))?;
            
        if !result.status.success() {
            let error_message = String::from_utf8_lossy(&result.stderr);
            return Err(DownloadError::ProcessFailed(format!("yt-dlpがエラーを返しました: {}", error_message)));
        }
        
        let output = String::from_utf8_lossy(&result.stdout);
        let urls: Vec<String> = output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();
        
        // 取得したURLがm3u8ファイルを指している場合、そのマニフェストを解析
        for url in urls {
            if url.ends_with(".m3u8") {
                return self.parse_m3u8_directly(&url).await;
            }
        }
        
        // m3u8が見つからない場合は空のリストを返す
        Ok(Vec::new())
    }
    
    /// m3u8ファイルを直接解析してセグメントURLを取得
    async fn parse_m3u8_directly(&self, url: &str) -> Result<Vec<String>, DownloadError> {
        let mut segments = Vec::new();
        let mut visited_urls = std::collections::HashSet::new();
        let mut current_url = url.to_string();
        
        // 最大10回のリダイレクトまで許可
        for _ in 0..10 {
            if visited_urls.contains(&current_url) {
                return Err(DownloadError::ProcessFailed("循環参照が検出されました".to_string()));
            }
            visited_urls.insert(current_url.clone());
            
            // m3u8ファイルを取得
            let response = self.client.get(&current_url)
                .send()
                .await
                .map_err(|e| DownloadError::NetworkError(format!("m3u8ファイルの取得に失敗しました: {}", e)))?;
            
            if !response.status().is_success() {
                return Err(DownloadError::NetworkError(format!("m3u8ファイルの取得に失敗しました: HTTP {}", response.status())));
            }
            
            let content = response.text().await
                .map_err(|e| DownloadError::ProcessFailed(format!("m3u8ファイルの読み込みに失敗しました: {}", e)))?;
            
            // ベースURLを取得
            let base_url = self.get_base_url(&current_url);
            
            // m3u8ファイルを解析
            segments.clear();
            let mut is_variant_playlist = false;
            let mut next_url = None;
            
            for line in content.lines() {
                let line = line.trim();
                
                // コメントや空行をスキップ
                if line.is_empty() || line.starts_with("#") {
                    // バリアントプレイリストかチェック
                    if line.starts_with("#EXT-X-STREAM-INF") {
                        is_variant_playlist = true;
                    }
                    continue;
                }
                
                // セグメントURLを取得
                let segment_url = if line.starts_with("http") {
                    line.to_string()
                } else {
                    format!("{}{}", base_url, line)
                };
                
                segments.push(segment_url);
            }
            
            // バリアントプレイリストで、次に処理するURLがある場合
            if is_variant_playlist && !segments.is_empty() {
                next_url = Some(segments[0].clone());
            } else {
                // 通常のプレイリストの場合は終了
                return Ok(segments);
            }
            
            // 次のURLがあれば更新、なければ終了
            if let Some(next) = next_url {
                current_url = next;
            } else {
                break;
            }
        }
        
        Ok(segments)
    }
    
    /// URLからベースURLを取得
    fn get_base_url(&self, url: &str) -> String {
        if let Ok(parsed_url) = Url::parse(url) {
            // URLの最後のスラッシュまでを取得
            let path = parsed_url.path();
            let last_slash_pos = path.rfind('/').unwrap_or(0);
            let base_path = &path[..=last_slash_pos];
            
            // ベースURLを再構築
            let mut base_url = parsed_url.clone();
            base_url.set_path(base_path);
            base_url.to_string()
        } else {
            // URLの解析に失敗した場合、最後のスラッシュまでを取得
            let last_slash_pos = url.rfind('/').unwrap_or(0);
            url[..=last_slash_pos].to_string()
        }
    }
    
    /// HLSストリームをダウンロード
    pub async fn download(
        &self,
        url: &str,
        output_path: &PathBuf,
        filename: &str,
        options: &DownloadOptions,
        progress_callback: Option<ProgressCallback>
    ) -> Result<PathBuf, DownloadError> {
        // ダウンロード方法を選択
        let use_direct_download = options.use_direct_download.unwrap_or(false);
        
        if use_direct_download {
            // 直接ダウンロードを使用
            self.download_direct(url, output_path, filename, options, progress_callback).await
        } else {
            // yt-dlpを使用したダウンロード
            self.download_with_ytdlp(url, output_path, filename, options, progress_callback).await
        }
    }
    
    /// reqwestを使用した直接ダウンロード
    async fn download_direct(
        &self,
        url: &str,
        output_path: &PathBuf,
        filename: &str,
        options: &DownloadOptions,
        progress_callback: Option<ProgressCallback>
    ) -> Result<PathBuf, DownloadError> {
        // 出力ディレクトリが存在するか確認
        ensure_dir_exists(output_path).await
            .map_err(|e| DownloadError::IoError(format!("出力ディレクトリの作成に失敗しました: {}", e)))?;
        
        // 一意なタスクIDを生成
        let task_id = uuid::Uuid::new_v4().to_string();
        
        // アクティブダウンロードに追加
        {
            let mut active_downloads = self.active_downloads.lock().await;
            active_downloads.insert(task_id.clone(), true);
        }
        
        // HLSマニフェストを解析
        let segments = self.parse_manifest(url).await?
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();
        
        if segments.is_empty() {
            return Err(DownloadError::ProcessFailed("HLSセグメントが見つかりません".to_string()));
        }
        
        // 一時ディレクトリを作成
        let temp_dir = output_path.join(format!("{}_temp", filename));
        ensure_dir_exists(&temp_dir).await
            .map_err(|e| DownloadError::IoError(format!("一時ディレクトリの作成に失敗しました: {}", e)))?;
        
        // 並列ダウンロードの設定
        let total_segments = segments.len();
        let connections = options.connections.min(32).max(1) as usize; // 1-32の範囲に制限
        
        // 進捗状況の記録用
        let downloaded = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let downloaded_clone = downloaded.clone();
        
        // 進捗コールバックの設定
        let progress_task = if let Some(callback) = progress_callback {
            let task_id_clone = task_id.clone();
            let active_downloads_clone = self.active_downloads.clone();
            
            Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
                
                loop {
                    interval.tick().await;
                    
                    // ダウンロードがキャンセルされたか確認
                    let is_active = {
                        let active_downloads = active_downloads_clone.lock().await;
                        active_downloads.get(&task_id_clone).copied().unwrap_or(false)
                    };
                    
                    if !is_active {
                        break;
                    }
                    
                    // 進捗状況を計算
                    let downloaded = downloaded_clone.load(std::sync::atomic::Ordering::Relaxed);
                    let progress = if total_segments > 0 {
                        downloaded as f64 / total_segments as f64
                    } else {
                        0.0
                    };
                    
                    // コールバックを呼び出し
                    let progress_info = ProgressInfo {
                        progress,
                        speed: format!("{}/{} segments", downloaded, total_segments),
                        eta: if progress > 0.0 {
                            format!("{:.1}%", progress * 100.0)
                        } else {
                            "計算中...".to_string()
                        },
                    };
                    
                    callback(progress_info);
                    
                    // ダウンロード完了時に終了
                    if downloaded >= total_segments {
                        break;
                    }
                }
            }))
        } else {
            None
        };
        
        // セグメントを並列ダウンロード
        let client = self.client.clone();
        let results = stream::iter(segments)
            .map(|(i, segment_url)| {
                let client = client.clone();
                let temp_dir = temp_dir.clone();
                let downloaded = downloaded.clone();
                
                async move {
                    let segment_path = temp_dir.join(format!("{:08}.ts", i));
                    let result = self.download_segment(&client, &segment_url, &segment_path).await;
                    
                    // ダウンロード完了数を更新
                    downloaded.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    
                    (i, result)
                }
            })
            .buffer_unordered(connections) // 並列数を制限
            .collect::<Vec<_>>()
            .await;
        
        // アクティブダウンロードから削除
        {
            let mut active_downloads = self.active_downloads.lock().await;
            active_downloads.remove(&task_id);
        }
        
        // 進捗タスクが完了するのを待つ
        if let Some(handle) = progress_task {
            let _ = handle.await;
        }
        
        // エラーがあれば返す
        for (_, result) in &results {
            if let Err(e) = result {
                return Err(e.clone());
            }
        }
        
        // 出力ファイル名を決定
        let format_ext = match options.format {
            crate::types::VideoFormat::Mp4 => "mp4",
            crate::types::VideoFormat::Mkv => "mkv",
            crate::types::VideoFormat::Mp3 => "mp3",
            _ => "ts", // デフォルトはTSフォーマット
        };
        
        let output_file = output_path.join(format!("{}.{}", filename, format_ext));
        
        // セグメントを結合
        if options.format == crate::types::VideoFormat::Mp3 {
            // 音声抽出の場合は、ffmpegを使用
            self.ffmpeg.extract_audio(&temp_dir, &output_file).await
                .map_err(|e| DownloadError::ProcessFailed(format!("音声抽出に失敗しました: {}", e)))?;
        } else {
            // ビデオの場合は、ffmpegを使用して結合
            self.ffmpeg.concat_segments(&temp_dir, &output_file, options.format).await
                .map_err(|e| DownloadError::ProcessFailed(format!("セグメントの結合に失敗しました: {}", e)))?;
        }
        
        // 一時ディレクトリを削除
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        
        Ok(output_file)
    }
    
    /// 個別のセグメントをダウンロード
    async fn download_segment(
        &self,
        client: &Client,
        url: &str,
        output_path: &PathBuf
    ) -> Result<(), DownloadError> {
        // セグメントを取得
        let response = client.get(url)
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(format!("セグメントの取得に失敗しました: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(DownloadError::NetworkError(format!("セグメントの取得に失敗しました: HTTP {}", response.status())));
        }
        
        // レスポンスボディを取得
        let bytes = response.bytes().await
            .map_err(|e| DownloadError::NetworkError(format!("セグメントデータの取得に失敗しました: {}", e)))?;
        
        // ファイルに書き込み
        let mut file = tokio::fs::File::create(output_path).await
            .map_err(|e| DownloadError::IoError(format!("セグメントファイルの作成に失敗しました: {}", e)))?;
        
        file.write_all(&bytes).await
            .map_err(|e| DownloadError::IoError(format!("セグメントファイルの書き込みに失敗しました: {}", e)))?;
        
        Ok(())
    }
    
    /// yt-dlpを使用したHLSダウンロード
    async fn download_with_ytdlp(
        &self,
        url: &str,
        output_path: &PathBuf,
        filename: &str,
        options: &DownloadOptions,
        progress_callback: Option<ProgressCallback>
    ) -> Result<PathBuf, DownloadError> {
        // yt-dlpの引数構築
        let mut args = vec![
            "--no-warnings".to_string(),
            "--downloader".to_string(),
            "aria2c".to_string(),
        ];
        
        // aria2cのオプション設定
        let mut aria2c_args = format!(
            "-x{} -s{} -k{}M --retry-wait={} --max-tries={}",
            options.connections,
            options.splits,
            options.chunk_size,
            options.retry_wait,
            options.max_retries
        );
        
        if options.use_http2 {
            aria2c_args.push_str(" --enable-http-pipelining=true --http2=true");
        }
        
        if options.use_quic {
            aria2c_args.push_str(" --enable-quic=true");
        }
        
        if options.use_keep_alive {
            aria2c_args.push_str(" --enable-http-keep-alive=true");
        }
        
        args.push("--downloader-args".to_string());
        args.push(format!("aria2c:{}", aria2c_args));
        
        // 出力ファイル名設定
        args.push("-o".to_string());
        args.push(format!("{}/{}.%(ext)s", output_path.to_string_lossy(), filename));
        
        // フォーマット設定
        match options.format {
            crate::types::VideoFormat::Mp4 => {
                args.push("--merge-output-format".to_string());
                args.push("mp4".to_string());
            }
            crate::types::VideoFormat::Mkv => {
                args.push("--merge-output-format".to_string());
                args.push("mkv".to_string());
            }
            crate::types::VideoFormat::Mp3 => {
                args.push("--extract-audio".to_string());
                args.push("--audio-format".to_string());
                args.push("mp3".to_string());
            }
            _ => {}
        }
        
        // URLを追加
        args.push(url.to_string());
        
        // yt-dlpの呼び出し
        self.ytdlp.run_with_args(&args, progress_callback).await?;
        
        // 生成されたファイルを検索
        let format_ext = match options.format {
            crate::types::VideoFormat::Mp4 => "mp4",
            crate::types::VideoFormat::Mkv => "mkv",
            crate::types::VideoFormat::Mp3 => "mp3",
            _ => "mp4", // デフォルトはMP4
        };
        
        let expected_filename = format!("{}.{}", filename, format_ext);
        let expected_path = output_path.join(&expected_filename);
        
        if expected_path.exists() {
            return Ok(expected_path);
        }
        
        // ファイル名パターンでマッチングを試行
        let mut dir_entries = tokio::fs::read_dir(output_path).await?;
        while let Ok(Some(entry)) = dir_entries.next_entry().await {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            if file_name_str.starts_with(filename) {
                return Ok(entry.path());
            }
        }
        
        Err(DownloadError::FileNotFound)
    }
}

// ytdlpの拡張は ytdlp.rs で実装されています
