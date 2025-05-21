//! # H.265変換モジュール
//! 
//! ダウンロード後のメディアファイルをH.265に変換する機能を提供します。

use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;

use log::{debug, error, info, warn};
use anyhow::{Context, Result};

use crate::encoding::{EncodingManager, EncodingOptions, VideoFormat, VideoInfo};
use crate::error::DownloaderError;

/// H.265変換処理を行うクラス
pub struct H265Converter {
    /// エンコーディングマネージャー
    encoding_manager: EncodingManager,
}

impl H265Converter {
    /// 新しいH.265変換器を作成します
    pub fn new() -> Result<Self, DownloaderError> {
        let encoding_manager = EncodingManager::new()
            .map_err(|e| DownloaderError::DependencyError(format!("エンコーディングマネージャーの初期化に失敗しました: {}", e)))?;
        
        Ok(Self {
            encoding_manager,
        })
    }
    
    /// 動画ファイルをH.265に変換します
    pub fn convert_to_h265(
        &self,
        input_path: &Path,
        output_path: Option<&Path>,
        options: &EncodingOptions,
        progress_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
    ) -> Result<PathBuf, DownloaderError> {
        // 入力ファイルの存在確認
        if !input_path.exists() {
            return Err(DownloaderError::FileNotFoundError(format!(
                "変換元ファイルが見つかりません: {}", 
                input_path.display()
            )));
        }
        
        // 出力パスの決定
        let output_path = if let Some(path) = output_path {
            path.to_path_buf()
        } else {
            // 入力パスの拡張子を変更して出力パスとする
            let mut path = input_path.to_path_buf();
            let stem = path.file_stem()
                .ok_or_else(|| DownloaderError::InvalidArgumentError("無効なファイル名です".to_string()))?;
            let new_name = format!("{}_h265.mp4", stem.to_string_lossy());
            path.set_file_name(new_name);
            path
        };
        
        // 出力ディレクトリの作成
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| DownloaderError::FileSystemError(format!(
                        "出力ディレクトリの作成に失敗しました: {}", e
                    )))?;
            }
        }
        
        // 動画情報の取得
        info!("動画情報の取得開始: {}", input_path.display());
        let video_info = self.encoding_manager.get_video_info(input_path)
            .map_err(|e| DownloaderError::EncodingError(format!(
                "動画情報の取得に失敗しました: {}", e
            )))?;
        
        // 既にH.265の場合はコピーのみ
        if video_info.codec.contains("hevc") || video_info.codec.contains("h265") {
            info!("ファイルは既にH.265形式です: {}", input_path.display());
            
            if input_path != output_path {
                info!("ファイルをコピーします: {} -> {}", input_path.display(), output_path.display());
                fs::copy(input_path, &output_path)
                    .map_err(|e| DownloaderError::FileSystemError(format!(
                        "ファイルのコピーに失敗しました: {}", e
                    )))?;
            }
            
            return Ok(output_path);
        }
        
        // H.265エンコーディングの開始
        info!(
            "H.265変換開始: {} -> {}\n解像度: {}x{}, 長さ: {:.2}秒, サイズ: {:.2} MB",
            input_path.display(),
            output_path.display(),
            video_info.width,
            video_info.height,
            video_info.duration,
            video_info.filesize as f64 / 1_048_576.0
        );
        
        // エンコード処理の実行
        let start_time = Instant::now();
        
        let mut encoding_options = options.clone();
        
        // カスタムエンコードオプションの適用
        if encoding_options.resolution.is_none() {
            // 入力ファイルと同じ解像度を使用
            encoding_options.resolution = Some((video_info.width, video_info.height));
        }
        
        // H.265へのエンコード実行
        self.encoding_manager.encode_video(input_path, &output_path, &encoding_options)
            .map_err(|e| DownloaderError::EncodingError(format!(
                "H.265へのエンコードに失敗しました: {}", e
            )))?;
        
        let elapsed = start_time.elapsed();
        
        // エンコード後の情報取得
        if let Ok(encoded_info) = self.encoding_manager.get_video_info(&output_path) {
            let compression_ratio = if video_info.filesize > 0 {
                encoded_info.filesize as f64 / video_info.filesize as f64
            } else {
                0.0
            };
            
            info!(
                "H.265変換完了: {}\n所要時間: {:.2}秒\nサイズ: {:.2} MB -> {:.2} MB (圧縮率: {:.1}%)",
                output_path.display(),
                elapsed.as_secs_f64(),
                video_info.filesize as f64 / 1_048_576.0,
                encoded_info.filesize as f64 / 1_048_576.0,
                compression_ratio * 100.0
            );
        } else {
            info!(
                "H.265変換完了: {}\n所要時間: {:.2}秒",
                output_path.display(),
                elapsed.as_secs_f64()
            );
        }
        
        Ok(output_path)
    }
    
    /// エンコーダーが利用可能かどうかを確認します
    pub fn is_available(&self) -> bool {
        // FFmpegが利用可能かどうかを確認
        true // エンコーディングマネージャーの初期化に成功していれば利用可能
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    #[ignore] // 実際のFFmpegに依存するためCI環境では無視
    fn test_converter_initialization() {
        match H265Converter::new() {
            Ok(converter) => assert!(converter.is_available()),
            Err(_) => panic!("H.265変換器の初期化に失敗しました"),
        }
    }
}