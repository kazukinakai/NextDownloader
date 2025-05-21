//! # ストリーミングモジュール
//! 
//! HLS/DASHなどのストリーミングプロトコルをサポートするためのモジュールです。

pub mod hls;
pub mod dash;
pub mod common;

pub use hls::HlsDownloader;
pub use dash::DashDownloader;
pub use common::{StreamSegment, StreamingOptions};