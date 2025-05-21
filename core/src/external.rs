//! # 外部ツール連携モジュール
//! 
//! yt-dlpとaria2cなどの外部ツールとの連携を提供します。

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use std::collections::HashMap;

use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::time::timeout;
use anyhow::{anyhow, Context, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use which::which;

/// 外部ツールエラー
#[derive(Debug, Error)]
pub enum ExternalToolError {
    /// ツールが見つかりません
    #[error("ツールが見つかりません: {0}")]
    ToolNotFound(String),
    
    /// ツールの実行に失敗しました
    #[error("ツールの実行に失敗しました: {0}")]
    ExecutionFailed(String),
    
    /// タイムアウトしました
    #[error("ツールの実行がタイムアウトしました: {0}")]
    Timeout(String),
    
    /// その他のエラー
    #[error("外部ツールエラー: {0}")]
    Other(String),
}

/// 外部ツールの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalTool {
    /// yt-dlp - 動画ダウンロードツール
    YtDlp,
    /// aria2c - 高速ダウンロードツール
    Aria2c,
    /// FFmpeg - エンコーディングツール
    FFmpeg,
}

impl ExternalTool {
    /// 実行ファイル名を返します
    pub fn executable_name(&self) -> &'static str {
        match self {
            ExternalTool::YtDlp => "yt-dlp",
            ExternalTool::Aria2c => "aria2c",
            ExternalTool::FFmpeg => "ffmpeg",
        }
    }
    
    /// インストールコマンドを返します
    pub fn install_command(&self) -> &'static str {
        match self {
            ExternalTool::YtDlp => "pip install yt-dlp",
            ExternalTool::Aria2c => "brew install aria2",
            ExternalTool::FFmpeg => "brew install ffmpeg",
        }
    }
}

/// ytdlp-JSON出力の動画情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtDlpVideoInfo {
    /// 動画ID
    pub id: String,
    /// タイトル
    pub title: String,
    /// 動画の説明
    #[serde(default)]
    pub description: String,
    /// 動画時間（秒）
    pub duration: f64,
    /// サムネイルURL
    #[serde(default)]
    pub thumbnail: String,
    /// フォーマット情報
    #[serde(default)]
    pub formats: Vec<YtDlpFormat>,
    /// アップロード日
    #[serde(default)]
    pub upload_date: String,
    /// チャンネル名
    #[serde(default)]
    pub channel: String,
    /// チャンネルID
    #[serde(default)]
    pub channel_id: String,
    /// 視聴回数
    #[serde(default)]
    pub view_count: Option<i64>,
    /// URL
    pub webpage_url: String,
}

/// ytdlp-JSON出力のフォーマット情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtDlpFormat {
    /// フォーマットID
    pub format_id: String,
    /// フォーマット説明
    pub format: String,
    /// URL
    pub url: String,
    /// 幅（ピクセル）
    pub width: Option<u32>,
    /// 高さ（ピクセル）
    pub height: Option<u32>,
    /// 解像度
    #[serde(default)]
    pub resolution: String,
    /// ビットレート
    pub tbr: Option<f64>,
    /// 拡張子
    #[serde(default)]
    pub ext: String,
    /// ファイルサイズ（バイト）
    pub filesize: Option<u64>,
    /// コンテナ形式
    #[serde(default)]
    pub container: String,
    /// ビデオコーデック
    #[serde(default)]
    pub vcodec: String,
    /// オーディオコーデック
    #[serde(default)]
    pub acodec: String,
}

/// 外部ツール管理マネージャー
pub struct ExternalToolManager {
    /// ツールのパス
    tool_paths: HashMap<ExternalTool, Option<PathBuf>>,
}

impl ExternalToolManager {
    /// 新しい外部ツールマネージャーを作成
    pub fn new() -> Self {
        Self {
            tool_paths: HashMap::new(),
        }
    }
    
    /// ツールが利用可能かどうかを確認
    pub async fn check_tool_available(&mut self, tool: ExternalTool) -> bool {
        if let Some(path_opt) = self.tool_paths.get(&tool) {
            return path_opt.is_some();
        }
        
        let path = which(tool.executable_name()).ok();
        self.tool_paths.insert(tool, path.clone());
        path.is_some()
    }
    
    /// yt-dlpを使用して動画情報を取得
    pub async fn get_video_info(&mut self, url: &str) -> Result<YtDlpVideoInfo, ExternalToolError> {
        // yt-dlpが利用可能か確認
        if !self.check_tool_available(ExternalTool::YtDlp).await {
            return Err(ExternalToolError::ToolNotFound(format!(
                "yt-dlpが見つかりません。インストールするには: {}", 
                ExternalTool::YtDlp.install_command()
            )));
        }
        
        // yt-dlpコマンドを実行して動画情報をJSON形式で取得
        let ytdlp_path = self.tool_paths.get(&ExternalTool::YtDlp)
            .and_then(|p| p.as_ref())
            .ok_or_else(|| ExternalToolError::ToolNotFound("yt-dlpが見つかりません".to_string()))?;
        
        info!("yt-dlpで動画情報を取得中: {}", url);
        
        let output = Command::new(ytdlp_path)
            .arg("--dump-json")
            .arg("--no-playlist")
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ExternalToolError::ExecutionFailed(format!("yt-dlpの実行に失敗しました: {}", e)))?;
        
        let output = timeout(
            Duration::from_secs(60), // 60秒タイムアウト
            output.wait_with_output()
        )
        .await
        .map_err(|_| ExternalToolError::Timeout("yt-dlpがタイムアウトしました".to_string()))?
        .map_err(|e| ExternalToolError::ExecutionFailed(format!("yt-dlpの実行に失敗しました: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExternalToolError::ExecutionFailed(format!(
                "yt-dlpの実行に失敗しました: {}", stderr
            )));
        }
        
        // JSON出力をパース
        let json_output = String::from_utf8_lossy(&output.stdout);
        let video_info: YtDlpVideoInfo = serde_json::from_str(&json_output)
            .map_err(|e| ExternalToolError::Other(format!("JSON解析に失敗しました: {}", e)))?;
        
        Ok(video_info)
    }
    
    /// yt-dlpを使って最高品質の動画をダウンロード
    pub async fn download_best_quality(
        &mut self, 
        url: &str, 
        output_dir: &Path,
        progress_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
    ) -> Result<PathBuf, ExternalToolError> {
        // yt-dlpが利用可能か確認
        if !self.check_tool_available(ExternalTool::YtDlp).await {
            return Err(ExternalToolError::ToolNotFound(format!(
                "yt-dlpが見つかりません。インストールするには: {}", 
                ExternalTool::YtDlp.install_command()
            )));
        }
        
        let ytdlp_path = self.tool_paths.get(&ExternalTool::YtDlp)
            .and_then(|p| p.as_ref())
            .ok_or_else(|| ExternalToolError::ToolNotFound("yt-dlpが見つかりません".to_string()))?;
        
        // 出力ディレクトリが存在することを確認
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)
                .map_err(|e| ExternalToolError::Other(format!("出力ディレクトリの作成に失敗しました: {}", e)))?;
        }
        
        info!("yt-dlpで動画をダウンロード中: {}", url);
        
        // 出力テンプレートを設定
        let output_template = output_dir.join("%(title)s.%(ext)s").to_string_lossy().to_string();
        
        // yt-dlpコマンドの構築
        let mut cmd = Command::new(ytdlp_path);
        cmd.arg("--no-playlist")
           .arg("--restrict-filenames")
           .arg("-o").arg(&output_template)
           .arg("--newline") // 改行ごとに進捗を出力
           .arg("--progress")
           .arg(url)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // 外部ダウンローダーとしてaria2cを使用（利用可能な場合）
        if self.check_tool_available(ExternalTool::Aria2c).await {
            if let Some(aria_path) = self.tool_paths.get(&ExternalTool::Aria2c).and_then(|p| p.as_ref()) {
                cmd.arg("--downloader").arg("aria2c");
                cmd.arg("--downloader-args").arg(format!(
                    "aria2c:'-x 16 -s 16 -k 1M --retry-wait=1 --max-tries=10 --file-allocation=none'"
                ));
            }
        }
        
        // コマンド実行
        let mut child = cmd.spawn()
            .map_err(|e| ExternalToolError::ExecutionFailed(format!("yt-dlpの実行に失敗しました: {}", e)))?;
        
        // 標準出力からの進捗情報の読み取り
        let stdout = child.stdout.take()
            .ok_or_else(|| ExternalToolError::Other("標準出力の取得に失敗しました".to_string()))?;
        
        let mut reader = BufReader::new(stdout).lines();
        
        // 進捗情報を読み取るタスクを開始
        let progress_task = tokio::spawn(async move {
            let mut downloaded_file = None;
            
            while let Ok(Some(line)) = reader.next_line().await {
                // 進捗情報の解析
                if line.contains("[download]") {
                    if let Some(percent_str) = line.split_whitespace().nth(1) {
                        if let Ok(percent) = percent_str.trim_end_matches('%').parse::<f64>() {
                            if let Some(ref callback) = progress_callback {
                                callback(percent / 100.0);
                            }
                        }
                    }
                } else if line.contains("Destination:") {
                    if let Some(file_path) = line.split("Destination:").nth(1) {
                        downloaded_file = Some(file_path.trim().to_string());
                    }
                }
                
                debug!("yt-dlp: {}", line);
            }
            
            downloaded_file
        });
        
        // 子プロセスが終了するのを待つ
        let status = child.wait().await
            .map_err(|e| ExternalToolError::ExecutionFailed(format!("yt-dlpの実行に失敗しました: {}", e)))?;
        
        // プロセスの終了コードを確認
        if !status.success() {
            return Err(ExternalToolError::ExecutionFailed(
                format!("yt-dlpがエラーコード{}で終了しました", status.code().unwrap_or(-1))
            ));
        }
        
        // 進捗情報タスクからファイルパスを取得
        let downloaded_file = timeout(Duration::from_secs(5), progress_task).await
            .map_err(|_| ExternalToolError::Timeout("進捗情報の取得がタイムアウトしました".to_string()))?
            .map_err(|e| ExternalToolError::Other(format!("進捗情報の取得に失敗しました: {}", e)))?;
        
        match downloaded_file {
            Some(file_path) => Ok(PathBuf::from(file_path)),
            None => {
                // ファイルパスが取得できなかった場合、ディレクトリ内の最新ファイルを探す
                let mut entries = std::fs::read_dir(output_dir)
                    .map_err(|e| ExternalToolError::Other(format!("出力ディレクトリの読み取りに失敗しました: {}", e)))?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                    .collect::<Vec<_>>();
                
                // 最新のファイルを取得
                entries.sort_by(|a, b| {
                    b.metadata().and_then(|m| m.modified()).unwrap_or_default()
                        .cmp(&a.metadata().and_then(|m| m.modified()).unwrap_or_default())
                });
                
                if let Some(latest) = entries.first() {
                    Ok(latest.path())
                } else {
                    Err(ExternalToolError::Other("ダウンロードされたファイルが見つかりません".to_string()))
                }
            }
        }
    }
    
    /// aria2cを使用して高速ダウンロード
    pub async fn download_with_aria2(
        &mut self,
        url: &str,
        output_path: &Path,
        progress_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
    ) -> Result<PathBuf, ExternalToolError> {
        // aria2cが利用可能か確認
        if !self.check_tool_available(ExternalTool::Aria2c).await {
            return Err(ExternalToolError::ToolNotFound(format!(
                "aria2cが見つかりません。インストールするには: {}", 
                ExternalTool::Aria2c.install_command()
            )));
        }
        
        let aria2c_path = self.tool_paths.get(&ExternalTool::Aria2c)
            .and_then(|p| p.as_ref())
            .ok_or_else(|| ExternalToolError::ToolNotFound("aria2cが見つかりません".to_string()))?;
        
        // 出力ディレクトリが存在することを確認
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ExternalToolError::Other(format!("出力ディレクトリの作成に失敗しました: {}", e)))?;
            }
        }
        
        info!("aria2cで高速ダウンロード中: {}", url);
        
        // aria2cコマンドの構築
        let mut cmd = Command::new(aria2c_path);
        cmd.arg("-x").arg("16")          // 最大接続数
           .arg("-s").arg("16")          // 分割数
           .arg("-k").arg("1M")          // 分割サイズ
           .arg("--retry-wait=1")        // リトライ間隔
           .arg("--max-tries=10")        // リトライ回数
           .arg("--file-allocation=none") // ファイル割り当てなし
           .arg("-o").arg(output_path)   // 出力パス
           .arg(url)                     // URL
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // コマンド実行
        let mut child = cmd.spawn()
            .map_err(|e| ExternalToolError::ExecutionFailed(format!("aria2cの実行に失敗しました: {}", e)))?;
        
        // 標準出力からの進捗情報の読み取り
        let stdout = child.stdout.take()
            .ok_or_else(|| ExternalToolError::Other("標準出力の取得に失敗しました".to_string()))?;
        
        let mut reader = BufReader::new(stdout).lines();
        
        // 進捗情報を読み取るタスクを開始
        let progress_task = tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                // 進捗情報の解析
                if line.contains("%") {
                    if let Some(percent_str) = line.split_whitespace()
                        .find(|s| s.contains("%"))
                        .and_then(|s| s.split('%').next()) {
                        if let Ok(percent) = percent_str.parse::<f64>() {
                            if let Some(ref callback) = progress_callback {
                                callback(percent / 100.0);
                            }
                        }
                    }
                }
                
                debug!("aria2c: {}", line);
            }
        });
        
        // 子プロセスが終了するのを待つ
        let status = child.wait().await
            .map_err(|e| ExternalToolError::ExecutionFailed(format!("aria2cの実行に失敗しました: {}", e)))?;
        
        // 進捗情報タスクをキャンセル
        progress_task.abort();
        
        // プロセスの終了コードを確認
        if !status.success() {
            return Err(ExternalToolError::ExecutionFailed(
                format!("aria2cがエラーコード{}で終了しました", status.code().unwrap_or(-1))
            ));
        }
        
        Ok(output_path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use tempfile::tempdir;
    
    #[test]
    #[ignore] // 実際のネットワークアクセスを行うためCI環境では無視
    fn test_check_tools_available() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut manager = ExternalToolManager::new();
            
            // ツールの存在確認
            let ytdlp_available = manager.check_tool_available(ExternalTool::YtDlp).await;
            let aria2c_available = manager.check_tool_available(ExternalTool::Aria2c).await;
            let ffmpeg_available = manager.check_tool_available(ExternalTool::FFmpeg).await;
            
            println!("yt-dlp available: {}", ytdlp_available);
            println!("aria2c available: {}", aria2c_available);
            println!("ffmpeg available: {}", ffmpeg_available);
        });
    }
} 