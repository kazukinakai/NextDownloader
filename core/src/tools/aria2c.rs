use std::path::PathBuf;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use regex::Regex;
use crate::types::{DownloadError, ProgressInfo, ProgressCallback, DownloadOptions};

/// aria2c外部ツールを扱うための構造体
pub struct Aria2cTool {
    /// 実行ファイルのパス
    executable_path: PathBuf,
}

impl Aria2cTool {
    /// 新しいAria2cToolを作成（デフォルトパス使用）
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let default_path = PathBuf::from(".\\bin\\aria2c.exe");
        
        #[cfg(target_os = "macos")]
        let default_path = PathBuf::from("/usr/local/bin/aria2c");
        
        #[cfg(target_os = "linux")]
        let default_path = PathBuf::from("/usr/bin/aria2c");
        
        Self {
            executable_path: default_path
        }
    }
    
    /// 指定したパスでAria2cToolを作成
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            executable_path: path
        }
    }
    
    /// aria2cが利用可能かチェック
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
    
    /// バージョン情報を取得
    pub async fn get_version(&self) -> Result<String, DownloadError> {
        let output = Command::new(&self.executable_path)
            .arg("--version")
            .output()
            .await?;
            
        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            return Err(DownloadError::ProcessFailed(error_message.to_string()));
        }
        
        let version_output = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = version_output.lines().collect();
        
        if let Some(line) = lines.first() {
            if line.contains("aria2") {
                return Ok(line.to_string());
            }
        }
        
        Ok("不明なバージョン".to_string())
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
        // 出力ファイル名
        let output_filename = format!("{}.{}", filename, options.format.to_string().to_lowercase());
        let output_file_path = output_path.join(&output_filename);
        
        // 引数構築
        let mut args = vec![
            format!("-x{}", options.connections),
            format!("-s{}", options.splits),
            format!("-k{}M", options.chunk_size),
            format!("--retry-wait={}", options.retry_wait),
            format!("--max-tries={}", options.max_retries),
            format!("--dir={}", output_path.to_string_lossy()),
            format!("--out={}", output_filename),
            "--summary-interval=1".to_string(),
            "--download-result=full".to_string(),
            "--file-allocation=none".to_string(),
        ];
        
        // HTTP/2サポート
        if options.use_http2 {
            args.push("--enable-http-pipelining=true".to_string());
            args.push("--http2=true".to_string());
        }
        
        // QUICサポート
        if options.use_quic {
            args.push("--enable-quic=true".to_string());
        }
        
        // Keep-Aliveサポート
        if options.use_keep_alive {
            args.push("--enable-http-keep-alive=true".to_string());
        }
        
        // URLを追加
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
            let progress_re = Regex::new(r"\d+%").unwrap();
            let speed_re = Regex::new(r"DL:(\S+)").unwrap();
            let eta_re = Regex::new(r"ETA:(\S+)").unwrap();
            
            tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    let mut progress_info = ProgressInfo {
                        progress: 0.0,
                        speed: String::new(),
                        eta: String::new(),
                    };
                    
                    // 進捗抽出
                    if let Some(progress_match) = progress_re.find(&line) {
                        let progress_str = progress_match.as_str().trim_end_matches('%');
                        if let Ok(progress) = progress_str.parse::<f64>() {
                            progress_info.progress = progress / 100.0;
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
        
        // 出力ファイルが存在するか確認
        if !output_file_path.exists() {
            return Err(DownloadError::FileNotFound);
        }
        
        Ok(output_file_path)
    }
}
