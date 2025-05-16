use std::path::PathBuf;
use tokio::process::Command;
use crate::types::{DownloadError, VideoFormat};

/// FFmpeg外部ツールを扱うための構造体
pub struct FFmpegTool {
    /// 実行ファイルのパス
    executable_path: PathBuf,
}

impl FFmpegTool {
    /// 新しいFFmpegToolを作成（デフォルトパス使用）
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let default_path = PathBuf::from(".\\bin\\ffmpeg.exe");
        
        #[cfg(target_os = "macos")]
        let default_path = PathBuf::from("/usr/local/bin/ffmpeg");
        
        #[cfg(target_os = "linux")]
        let default_path = PathBuf::from("/usr/bin/ffmpeg");
        
        Self {
            executable_path: default_path
        }
    }
    
    /// 指定したパスでFFmpegToolを作成
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            executable_path: path
        }
    }
    
    /// ffmpegが利用可能かチェック
    pub async fn is_available(&self) -> bool {
        if !self.executable_path.exists() {
            return false;
        }
        
        let result = Command::new(&self.executable_path)
            .arg("-version")
            .output()
            .await;
            
        result.is_ok()
    }
    
    /// 動画を処理する
    pub async fn process_video(
        &self,
        input_url: &PathBuf,
        output_path: &PathBuf,
        filename: &str,
        format: &VideoFormat
    ) -> Result<PathBuf, DownloadError> {
        // 出力ファイル名
        let output_filename = format!("{}.{}", filename, format.to_string().to_lowercase());
        let output_file_path = output_path.join(&output_filename);
        
        // 基本的な引数
        let mut args = vec![
            "-i".to_string(),
            input_url.to_string_lossy().to_string(),
            "-c:v".to_string(),
            "copy".to_string(),
            "-c:a".to_string(),
            "copy".to_string(),
            "-y".to_string(),
        ];
        
        // フォーマット固有の設定
        match format {
            VideoFormat::Mp4 => {
                args.push("-movflags".to_string());
                args.push("faststart".to_string());
            },
            VideoFormat::Mkv => {
                // mkvの場合は特別な設定なし
            },
            VideoFormat::Mp3 => {
                args.push("-vn".to_string()); // ビデオストリームを除外
            },
        }
        
        // 出力ファイルパスを追加
        args.push(output_file_path.to_string_lossy().to_string());
        
        // プロセス起動
        let mut child = Command::new(&self.executable_path)
            .args(&args)
            .stderr(std::process::Stdio::piped())
            .spawn()?;
            
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
    
    /// 音声を抽出
    pub async fn extract_audio(
        &self,
        input_url: &PathBuf,
        output_path: &PathBuf,
        filename: &str
    ) -> Result<PathBuf, DownloadError> {
        self.process_video(input_url, output_path, filename, &VideoFormat::Mp3).await
    }
}

impl std::fmt::Display for VideoFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoFormat::Mp4 => write!(f, "MP4"),
            VideoFormat::Mkv => write!(f, "MKV"),
            VideoFormat::Mp3 => write!(f, "MP3"),
        }
    }
}
