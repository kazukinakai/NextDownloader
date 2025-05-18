/**
 * ダウンロード情報
 */
export interface DownloadInfo {
  /** ダウンロードID */
  id: string;
  /** ダウンロードURL */
  url: string;
  /** 保存先パス */
  destination: string;
  /** ファイル名 */
  filename: string;
  /** コンテンツタイプ */
  content_type: string;
  /** ダウンロードの進捗状況 (0.0 - 1.0) */
  progress: number;
  /** ダウンロード速度（バイト/秒） */
  speed?: number;
  /** 推定残り時間（秒） */
  eta?: number;
  /** ダウンロード済みサイズ（バイト） */
  downloaded_size: number;
  /** 合計サイズ（バイト） */
  total_size?: number;
  /** ステータスメッセージ */
  status_message?: string;
  /** ダウンロードステータス */
  status: string;
  /** 作成日時 */
  created_at: string;
  /** 更新日時 */
  updated_at: string;
}

/**
 * ダウンロード開始リクエスト
 */
export interface StartDownloadRequest {
  /** ダウンロードURL */
  url: string;
  /** 保存先ディレクトリ */
  destination_dir: string;
  /** ファイル名（省略可） */
  filename?: string;
  /** フォーマット（省略可） */
  format?: string;
}

/**
 * ダウンロード開始レスポンス
 */
export interface StartDownloadResponse {
  /** ダウンロードID */
  id: string;
}