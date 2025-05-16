use std::path::PathBuf;
use crate::types::{DownloadError, ProgressCallback, DownloadOptions};
use crate::tools::{YtDlpTool, Aria2cTool, FFmpegTool};

/// HLSダウンロードを扱うための構造体
pub struct HlsDownloadTool {
    ytdlp: YtDlpTool,
    aria2c: Aria2cTool,
    ffmpeg: FFmpegTool,
}

impl HlsDownloadTool {
    /// 新しいHlsDownloadToolを作成
    pub fn new() -> Self {
        Self {
            ytdlp: YtDlpTool::new(),
            aria2c: Aria2cTool::new(),
            ffmpeg: FFmpegTool::new(),
        }
    }
    
    /// HLSマニフェストを解析してセグメントURLを取得
    pub async fn parse_manifest(&self, url: &str) -> Result<Vec<String>, DownloadError> {
        // yt-dlpを使用してURLを展開
        // 実際には、yt-dlpの--dump-jsonやパターンマッチングなどを組み合わせて実装する
        // ここでは簡易的に--get-urlを使用して、ストリーミングURLのみを取得
        let result = tokio::process::Command::new(self.ytdlp.executable_path())
            .args(&["--get-url", "--no-warnings", url])
            .output()
            .await?;
            
        if !result.status.success() {
            let error_message = String::from_utf8_lossy(&result.stderr);
            return Err(DownloadError::ProcessFailed(error_message.to_string()));
        }
        
        let output = String::from_utf8_lossy(&result.stdout);
        let segments: Vec<String> = output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();
            
        Ok(segments)
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
        // yt-dlpを用いてHLSストリームをダウンロード
        // 最も簡単かつ堅牢なアプローチ
        
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
        }
        
        // URLを追加
        args.push(url.to_string());
        
        // yt-dlpの呼び出し
        // 本来はここで詳細な実装が必要だが、既存のYtDlpToolを使用して簡略化
        self.ytdlp.run_with_args(&args, progress_callback).await?;
        
        // 生成されたファイルを検索
        let format_ext = match options.format {
            crate::types::VideoFormat::Mp4 => "mp4",
            crate::types::VideoFormat::Mkv => "mkv",
            crate::types::VideoFormat::Mp3 => "mp3",
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

// ytdlpの拡張
impl YtDlpTool {
    pub fn executable_path(&self) -> &PathBuf {
        &self.executable_path
    }
    
    pub async fn run_with_args(
        &self,
        args: &[String],
        progress_callback: Option<ProgressCallback>
    ) -> Result<(), DownloadError> {
        let mut child = tokio::process::Command::new(&self.executable_path)
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
            
        if let Some(callback) = progress_callback {
            let stdout = child.stdout.take().expect("Failed to get stdout");
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();
            
            // 進捗パースの正規表現
            let progress_re = regex::Regex::new(r"(\d+\.\d+)%").unwrap();
            let speed_re = regex::Regex::new(r"at\s+(\S+/s)").unwrap();
            let eta_re = regex::Regex::new(r"ETA\s+(\S+)").unwrap();
            
            tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    let mut progress_info = crate::types::ProgressInfo {
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
        
        let status = child.wait().await?;
        
        if !status.success() {
            let mut stderr = child.stderr.take().expect("Failed to get stderr");
            let mut error_message = String::new();
            use tokio::io::AsyncReadExt;
            stderr.read_to_string(&mut error_message).await?;
            
            return Err(DownloadError::ProcessFailed(error_message));
        }
        
        Ok(())
    }
}
