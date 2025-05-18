/**
 * アプリケーション設定
 */
export interface AppConfig {
  /** ダウンロードマネージャーの設定 */
  download_manager: DownloadManagerConfig;
  /** デフォルトのダウンロードディレクトリ */
  default_download_dir: string;
  /** テーマ（"light", "dark", "system"） */
  theme: string;
  /** 言語（"ja", "en"） */
  language: string;
  /** ダウンロード完了時に通知する */
  notify_on_completion: boolean;
  /** アーカイブを自動的に解凍する */
  auto_extract_archives: boolean;
  /** 依存ツールのパス */
  tool_paths: ToolPaths;
}

/**
 * ダウンロードマネージャーの設定
 */
export interface DownloadManagerConfig {
  /** 同時ダウンロード数 */
  concurrent_downloads: number;
  /** ダウンロード再試行回数 */
  retry_count: number;
  /** ダウンロードのタイムアウト（秒） */
  timeout_seconds: number;
  /** ダウンロード速度制限（バイト/秒、0は無制限） */
  speed_limit: number;
}

/**
 * 依存ツールのパス
 */
export interface ToolPaths {
  /** yt-dlpのパス */
  ytdlp?: string;
  /** aria2cのパス */
  aria2c?: string;
  /** ffmpegのパス */
  ffmpeg?: string;
}

/**
 * 設定取得レスポンス
 */
export interface GetConfigResponse {
  /** 設定内容 */
  config: AppConfig;
  /** 設定ファイルのパス */
  config_path: string;
}