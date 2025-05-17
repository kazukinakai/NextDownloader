use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use std::time::Duration;
use url::Url;

/// ユーティリティ関数を提供するモジュール

/// 実行可能ファイルのパスを取得します。
/// 
/// 環境変数PATHからのパスの検索、または現在の実行ファイルの
/// ディレクトリからの相対パスを試みます。
pub async fn find_executable(name: &str) -> Option<PathBuf> {
    // 環境変数PATHからの検索
    if let Ok(paths) = std::env::var("PATH") {
        let paths: Vec<_> = std::env::split_paths(&paths).collect();
        for path in paths {
            let full_path = path.join(name);
            if full_path.exists() && is_executable(&full_path) {
                return Some(full_path);
            }
            
            // Windows用の.exe拡張子を追加
            #[cfg(target_os = "windows")]
            {
                let exe_path = path.join(format!("{}.exe", name));
                if exe_path.exists() && is_executable(&exe_path) {
                    return Some(exe_path);
                }
            }
        }
    }
    
    // 現在の実行ファイルからの相対パスを試行
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // binディレクトリを確認
            let bin_path = exe_dir.join("bin").join(name);
            if bin_path.exists() && is_executable(&bin_path) {
                return Some(bin_path);
            }
            
            // Windows用の.exe拡張子を追加
            #[cfg(target_os = "windows")]
            {
                let bin_exe_path = exe_dir.join("bin").join(format!("{}.exe", name));
                if bin_exe_path.exists() && is_executable(&bin_exe_path) {
                    return Some(bin_exe_path);
                }
            }
        }
    }
    
    None
}

/// 指定されたパスが実行可能ファイルかどうかを確認します。
fn is_executable(path: &PathBuf) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let permissions = metadata.permissions();
            // 所有者実行権限をチェック
            return permissions.mode() & 0o100 != 0;
        }
    }
    
    #[cfg(windows)]
    {
        // Windowsでは拡張子で判断
        if let Some(ext) = path.extension() {
            return ext.to_string_lossy().to_lowercase() == "exe";
        }
    }
    
    false
}

/// バイナリデータをファイルに保存します。
pub async fn save_binary_file(path: &PathBuf, data: &[u8]) -> std::io::Result<()> {
    // 親ディレクトリが存在しない場合は作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    let mut file = File::create(path).await?;
    file.write_all(data).await?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = file.metadata().await?;
        let mut permissions = metadata.permissions();
        // 実行権限を追加
        permissions.set_mode(permissions.mode() | 0o100);
        fs::set_permissions(path, permissions).await?;
    }
    
    Ok(())
}

/// 指定されたURLからファイルをダウンロードして保存します。
pub async fn download_file(url: &str, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // reqwestを使ってダウンロード
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    
    save_binary_file(path, &bytes).await?;
    
    Ok(())
}

/// 指定されたツールをダウンロードしてインストールします。
pub async fn install_tool(
    name: &str, 
    url: &str,
    install_dir: &PathBuf
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let file_name = url.split('/').last().unwrap_or(name);
    let install_path = install_dir.join(file_name);
    
    // ディレクトリ作成
    if let Some(parent) = install_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    // ダウンロードして保存
    download_file(url, &install_path).await?;
    
    Ok(install_path)
}

/// ディレクトリが存在することを確認し、存在しない場合は作成します。
pub async fn ensure_dir_exists(dir: &PathBuf) -> std::io::Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir).await?
    } else if !dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{:?} はディレクトリではありません", dir)
        ));
    }
    
    Ok(())
}

/// リトライ可能な非同期処理を実行します。
pub async fn with_retry<F, Fut, T, E>(
    f: F,
    retries: u32,
    delay: Duration
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempts = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                attempts += 1;
                if attempts >= retries {
                    return Err(err);
                }
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// URLからベースURLを取得します。
pub fn get_base_url(url_str: &str) -> String {
    if let Ok(url) = Url::parse(url_str) {
        let path = url.path();
        let last_slash_pos = path.rfind('/').unwrap_or(0);
        let base_path = &path[..=last_slash_pos];
        
        let mut base_url = url.clone();
        base_url.set_path(base_path);
        base_url.to_string()
    } else {
        let last_slash_pos = url_str.rfind('/').unwrap_or(0);
        url_str[..=last_slash_pos].to_string()
    }
}
