use std::path::PathBuf;
use tokio::process::Command;
use crate::types::{DownloadError, VideoFormat};
use std::ffi::OsStr;
use tokio::fs;

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
        input_path: &PathBuf,
        output_file: &PathBuf
    ) -> Result<(), DownloadError> {
        // 入力がディレクトリの場合、セグメントリストを作成
        if input_path.is_dir() {
            let segments_list = self.create_segments_list(input_path).await?;
            let temp_list_path = input_path.join("segments.txt");
            fs::write(&temp_list_path, segments_list).await
                .map_err(|e| DownloadError::IoError(format!("セグメントリストの書き込みに失敗しました: {}", e)))?;
            
            // ffmpegを使用して音声を抽出
            let args = vec![
                "-f", "concat",
                "-safe", "0",
                "-i", temp_list_path.to_str().unwrap_or(""),
                "-vn", // ビデオストリームを除外
                "-c:a", "libmp3lame", // MP3エンコーダーを使用
                "-q:a", "2", // 品質設定
                "-y", // 既存ファイルを上書き
                output_file.to_str().unwrap_or(""),
            ];
            
            self.run_ffmpeg_command(&args).await?;
            
            // 一時ファイルを削除
            let _ = fs::remove_file(temp_list_path).await;
            
            Ok(())
        } else {
            // 単一ファイルからの音声抽出
            let args = vec![
                "-i", input_path.to_str().unwrap_or(""),
                "-vn", // ビデオストリームを除外
                "-c:a", "libmp3lame", // MP3エンコーダーを使用
                "-q:a", "2", // 品質設定
                "-y", // 既存ファイルを上書き
                output_file.to_str().unwrap_or(""),
            ];
            
            self.run_ffmpeg_command(&args).await
        }
    }
    
    /// HLSセグメントを結合
    pub async fn concat_segments(
        &self,
        segments_dir: &PathBuf,
        output_file: &PathBuf,
        format: VideoFormat
    ) -> Result<(), DownloadError> {
        // セグメントリストを作成
        let segments_list = self.create_segments_list(segments_dir).await?;
        let temp_list_path = segments_dir.join("segments.txt");
        fs::write(&temp_list_path, segments_list).await
            .map_err(|e| DownloadError::IoError(format!("セグメントリストの書き込みに失敗しました: {}", e)))?;
        
        // フォーマットに応じた引数を設定
        let mut args = vec![
            "-f", "concat",
            "-safe", "0",
            "-i", temp_list_path.to_str().unwrap_or(""),
        ];
        
        match format {
            VideoFormat::Mp4 => {
                args.extend_from_slice(&[
                    "-c:v", "copy",
                    "-c:a", "aac",
                    "-movflags", "faststart", // Web再生用に最適化
                ]);
            },
            VideoFormat::Mkv => {
                args.extend_from_slice(&[
                    "-c:v", "copy",
                    "-c:a", "copy",
                ]);
            },
            VideoFormat::Mp3 => {
                args.extend_from_slice(&[
                    "-vn", // ビデオストリームを除外
                    "-c:a", "libmp3lame",
                    "-q:a", "2",
                ]);
            },
            _ => {
                // デフォルトはそのままコピー
                args.extend_from_slice(&[
                    "-c", "copy",
                ]);
            }
        }
        
        // 出力ファイルを追加
        args.extend_from_slice(&[
            "-y", // 既存ファイルを上書き
            output_file.to_str().unwrap_or(""),
        ]);
        
        // ffmpegコマンドを実行
        self.run_ffmpeg_command(&args).await?;
        
        // 一時ファイルを削除
        let _ = fs::remove_file(temp_list_path).await;
        
        Ok(())
    }
    
    /// ディレクトリ内のセグメントファイルからリストを作成
    async fn create_segments_list(&self, dir: &PathBuf) -> Result<String, DownloadError> {
        // ディレクトリ内のファイルを取得
        let mut entries = fs::read_dir(dir).await
            .map_err(|e| DownloadError::IoError(format!("ディレクトリの読み込みに失敗しました: {}", e)))?;
        
        // .tsファイルを収集
        let mut segment_files = Vec::new();
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| DownloadError::IoError(format!("ディレクトリエントリの読み込みに失敗しました: {}", e)))? {
            
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("ts")) {
                segment_files.push(path);
            }
        }
        
        // 数字順にソート
        segment_files.sort_by(|a, b| {
            let a_name = a.file_name().unwrap_or_default().to_string_lossy();
            let b_name = b.file_name().unwrap_or_default().to_string_lossy();
            a_name.cmp(&b_name)
        });
        
        // ffmpegのconcatフォーマットに合わせたリストを作成
        let mut list = String::new();
        for file in segment_files {
            // ファイル名にシングルクォートが含まれる場合はエスケープ
            let file_path = file.to_string_lossy().replace("'", "'\\'''");
            list.push_str(&format!("file '{}'", file_path));
            list.push('\n');
        }
        
        if list.is_empty() {
            return Err(DownloadError::ProcessFailed("セグメントファイルが見つかりません".to_string()));
        }
        
        Ok(list)
    }
    
    /// ffmpegコマンドを実行
    async fn run_ffmpeg_command(&self, args: &[&str]) -> Result<(), DownloadError> {
        // プロセス起動
        let mut child = Command::new(&self.executable_path)
            .args(args)
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| DownloadError::ProcessFailed(format!("ffmpegの起動に失敗しました: {}", e)))?;
            
        // 実行終了を待機
        let status = child.wait().await
            .map_err(|e| DownloadError::ProcessFailed(format!("ffmpegの実行中にエラーが発生しました: {}", e)))?;
        
        if !status.success() {
            let mut stderr = child.stderr.take().expect("Failed to get stderr");
            let mut error_message = String::new();
            stderr.read_to_string(&mut error_message).await
                .map_err(|e| DownloadError::IoError(format!("エラー出力の読み込みに失敗しました: {}", e)))?;
            
            return Err(DownloadError::ProcessFailed(format!("ffmpegがエラーを返しました: {}", error_message)));
        }
        
        Ok(())
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
