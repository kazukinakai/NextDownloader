//! # コンテンツタイプモジュール
//! 
//! ダウンロード対象のコンテンツタイプを定義します。

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// ダウンロード対象のコンテンツタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// MP4ファイル
    MP4,
    /// HLS (HTTP Live Streaming)
    HLS,
    /// DASH (Dynamic Adaptive Streaming over HTTP)
    DASH,
    /// YouTube
    YouTube,
    /// DRM保護されたコンテンツ
    DrmProtected,
    /// 不明なコンテンツタイプ
    Unknown,
}

impl ContentType {
    /// URLからコンテンツタイプを検出します
    pub fn detect_from_url(url: &str) -> Self {
        // YouTubeのURLパターン
        if url.contains("youtube.com") || url.contains("youtu.be") {
            return ContentType::YouTube;
        }
        
        // ファイル拡張子によるコンテンツタイプの判定
        if url.ends_with(".mp4") {
            return ContentType::MP4;
        }
        
        // HLSのURLパターン
        if url.ends_with(".m3u8") {
            return ContentType::HLS;
        }
        
        // DASHのURLパターン
        if url.ends_with(".mpd") {
            return ContentType::DASH;
        }
        
        // その他の場合は不明として扱う
        ContentType::Unknown
    }
    
    /// コンテンツタイプが動画かどうかを判定します
    pub fn is_video(&self) -> bool {
        match self {
            ContentType::MP4 | ContentType::HLS | ContentType::DASH | ContentType::YouTube | ContentType::DrmProtected => true,
            ContentType::Unknown => false,
        }
    }
    
    /// コンテンツタイプがストリーミングかどうかを判定します
    pub fn is_streaming(&self) -> bool {
        match self {
            ContentType::HLS | ContentType::DASH => true,
            _ => false,
        }
    }
    
    /// コンテンツタイプがDRM保護されているかどうかを判定します
    pub fn is_drm_protected(&self) -> bool {
        match self {
            ContentType::DrmProtected => true,
            _ => false,
        }
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::MP4 => write!(f, "MP4"),
            ContentType::HLS => write!(f, "HLS"),
            ContentType::DASH => write!(f, "DASH"),
            ContentType::YouTube => write!(f, "YouTube"),
            ContentType::DrmProtected => write!(f, "DrmProtected"),
            ContentType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl FromStr for ContentType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mp4" => Ok(ContentType::MP4),
            "hls" => Ok(ContentType::HLS),
            "dash" => Ok(ContentType::DASH),
            "youtube" => Ok(ContentType::YouTube),
            "drm" | "drmprotected" => Ok(ContentType::DrmProtected),
            "unknown" => Ok(ContentType::Unknown),
            _ => Err(format!("Invalid content type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_from_url() {
        assert_eq!(ContentType::detect_from_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"), ContentType::YouTube);
        assert_eq!(ContentType::detect_from_url("https://youtu.be/dQw4w9WgXcQ"), ContentType::YouTube);
        assert_eq!(ContentType::detect_from_url("https://example.com/video.mp4"), ContentType::MP4);
        assert_eq!(ContentType::detect_from_url("https://example.com/stream.m3u8"), ContentType::HLS);
        assert_eq!(ContentType::detect_from_url("https://example.com/stream.mpd"), ContentType::DASH);
        assert_eq!(ContentType::detect_from_url("https://example.com/file.txt"), ContentType::Unknown);
    }
    
    #[test]
    fn test_is_video() {
        assert!(ContentType::MP4.is_video());
        assert!(ContentType::HLS.is_video());
        assert!(ContentType::DASH.is_video());
        assert!(ContentType::YouTube.is_video());
        assert!(!ContentType::Unknown.is_video());
    }
    
    #[test]
    fn test_display() {
        assert_eq!(ContentType::MP4.to_string(), "MP4");
        assert_eq!(ContentType::HLS.to_string(), "HLS");
        assert_eq!(ContentType::DASH.to_string(), "DASH");
        assert_eq!(ContentType::YouTube.to_string(), "YouTube");
        assert_eq!(ContentType::Unknown.to_string(), "Unknown");
    }
    
    #[test]
    fn test_from_str() {
        assert_eq!("mp4".parse::<ContentType>().unwrap(), ContentType::MP4);
        assert_eq!("hls".parse::<ContentType>().unwrap(), ContentType::HLS);
        assert_eq!("dash".parse::<ContentType>().unwrap(), ContentType::DASH);
        assert_eq!("youtube".parse::<ContentType>().unwrap(), ContentType::YouTube);
        assert_eq!("unknown".parse::<ContentType>().unwrap(), ContentType::Unknown);
        assert!("invalid".parse::<ContentType>().is_err());
    }
}