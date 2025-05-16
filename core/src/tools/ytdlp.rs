use std::path::PathBuf;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use regex::Regex;
use crate::types::{DownloadError, VideoInfo, ProgressInfo, ProgressCallback, DownloadOptions, FormatInfo};

/// YouTube-DLP外部ツールを扱うための構造体
pub struct YtDlpTool {
    /// 実行ファイルのパス
    executable_path: PathBuf,
}

impl YtDlpTool {
    /// 新しいYtDlpToolを作成（デフォルトパス使用）
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let default_path = PathBuf::from(".\\bin\\yt-dlp.exe");
        
        #[cfg(target_os = "macos")]
        let default_path = PathBuf::from("/usr/local/bin/yt-dlp");
        
        #[cfg(target_os = "linux")]
        let default_path = PathBuf::from("/usr/bin/yt-dlp");
        
        Self {
            executable_path: default_path
        }
    }
    
    /// 指定したパスでYtDlpToolを作成
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            executable_path: path
        }
    }
    
    /// yt-dlpが利用可能かチェック
    pub async fn is_available(&self) -> bool {
        if !self.executable_path.exists() {
            return false;
        }
        
        let result = Command::new(&self.executable_path)
            .arg("--version")
            .output()
            .await;
            
        result.is_ok()
    }
    
    /// 動画情報を取得
    pub async fn get_video_info(&self, url: &str) -> Result<VideoInfo, DownloadError> {
        let output = Command::new(&self.executable_path)
            .arg("-J")
            .arg("--no-warnings")
            .arg(url)
            .output()
            .await?;
            
        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            return Err(DownloadError::ProcessFailed(error_message.to_string()));
        }
        
        let json_str = String::from_utf8_lossy(&output.stdout);
        let video_info: VideoInfo = serde_json::from_str(&json_str)?;
        
        Ok(video_info)
    }
    
    /// 動画をダウンロード
    pub async fn download(
        &self,
        url: &str,
        output_path: &PathBuf,
        filename: &str,
        options: &DownloadOptions,
        progress_callback: Option<ProgressCallback>
    ) -> Result<PathBuf, DownloadError> {
        // 引数構築
        let mut args = vec![
            "--no-warnings".to_string(),
            "--downloader".to_string(), 
            "aria2c".to_string(),
        ];
        
        // aria2c引数
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
        
        // 出力フォーマット
        let output_template = format!("{}/{}",
            output_path.to_string_lossy(), 
            filename
        );
        
        args.push("-o".to_string());
        args.push(format!("{}.%(ext)s", output_template));
        
        // フォーマット
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
        }
        
        // URL追加
        args.push(url.to_string());
        
        // プロセス起動
        let mut child = Command::new(&self.executable_path)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
            
        // 進捗処理
        if let Some(callback) = progress_callback {
            let stdout = child.stdout.take().expect("Failed to get stdout");
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            
            // 正規表現コンパイル
            let progress_re = Regex::new(r"(\d+\.\d+)%").unwrap();
            let speed_re = Regex::new(r"at\s+(\S+/s)").unwrap();
            let eta_re = Regex::new(r"ETA\s+(\S+)").unwrap();
            
            tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    let mut progress_info = ProgressInfo {
                        progress: 0.0,
                        speed: String::new(),
                        eta: String::new(),
                    };
                    
                    // 進捗抽出
                    if let Some(caps) = progress_re.captures(&line) {
                        if let Some(progress_str) = caps.get(1) {
                            if let Ok(progress) = progress_str.as_str().parse::<f64>() {
                                progress_info.progress = progress / 100.0;
                            }
                        }
                    }
                    
                    // 速度抽出
                    if let Some(caps) = speed_re.captures(&line) {
                        if let Some(speed) = caps.get(1) {
                            progress_info.speed = speed.as_str().to_string();
                        }
                    }
                    
                    // ETA抽出
                    if let Some(caps) = eta_re.captures(&line) {
                        if let Some(eta) = caps.get(1) {
                            progress_info.eta = eta.as_str().to_string();
                        }
                    }
                    
                    // コールバック実行
                    if progress_info.progress > 0.0 || !progress_info.speed.is_empty() {
                        callback(progress_info);
                    }
                }
            });
        }
        
        // 実行終了を待機
        let status = child.wait().await?;
        
        if !status.success() {
            let mut stderr = child.stderr.take().expect("Failed to get stderr");
            let mut error_message = String::new();
            use tokio::io::AsyncReadExt;
            stderr.read_to_string(&mut error_message).await?;
            
            return Err(DownloadError::ProcessFailed(error_message));
        }
        
        // 出力ファイルを探す
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
