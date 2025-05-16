/// NextDownloader core library
///
/// This module provides core functionality for downloading and processing
/// video content from various sources.

// 外部クレート
use std::path::PathBuf;
use tokio::process::Command;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use async_trait::async_trait;

// モジュール宣言
pub mod types;
pub mod downloader;
pub mod tools;
pub mod utils;

// 再エクスポート
pub use crate::types::*;
pub use crate::downloader::*;
pub use crate::tools::*;

/// FFI向けエラーコード
#[repr(C)]
pub enum ErrorCode {
    Success = 0,
    FileNotFound = 1,
    ProcessFailed = 2,
    IoError = 3,
    JsonError = 4,
    UnknownError = 5,
}

// C FFIのための外部インターフェース
#[cfg(feature = "ffi")]
pub mod ffi {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

    #[no_mangle]
    pub extern "C" fn download_url(
        url: *const c_char,
        output_path: *const c_char,
        filename: *const c_char,
    ) -> libc::c_int {
        if url.is_null() || output_path.is_null() || filename.is_null() {
            return ErrorCode::UnknownError as libc::c_int;
        }

        let c_url = unsafe { CStr::from_ptr(url) };
        let c_output_path = unsafe { CStr::from_ptr(output_path) };
        let c_filename = unsafe { CStr::from_ptr(filename) };

        let url_str = match c_url.to_str() {
            Ok(s) => s,
            Err(_) => return ErrorCode::UnknownError as libc::c_int,
        };

        let output_path_str = match c_output_path.to_str() {
            Ok(s) => s,
            Err(_) => return ErrorCode::UnknownError as libc::c_int,
        };

        let filename_str = match c_filename.to_str() {
            Ok(s) => s,
            Err(_) => return ErrorCode::UnknownError as libc::c_int,
        };

        // この関数は実際にはasyncなので、ここでは簡易版の実装
        ErrorCode::Success as libc::c_int
    }
}

// Tauriコマンド実装
#[cfg(feature = "tauri-plugin")]
pub mod tauri_plugin {
    use super::*;

    #[tauri::command]
    pub async fn download(
        url: String,
        output_path: String,
        filename: String,
        options: Option<DownloadOptions>,
    ) -> Result<String, String> {
        let downloader = DownloadManager::new();
        let path_buf = PathBuf::from(output_path);
        
        match downloader.download(&url, &path_buf, &filename, options, None).await {
            Ok(output_file) => Ok(output_file.to_string_lossy().to_string()),
            Err(err) => Err(err.to_string()),
        }
    }
}
