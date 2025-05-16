use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use crate::{DownloadManager, Downloader, DownloadOptions, ContentType, VideoFormat, ErrorCode};

// FFI用のエクスポート関数

/// システムの依存関係をチェックする
/// 
/// # 戻り値
/// 
/// 3バイトの配列。順に yt-dlp, aria2c, ffmpegが利用可能かどうかを示す。
/// 1なら利用可能、0なら利用不可。
#[no_mangle]
pub extern "C" fn check_dependencies(result: *mut [u8; 3]) -> libc::c_int {
    let download_manager = DownloadManager::new();
    
    // 非同期関数を同期的に実行
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (ytdlp, aria2c, ffmpeg) = rt.block_on(download_manager.check_dependencies());
    
    if !result.is_null() {
        unsafe {
            (*result)[0] = if ytdlp { 1 } else { 0 };
            (*result)[1] = if aria2c { 1 } else { 0 };
            (*result)[2] = if ffmpeg { 1 } else { 0 };
        }
    }
    
    ErrorCode::Success as libc::c_int
}

/// URLからコンテンツタイプを検出する
/// 
/// # 引数
/// 
/// * `url` - 検出するURL
/// * `result` - 結果を格納するポインタ (0: MP4, 1: HLS, 2: DASH, 3: YouTube, 4: Unknown)
/// 
/// # 戻り値
/// 
/// エラーコード
#[no_mangle]
pub extern "C" fn detect_content_type(
    url: *const c_char,
    result: *mut libc::c_int
) -> libc::c_int {
    if url.is_null() || result.is_null() {
        return ErrorCode::UnknownError as libc::c_int;
    }
    
    let c_url = unsafe { CStr::from_ptr(url) };
    let url_str = match c_url.to_str() {
        Ok(s) => s,
        Err(_) => return ErrorCode::UnknownError as libc::c_int,
    };
    
    let download_manager = DownloadManager::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    match rt.block_on(download_manager.detect_content_type(url_str)) {
        Ok(content_type) => {
            unsafe {
                *result = match content_type {
                    ContentType::Mp4 => 0,
                    ContentType::Hls => 1,
                    ContentType::Dash => 2,
                    ContentType::YouTube => 3,
                    ContentType::Unknown => 4,
                };
            }
            ErrorCode::Success as libc::c_int
        },
        Err(_) => {
            unsafe { *result = 4 }; // Unknown
            ErrorCode::UnknownError as libc::c_int
        }
    }
}

/// URLからビデオ情報を取得する
/// 
/// # 引数
/// 
/// * `url` - 取得するURL
/// * `json_result` - JSON形式の結果を格納するポインタ (呼び出し側で解放する必要あり)
/// 
/// # 戻り値
/// 
/// エラーコード
#[no_mangle]
pub extern "C" fn get_video_info(
    url: *const c_char,
    json_result: *mut *mut c_char
) -> libc::c_int {
    if url.is_null() || json_result.is_null() {
        return ErrorCode::UnknownError as libc::c_int;
    }
    
    let c_url = unsafe { CStr::from_ptr(url) };
    let url_str = match c_url.to_str() {
        Ok(s) => s,
        Err(_) => return ErrorCode::UnknownError as libc::c_int,
    };
    
    let download_manager = DownloadManager::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    match rt.block_on(download_manager.ytdlp.get_video_info(url_str)) {
        Ok(video_info) => {
            match serde_json::to_string(&video_info) {
                Ok(json) => {
                    let c_json = match CString::new(json) {
                        Ok(s) => s,
                        Err(_) => return ErrorCode::UnknownError as libc::c_int,
                    };
                    
                    unsafe {
                        *json_result = c_json.into_raw();
                    }
                    
                    ErrorCode::Success as libc::c_int
                },
                Err(_) => ErrorCode::JsonError as libc::c_int,
            }
        },
        Err(_) => ErrorCode::ProcessFailed as libc::c_int,
    }
}

/// URLからビデオをダウンロードする
/// 
/// # 引数
/// 
/// * `url` - ダウンロードするURL
/// * `output_path` - 出力先ディレクトリ
/// * `filename` - ファイル名
/// * `options_json` - オプションのJSON文字列 (NULLの場合はデフォルト値を使用)
/// * `result_path` - 結果のファイルパスを格納するポインタ (呼び出し側で解放する必要あり)
/// 
/// # 戻り値
/// 
/// エラーコード
#[no_mangle]
pub extern "C" fn download_video(
    url: *const c_char,
    output_path: *const c_char,
    filename: *const c_char,
    options_json: *const c_char,
    result_path: *mut *mut c_char
) -> libc::c_int {
    if url.is_null() || output_path.is_null() || filename.is_null() || result_path.is_null() {
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
    
    // オプションのJSONがあれば解析
    let options = if !options_json.is_null() {
        let c_options_json = unsafe { CStr::from_ptr(options_json) };
        let options_json_str = match c_options_json.to_str() {
            Ok(s) => s,
            Err(_) => return ErrorCode::UnknownError as libc::c_int,
        };
        
        match serde_json::from_str::<DownloadOptions>(options_json_str) {
            Ok(opts) => Some(opts),
            Err(_) => None,
        }
    } else {
        None
    };
    
    let download_manager = DownloadManager::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    match rt.block_on(download_manager.download(
        url_str,
        &PathBuf::from(output_path_str),
        filename_str,
        options,
        None
    )) {
        Ok(path) => {
            let path_str = path.to_string_lossy().to_string();
            let c_path = match CString::new(path_str) {
                Ok(s) => s,
                Err(_) => return ErrorCode::UnknownError as libc::c_int,
            };
            
            unsafe {
                *result_path = c_path.into_raw();
            }
            
            ErrorCode::Success as libc::c_int
        },
        Err(err) => {
            match err {
                crate::DownloadError::FileNotFound => ErrorCode::FileNotFound as libc::c_int,
                crate::DownloadError::ProcessFailed(_) => ErrorCode::ProcessFailed as libc::c_int,
                crate::DownloadError::Io(_) => ErrorCode::IoError as libc::c_int,
                crate::DownloadError::Json(_) => ErrorCode::JsonError as libc::c_int,
                _ => ErrorCode::UnknownError as libc::c_int,
            }
        }
    }
}

/// C文字列を解放する
/// 
/// # 引数
/// 
/// * `ptr` - 解放するC文字列ポインタ
#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
