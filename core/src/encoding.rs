//! # エンコーディングモジュール
//! 
//! 動画のエンコーディング（変換）機能を提供します。

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::fs;
use std::process::Command;

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// エンコーディングエラー
#[derive(Debug, Error)]
pub enum EncodingError {
    /// FFmpegが見つかりません
    #[error("FFmpegが見つかりません")]
    FFmpegNotFound,
    
    /// エンコーディング処理が失敗しました
    #[error("エンコーディング処理が失敗しました: {0}")]
    EncodingFailed(String),
    
    /// ファイルエラー
    #[error("ファイルエラー: {0}")]
    FileError(String),
    
    /// その他のエラー
    #[error("エンコードエラー: {0}")]
    Other(String),
}

/// エンコーディングプリセット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncodingPreset {
    /// 非常に高速（低品質）
    UltraFast,
    /// 高速（低品質）
    VeryFast,
    /// 速い（中品質）
    Faster,
    /// 標準（中品質）
    Medium,
    /// 高品質（低速）
    Slow,
    /// 非常に高品質（非常に低速）
    VerySlow,
}

impl EncodingPreset {
    /// FFmpeg用のプリセット文字列を返します
    pub fn to_string(&self) -> &'static str {
        match self {
            EncodingPreset::UltraFast => "ultrafast",
            EncodingPreset::VeryFast => "veryfast",
            EncodingPreset::Faster => "faster",
            EncodingPreset::Medium => "medium",
            EncodingPreset::Slow => "slow",
            EncodingPreset::VerySlow => "veryslow",
        }
    }
}

impl Default for EncodingPreset {
    fn default() -> Self {
        EncodingPreset::Medium
    }
}

/// エンコーディングオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingOptions {
    /// 出力フォーマット
    pub format: VideoFormat,
    /// エンコーディングプリセット
    pub preset: EncodingPreset,
    /// ビットレート (bps)
    pub bitrate: Option<u64>,
    /// CRF（品質）値 (0-51, 低いほど高品質)
    pub crf: Option<u8>,
    /// 解像度（幅x高さ）
    pub resolution: Option<(u32, u32)>,
    /// フレームレート
    pub framerate: Option<u32>,
    /// 音声ビットレート (bps)
    pub audio_bitrate: Option<u64>,
    /// 音声チャンネル数
    pub audio_channels: Option<u8>,
    /// ハードウェアアクセラレーション
    pub hardware_accel: bool,
    /// メタデータ
    pub metadata: Vec<(String, String)>,
    /// 進捗コールバック
    #[serde(skip)]
    pub progress_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
}

impl Default for EncodingOptions {
    fn default() -> Self {
        Self {
            format: VideoFormat::H265,
            preset: EncodingPreset::Medium,
            bitrate: None,
            crf: Some(28), // H.265の場合、H.264より低い値でも同等の品質
            resolution: None,
            framerate: None,
            audio_bitrate: None,
            audio_channels: None,
            hardware_accel: true,
            metadata: Vec::new(),
            progress_callback: None,
        }
    }
}

/// 動画フォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoFormat {
    /// H.264 / AVC
    H264,
    /// H.265 / HEVC
    H265,
    /// VP9
    VP9,
    /// AV1
    AV1,
}

impl VideoFormat {
    /// FFmpeg用のコーデック文字列を返します
    pub fn codec_string(&self) -> &'static str {
        match self {
            VideoFormat::H264 => "libx264",
            VideoFormat::H265 => "libx265",
            VideoFormat::VP9 => "libvpx-vp9",
            VideoFormat::AV1 => "libaom-av1",
        }
    }
    
    /// ハードウェアアクセラレーション用のコーデック文字列を返します
    pub fn hw_codec_string(&self) -> Option<&'static str> {
        match self {
            VideoFormat::H264 => Some("h264_videotoolbox"),
            VideoFormat::H265 => Some("hevc_videotoolbox"),
            VideoFormat::VP9 => None, // macOSではVP9のハードウェアアクセラレーションに対応していない
            VideoFormat::AV1 => None, // macOSではAV1のハードウェアアクセラレーションに対応していない
        }
    }
}

/// エンコーディングマネージャー
pub struct EncodingManager {
    /// FFmpegのパス
    ffmpeg_path: PathBuf,
}

impl EncodingManager {
    /// 新しいエンコーディングマネージャーを作成します
    pub fn new() -> Result<Self, EncodingError> {
        // FFmpegのパスを検索
        let ffmpeg_path = Self::find_ffmpeg()
            .ok_or(EncodingError::FFmpegNotFound)?;
        
        Ok(Self {
            ffmpeg_path,
        })
    }
    
    /// FFmpegのパスを検索します
    fn find_ffmpeg() -> Option<PathBuf> {
        // 環境変数PATHからFFmpegを検索
        if let Ok(output) = Command::new("which").arg("ffmpeg").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path_str = path_str.trim();
                if !path_str.is_empty() {
                    return Some(PathBuf::from(path_str));
                }
            }
        }
        
        // 一般的なインストール場所をチェック
        let common_paths = [
            "/usr/bin/ffmpeg",
            "/usr/local/bin/ffmpeg",
            "/opt/homebrew/bin/ffmpeg",
        ];
        
        for path in &common_paths {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }
        
        None
    }
    
    /// 動画をH.265にエンコードします
    pub fn encode_to_h265(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &EncodingOptions,
    ) -> Result<(), EncodingError> {
        // FFmpegコマンドを構築
        let mut cmd = Command::new(&self.ffmpeg_path);
        
        // 入力ファイル
        cmd.arg("-i").arg(input_path);
        
        // ビデオコーデック
        let video_codec = if options.hardware_accel {
            if let Some(hw_codec) = VideoFormat::H265.hw_codec_string() {
                hw_codec
            } else {
                VideoFormat::H265.codec_string()
            }
        } else {
            VideoFormat::H265.codec_string()
        };
        
        cmd.arg("-c:v").arg(video_codec);
        
        // プリセット
        if !options.hardware_accel || video_codec == "libx265" {
            cmd.arg("-preset").arg(options.preset.to_string());
        }
        
        // 品質設定（CRF優先、なければビットレート）
        if let Some(crf) = options.crf {
            cmd.arg("-crf").arg(crf.to_string());
        } else if let Some(bitrate) = options.bitrate {
            cmd.arg("-b:v").arg(format!("{}k", bitrate / 1000));
        }
        
        // 解像度
        if let Some((width, height)) = options.resolution {
            cmd.arg("-s").arg(format!("{}x{}", width, height));
        }
        
        // フレームレート
        if let Some(fps) = options.framerate {
            cmd.arg("-r").arg(fps.to_string());
        }
        
        // 音声設定
        if let Some(audio_bitrate) = options.audio_bitrate {
            cmd.arg("-c:a").arg("aac");
            cmd.arg("-b:a").arg(format!("{}k", audio_bitrate / 1000));
        } else {
            cmd.arg("-c:a").arg("copy"); // 音声はコピー
        }
        
        if let Some(channels) = options.audio_channels {
            cmd.arg("-ac").arg(channels.to_string());
        }
        
        // メタデータ
        for (key, value) in &options.metadata {
            cmd.arg("-metadata").arg(format!("{}={}", key, value));
        }
        
        // 出力フォーマット
        cmd.arg("-f").arg("mp4");
        
        // 高速スタート用のオプション
        cmd.arg("-movflags").arg("faststart");
        
        // 出力ファイル（上書き）
        cmd.arg("-y").arg(output_path);
        
        // FFmpegを実行
        info!("FFmpegコマンド: {:?}", cmd);
        
        let output = cmd.output()
            .map_err(|e| EncodingError::EncodingFailed(format!("FFmpegの実行に失敗しました: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EncodingError::EncodingFailed(format!("FFmpegの実行に失敗しました: {}", stderr)));
        }
        
        Ok(())
    }
    
    /// 動画を指定されたフォーマットにエンコードします
    pub fn encode_video(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &EncodingOptions,
    ) -> Result<(), EncodingError> {
        // エンコードするファイルが存在するかチェック
        if !input_path.exists() {
            return Err(EncodingError::FileError(format!("入力ファイルが見つかりません: {}", input_path.display())));
        }
        
        // 出力ディレクトリが存在するかチェック
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| EncodingError::FileError(format!("出力ディレクトリの作成に失敗しました: {}", e)))?;
            }
        }
        
        // フォーマットに応じたエンコード処理
        match options.format {
            VideoFormat::H265 => self.encode_to_h265(input_path, output_path, options),
            VideoFormat::H264 => self.encode_to_h264(input_path, output_path, options),
            VideoFormat::VP9 => self.encode_to_vp9(input_path, output_path, options),
            VideoFormat::AV1 => self.encode_to_av1(input_path, output_path, options),
        }
    }
    
    /// 動画をH.264にエンコードします
    fn encode_to_h264(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &EncodingOptions,
    ) -> Result<(), EncodingError> {
        // FFmpegコマンドを構築
        let mut cmd = Command::new(&self.ffmpeg_path);
        
        // 入力ファイル
        cmd.arg("-i").arg(input_path);
        
        // ビデオコーデック
        let video_codec = if options.hardware_accel {
            if let Some(hw_codec) = VideoFormat::H264.hw_codec_string() {
                hw_codec
            } else {
                VideoFormat::H264.codec_string()
            }
        } else {
            VideoFormat::H264.codec_string()
        };
        
        cmd.arg("-c:v").arg(video_codec);
        
        // プリセット
        if !options.hardware_accel || video_codec == "libx264" {
            cmd.arg("-preset").arg(options.preset.to_string());
        }
        
        // 品質設定（CRF優先、なければビットレート）
        if let Some(crf) = options.crf {
            cmd.arg("-crf").arg(crf.to_string());
        } else if let Some(bitrate) = options.bitrate {
            cmd.arg("-b:v").arg(format!("{}k", bitrate / 1000));
        }
        
        // 解像度
        if let Some((width, height)) = options.resolution {
            cmd.arg("-s").arg(format!("{}x{}", width, height));
        }
        
        // フレームレート
        if let Some(fps) = options.framerate {
            cmd.arg("-r").arg(fps.to_string());
        }
        
        // 音声設定
        if let Some(audio_bitrate) = options.audio_bitrate {
            cmd.arg("-c:a").arg("aac");
            cmd.arg("-b:a").arg(format!("{}k", audio_bitrate / 1000));
        } else {
            cmd.arg("-c:a").arg("copy"); // 音声はコピー
        }
        
        if let Some(channels) = options.audio_channels {
            cmd.arg("-ac").arg(channels.to_string());
        }
        
        // メタデータ
        for (key, value) in &options.metadata {
            cmd.arg("-metadata").arg(format!("{}={}", key, value));
        }
        
        // 出力フォーマット
        cmd.arg("-f").arg("mp4");
        
        // 高速スタート用のオプション
        cmd.arg("-movflags").arg("faststart");
        
        // 出力ファイル（上書き）
        cmd.arg("-y").arg(output_path);
        
        // FFmpegを実行
        info!("FFmpegコマンド: {:?}", cmd);
        
        let output = cmd.output()
            .map_err(|e| EncodingError::EncodingFailed(format!("FFmpegの実行に失敗しました: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EncodingError::EncodingFailed(format!("FFmpegの実行に失敗しました: {}", stderr)));
        }
        
        Ok(())
    }
    
    /// 動画をVP9にエンコードします
    fn encode_to_vp9(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &EncodingOptions,
    ) -> Result<(), EncodingError> {
        // FFmpegコマンドを構築
        let mut cmd = Command::new(&self.ffmpeg_path);
        
        // 入力ファイル
        cmd.arg("-i").arg(input_path);
        
        // ビデオコーデック
        cmd.arg("-c:v").arg(VideoFormat::VP9.codec_string());
        
        // 品質設定（CRF優先、なければビットレート）
        if let Some(crf) = options.crf {
            cmd.arg("-crf").arg(crf.to_string());
            cmd.arg("-b:v").arg("0"); // CRFモードでは可変ビットレートを使用
        } else if let Some(bitrate) = options.bitrate {
            cmd.arg("-b:v").arg(format!("{}k", bitrate / 1000));
        }
        
        // 2パスエンコード（VP9では推奨）
        cmd.arg("-pass").arg("1");
        cmd.arg("-passlogfile").arg(format!("{}.log", output_path.display()));
        
        // 解像度
        if let Some((width, height)) = options.resolution {
            cmd.arg("-s").arg(format!("{}x{}", width, height));
        }
        
        // フレームレート
        if let Some(fps) = options.framerate {
            cmd.arg("-r").arg(fps.to_string());
        }
        
        // 音声設定
        if let Some(audio_bitrate) = options.audio_bitrate {
            cmd.arg("-c:a").arg("libopus");
            cmd.arg("-b:a").arg(format!("{}k", audio_bitrate / 1000));
        } else {
            cmd.arg("-c:a").arg("libopus");
            cmd.arg("-b:a").arg("128k");
        }
        
        if let Some(channels) = options.audio_channels {
            cmd.arg("-ac").arg(channels.to_string());
        }
        
        // メタデータ
        for (key, value) in &options.metadata {
            cmd.arg("-metadata").arg(format!("{}={}", key, value));
        }
        
        // 出力フォーマット
        cmd.arg("-f").arg("webm");
        
        // 出力ファイル（上書き）
        cmd.arg("-y").arg(output_path);
        
        // FFmpegを実行
        info!("FFmpegコマンド: {:?}", cmd);
        
        let output = cmd.output()
            .map_err(|e| EncodingError::EncodingFailed(format!("FFmpegの実行に失敗しました: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EncodingError::EncodingFailed(format!("FFmpegの実行に失敗しました: {}", stderr)));
        }
        
        Ok(())
    }
    
    /// 動画をAV1にエンコードします
    fn encode_to_av1(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &EncodingOptions,
    ) -> Result<(), EncodingError> {
        // FFmpegコマンドを構築
        let mut cmd = Command::new(&self.ffmpeg_path);
        
        // 入力ファイル
        cmd.arg("-i").arg(input_path);
        
        // ビデオコーデック
        cmd.arg("-c:v").arg(VideoFormat::AV1.codec_string());
        
        // AV1は非常に低速なため、低速度・高速度設定
        cmd.arg("-cpu-used").arg("8"); // 0-8, 値が大きいほど高速だが低品質
        
        // 品質設定（CRF優先、なければビットレート）
        if let Some(crf) = options.crf {
            cmd.arg("-crf").arg(crf.to_string());
        } else if let Some(bitrate) = options.bitrate {
            cmd.arg("-b:v").arg(format!("{}k", bitrate / 1000));
        }
        
        // 解像度
        if let Some((width, height)) = options.resolution {
            cmd.arg("-s").arg(format!("{}x{}", width, height));
        }
        
        // フレームレート
        if let Some(fps) = options.framerate {
            cmd.arg("-r").arg(fps.to_string());
        }
        
        // 音声設定
        if let Some(audio_bitrate) = options.audio_bitrate {
            cmd.arg("-c:a").arg("libopus");
            cmd.arg("-b:a").arg(format!("{}k", audio_bitrate / 1000));
        } else {
            cmd.arg("-c:a").arg("libopus");
            cmd.arg("-b:a").arg("128k");
        }
        
        if let Some(channels) = options.audio_channels {
            cmd.arg("-ac").arg(channels.to_string());
        }
        
        // メタデータ
        for (key, value) in &options.metadata {
            cmd.arg("-metadata").arg(format!("{}={}", key, value));
        }
        
        // 出力フォーマット
        cmd.arg("-f").arg("mp4");
        
        // 出力ファイル（上書き）
        cmd.arg("-y").arg(output_path);
        
        // FFmpegを実行
        info!("FFmpegコマンド: {:?}", cmd);
        
        let output = cmd.output()
            .map_err(|e| EncodingError::EncodingFailed(format!("FFmpegの実行に失敗しました: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EncodingError::EncodingFailed(format!("FFmpegの実行に失敗しました: {}", stderr)));
        }
        
        Ok(())
    }
    
    /// 動画情報を取得します
    pub fn get_video_info(&self, video_path: &Path) -> Result<VideoInfo, EncodingError> {
        // FFprobeコマンドを構築
        let ffprobe_path = self.ffmpeg_path.parent().unwrap_or(Path::new("/usr/bin")).join("ffprobe");
        
        let output = Command::new(ffprobe_path)
            .arg("-v").arg("quiet")
            .arg("-print_format").arg("json")
            .arg("-show_format")
            .arg("-show_streams")
            .arg(video_path)
            .output()
            .map_err(|e| EncodingError::Other(format!("FFprobeの実行に失敗しました: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EncodingError::Other(format!("FFprobeの実行に失敗しました: {}", stderr)));
        }
        
        // JSON出力を解析
        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| EncodingError::Other(format!("FFprobe出力の解析に失敗しました: {}", e)))?;
        
        // ビデオストリーム情報の抽出
        let streams = json["streams"].as_array().unwrap_or(&Vec::new());
        let mut video_stream = None;
        let mut audio_stream = None;
        
        for stream in streams {
            let codec_type = stream["codec_type"].as_str().unwrap_or("");
            
            if codec_type == "video" && video_stream.is_none() {
                video_stream = Some(stream);
            } else if codec_type == "audio" && audio_stream.is_none() {
                audio_stream = Some(stream);
            }
        }
        
        // ビデオ情報の構築
        let mut info = VideoInfo {
            width: 0,
            height: 0,
            duration: 0.0,
            bitrate: 0,
            codec: String::new(),
            framerate: 0.0,
            audio_codec: None,
            audio_channels: None,
            audio_sample_rate: None,
            format: String::new(),
            filesize: fs::metadata(video_path)
                .map(|m| m.len())
                .unwrap_or(0),
        };
        
        // ビデオストリーム情報
        if let Some(stream) = video_stream {
            info.width = stream["width"].as_u64().unwrap_or(0) as u32;
            info.height = stream["height"].as_u64().unwrap_or(0) as u32;
            info.codec = stream["codec_name"].as_str().unwrap_or("unknown").to_string();
            
            // フレームレートの解析
            if let (Some(num), Some(den)) = (
                stream["r_frame_rate"].as_str().and_then(|s| s.split('/').next().and_then(|n| n.parse::<f64>().ok())),
                stream["r_frame_rate"].as_str().and_then(|s| s.split('/').nth(1).and_then(|n| n.parse::<f64>().ok()))
            ) {
                if den > 0.0 {
                    info.framerate = num / den;
                }
            }
        }
        
        // 音声ストリーム情報
        if let Some(stream) = audio_stream {
            info.audio_codec = Some(stream["codec_name"].as_str().unwrap_or("unknown").to_string());
            info.audio_channels = stream["channels"].as_u64().map(|c| c as u8);
            info.audio_sample_rate = stream["sample_rate"].as_str()
                .and_then(|s| s.parse::<u32>().ok());
        }
        
        // フォーマット情報
        if let Some(format) = json["format"].as_object() {
            info.format = format.get("format_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
                
            info.duration = format.get("duration")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
                
            info.bitrate = format.get("bit_rate")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
        }
        
        Ok(info)
    }
}

/// 動画情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    /// 幅（ピクセル単位）
    pub width: u32,
    /// 高さ（ピクセル単位）
    pub height: u32,
    /// 動画時間（秒）
    pub duration: f64,
    /// ビットレート（bps）
    pub bitrate: u64,
    /// ビデオコーデック（例: "h264"）
    pub codec: String,
    /// フレームレート（fps）
    pub framerate: f64,
    /// 音声コーデック（例: "aac"）
    pub audio_codec: Option<String>,
    /// 音声チャンネル数
    pub audio_channels: Option<u8>,
    /// 音声サンプルレート（Hz）
    pub audio_sample_rate: Option<u32>,
    /// コンテナフォーマット（例: "mp4"）
    pub format: String,
    /// ファイルサイズ（バイト）
    pub filesize: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    #[ignore] // 実際のFFmpegに依存するためCI環境では無視
    fn test_find_ffmpeg() {
        if let Ok(manager) = EncodingManager::new() {
            assert!(manager.ffmpeg_path.exists());
        }
    }
}