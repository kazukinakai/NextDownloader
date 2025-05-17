use std::path::PathBuf;
use std::io;
use tokio::process::Command;
use tokio::io::AsyncBufReadExt;
use regex::Regex;
use crate::types::{DownloadError, VideoInfo, ProgressInfo, ProgressCallback, DownloadOptions, FormatInfo};

/// YouTube-DLP外部ツールを扱うための構造体
pub struct YtDlpTool {
    /// 実行ファイルのパス
    executable_path: PathBuf,
}

impl From<io::Error> for DownloadError {
    fn from(error: io::Error) -> Self {
        DownloadError::ProcessFailed(error.to_string())
    }
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
    
    /// 実行ファイルのパスを取得
    pub fn executable_path(&self) -> &PathBuf {
        &self.executable_path
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
    
    /// カスタム引数でyt-dlpを実行
    pub async fn run_with_args(
        &self,
        args: &[String],
        progress_callback: Option<ProgressCallback>,
    ) -> Result<(), DownloadError> {
        let mut child = tokio::process::Command::new(self.executable_path())
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(callback) = progress_callback {
            let stdout = child.stdout.take().expect("Failed to get stdout");
            let stderr = child.stderr.take().expect("Failed to get stderr");
            
            let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
            let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();
            
            loop {
                tokio::select! {
                    result = stdout_reader.next_line() => {
                        if let Ok(Some(line)) = result {
                            // 進捗情報をパースしてコールバックを呼び出す
                            if let Some(progress) = Self::parse_progress(&line) {
                                callback(progress);
                            }
                        } else {
                            break;
                        }
                    }
                    result = stderr_reader.next_line() => {
                        if let Ok(Some(line)) = result {
                            // エラーメッセージをログに出力
                            log::error!("yt-dlp error: {}", line);
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        
        let status = child.wait().await?;
        
        if !status.success() {
            return Err(DownloadError::ProcessFailed("yt-dlp process failed".to_string()));
        }
        
        Ok(())
    }
    
    /// yt-dlpの進捗出力から進捗情報をパース
    fn parse_progress(line: &str) -> Option<ProgressInfo> {
        // 進捗情報をパースする正規表現
        let progress_re = regex::Regex::new(r"(\d+\.?\d*)% of ([\d.]+\s*[KMGT]?i?B) at\s*([\d.]+\s*[KMGT]?i?B/s)(?:\s*ETA\s*(\d+:?\d*))?").ok()?;
        
        // 正規表現でマッチング
        let caps = progress_re.captures(line)?;
        
        // 進捗パーセンテージを取得 (0.0 〜 1.0 に正規化)
        let progress = caps.get(1)?.as_str().parse::<f64>().ok()? / 100.0;
        
        // 速度を取得
        let speed = caps.get(3)?.as_str().trim().to_string();
        
        // ETAを取得 (オプション)
        let eta = caps.get(4).map(|m| m.as_str().to_string()).unwrap_or_default();
        
        Some(ProgressInfo {
            progress,
            speed,
            eta,
        })
    }
}
