//! # C FFI モジュール
//! 
//! NextDownloaderのC FFIバインディングを提供します。
//! このモジュールは、C言語やSwift、Objective-Cなどから
//! NextDownloaderのコア機能を利用するためのインターフェースを提供します。

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};
use std::path::PathBuf;
use std::ptr;

use log::{debug, error, info, warn};

use nextdownloader_core::{
    ContentType, DownloadManager, DownloadOptions, DownloadStatus, ErrorCode,
};

use crate::DOWNLOAD_MANAGER;

/// FFIレイヤーを初期化します
#[no_mangle]
pub extern "C" fn nd_initialize() {
    crate::initialize();
}

/// ダウンロードマネージャーのインスタンスを作成します
#[no_mangle]
pub extern "C" fn nd_create_download_manager() -> *mut DownloadManager {
    let manager = Box::new(DownloadManager::new());
    Box::into_raw(manager)
}

/// ダウンロードマネージャーのインスタンスを破棄します
#[no_mangle]
pub extern "C" fn nd_destroy_download_manager(ptr: *mut DownloadManager) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

/// URLからコンテンツタイプを検出します
#[no_mangle]
pub extern "C" fn nd_detect_content_type(url: *const c_char) -> c_int {
    if url.is_null() {
        return ContentType::Unknown as c_int;
    }
    
    let url_str = unsafe {
        match CStr::from_ptr(url).to_str() {
            Ok(s) => s,
            Err(_) => return ContentType::Unknown as c_int,
        }
    };
    
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    match manager.detect_content_type(url_str) {
        Ok(content_type) => content_type as c_int,
        Err(_) => ContentType::Unknown as c_int,
    }
}

/// ダウンロードを開始します
#[no_mangle]
pub extern "C" fn nd_start_download(
    url: *const c_char,
    destination: *const c_char,
    format: *const c_char,
    download_id_out: *mut c_char,
    download_id_len: c_int,
) -> c_int {
    if url.is_null() || destination.is_null() || download_id_out.is_null() {
        return ErrorCode::InvalidUrl as c_int;
    }
    
    let url_str = unsafe {
        match CStr::from_ptr(url).to_str() {
            Ok(s) => s,
            Err(_) => return ErrorCode::InvalidUrl as c_int,
        }
    };
    
    let destination_str = unsafe {
        match CStr::from_ptr(destination).to_str() {
            Ok(s) => s,
            Err(_) => return ErrorCode::FileSystemError as c_int,
        }
    };
    
    let format_str = if format.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(format).to_str() {
                Ok(s) => Some(s),
                Err(_) => None,
            }
        }
    };
    
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        manager.start_download(url_str, &PathBuf::from(destination_str), format_str)
    });
    
    match result {
        Ok(download_id) => {
            let c_download_id = match CString::new(download_id) {
                Ok(s) => s,
                Err(_) => return ErrorCode::UnknownError as c_int,
            };
            
            let bytes = c_download_id.as_bytes_with_nul();
            if bytes.len() > download_id_len as usize {
                return ErrorCode::UnknownError as c_int;
            }
            
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    download_id_out as *mut u8,
                    bytes.len(),
                );
            }
            
            0 // 成功
        }
        Err(e) => e.error_code() as c_int,
    }
}

/// ダウンロードの進捗を取得します
#[no_mangle]
pub extern "C" fn nd_get_download_progress(
    download_id: *const c_char,
) -> c_double {
    if download_id.is_null() {
        return 0.0;
    }
    
    let download_id_str = unsafe {
        match CStr::from_ptr(download_id).to_str() {
            Ok(s) => s,
            Err(_) => return 0.0,
        }
    };
    
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        manager.get_download_progress(download_id_str)
    });
    
    match result {
        Ok(progress) => progress as c_double,
        Err(_) => 0.0,
    }
}

/// ダウンロードをキャンセルします
#[no_mangle]
pub extern "C" fn nd_cancel_download(
    download_id: *const c_char,
) -> c_int {
    if download_id.is_null() {
        return ErrorCode::UnknownError as c_int;
    }
    
    let download_id_str = unsafe {
        match CStr::from_ptr(download_id).to_str() {
            Ok(s) => s,
            Err(_) => return ErrorCode::UnknownError as c_int,
        }
    };
    
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        manager.cancel_download(download_id_str)
    });
    
    match result {
        Ok(_) => 0, // 成功
        Err(e) => e.error_code() as c_int,
    }
}

/// 依存関係をチェックします
#[no_mangle]
pub extern "C" fn nd_check_dependencies(
    ytdlp_out: *mut c_int,
    aria2c_out: *mut c_int,
    ffmpeg_out: *mut c_int,
) -> c_int {
    if ytdlp_out.is_null() || aria2c_out.is_null() || ffmpeg_out.is_null() {
        return ErrorCode::UnknownError as c_int;
    }
    
    let manager = DOWNLOAD_MANAGER.lock().unwrap();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        manager.check_dependencies()
    });
    
    match result {
        Ok(status) => {
            unsafe {
                *ytdlp_out = if status.ytdlp { 1 } else { 0 };
                *aria2c_out = if status.aria2c { 1 } else { 0 };
                *ffmpeg_out = if status.ffmpeg { 1 } else { 0 };
            }
            0 // 成功
        }
        Err(e) => e.error_code() as c_int,
    }
}

/// NextDownloaderのバージョンを取得します
#[no_mangle]
pub extern "C" fn nd_get_version() -> *const c_char {
    let version = nextdownloader_core::version();
    let c_version = CString::new(version).unwrap();
    c_version.into_raw()
}

/// 文字列リソースを解放します
#[no_mangle]
pub extern "C" fn nd_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}