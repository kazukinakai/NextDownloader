use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use nextdownloader_core::{
    DownloadManager, 
    Downloader, 
    DownloadOptions, 
    ProgressInfo,
    VideoFormat
};
use anyhow::{Result, Context};

/// NextDownloader - マルチプラットフォーム動画ダウンロードツール
#[derive(Parser)]
#[clap(name = "nextdownloader")]
#[clap(about = "高速マルチプラットフォーム動画ダウンロードツール", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// URLから動画をダウンロード
    Download {
        /// ダウンロードするURL
        #[clap(short, long)]
        url: String,
        
        /// 出力ディレクトリ
        #[clap(short, long, default_value = ".")]
        output: PathBuf,
        
        /// 出力ファイル名（拡張子なし）
        #[clap(short, long)]
        filename: Option<String>,
        
        /// 出力フォーマット（mp4, mkv, mp3）
        #[clap(short, long, default_value = "mp4")]
        format: String,
        
        /// 並列コネクション数
        #[clap(short, long, default_value_t = 16)]
        connections: u32,
        
        /// ファイル分割数
        #[clap(short = 's', long, default_value_t = 16)]
        splits: u32,
        
        /// チャンクサイズ (MB)
        #[clap(short, long, default_value_t = 4)]
        chunk_size: u32,
    },
    
    /// システム状態を確認
    Check,
}

/// メイン関数
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Download { 
            url, 
            output, 
            filename, 
            format,
            connections,
            splits,
            chunk_size
        } => {
            download_command(
                &url, 
                &output, 
                filename.as_deref(), 
                &format,
                connections,
                splits,
                chunk_size
            ).await?;
        }
        Commands::Check => {
            check_command().await?;
        }
    }
    
    Ok(())
}

/// ダウンロードコマンドの実装
async fn download_command(
    url: &str, 
    output_path: &PathBuf, 
    filename_opt: Option<&str>,
    format_str: &str,
    connections: u32,
    splits: u32,
    chunk_size: u32
) -> Result<()> {
    // ダウンロードマネージャーの初期化
    let downloader = DownloadManager::new();
    
    // システム状態のチェック
    let status = downloader.system_status().await;
    if !status.is_ready() {
        println!("{}", status.description());
        return Ok(());
    }
    
    // フォーマット解析
    let format = match format_str.to_lowercase().as_str() {
        "mp4" => VideoFormat::Mp4,
        "mkv" => VideoFormat::Mkv,
        "mp3" => VideoFormat::Mp3,
        _ => {
            println!("サポートされていないフォーマット: {}。MP4を使用します。", format_str);
            VideoFormat::Mp4
        }
    };
    
    // ファイル名の生成
    let filename = if let Some(name) = filename_opt {
        name.to_string()
    } else {
        // URLからファイル名を抽出
        url.split('/')
            .last()
            .unwrap_or("download")
            .split('?')
            .next()
            .unwrap_or("download")
            .to_string()
    };
    
    // ダウンロードオプション
    let options = DownloadOptions {
        connections,
        splits,
        chunk_size,
        format,
        ..Default::default()
    };
    
    // プログレスバーの設定
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(format!("ダウンロード中: {}", url));
    
    // プログレスコールバック
    let progress_callback = Box::new(move |info: ProgressInfo| {
        pb.set_position((info.progress * 100.0) as u64);
        pb.set_message(format!(
            "ダウンロード中: {} (速度: {}, 残り時間: {})",
            url, info.speed, info.eta
        ));
    });
    
    // ダウンロード実行
    let result = downloader
        .download(url, output_path, &filename, Some(options), Some(progress_callback))
        .await
        .context("ダウンロード中にエラーが発生しました")?;
        
    pb.finish_with_message(format!("ダウンロード完了: {}", result.to_string_lossy()));
    
    println!("\nファイルを保存しました: {}", result.to_string_lossy());
    
    Ok(())
}

/// システム状態確認コマンドの実装
async fn check_command() -> Result<()> {
    let downloader = DownloadManager::new();
    let (ytdlp, aria2c, ffmpeg) = downloader.check_dependencies().await;
    
    println!("NextDownloader システム状態:");
    println!("============================");
    println!("yt-dlp: {}", if ytdlp { "✅ 利用可能" } else { "❌ 見つかりません" });
    println!("aria2c: {}", if aria2c { "✅ 利用可能" } else { "❌ 見つかりません" });
    println!("ffmpeg: {}", if ffmpeg { "✅ 利用可能" } else { "❌ 見つかりません" });
    
    if ytdlp && aria2c && ffmpeg {
        println!("\n✅ システムは正常に動作しています");
    } else {
        println!("\n❌ 一部の依存関係が不足しています");
        
        // 不足ツールのインストール方法
        if !ytdlp {
            println!("\nyt-dlpのインストール方法:");
            println!("  • macOS: brew install yt-dlp");
            println!("  • Linux: pip install yt-dlp");
            println!("  • Windows: winget install yt-dlp");
        }
        
        if !aria2c {
            println!("\naria2cのインストール方法:");
            println!("  • macOS: brew install aria2");
            println!("  • Linux: sudo apt install aria2 または sudo dnf install aria2");
            println!("  • Windows: winget install aria2");
        }
        
        if !ffmpeg {
            println!("\nffmpegのインストール方法:");
            println!("  • macOS: brew install ffmpeg");
            println!("  • Linux: sudo apt install ffmpeg または sudo dnf install ffmpeg");
            println!("  • Windows: winget install ffmpeg");
        }
    }
    
    Ok(())
}
