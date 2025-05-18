/**
 * NextDownloader APIラッパー
 * Tauriバックエンドとの通信を担当します
 */

// Tauri APIのインポート
import { invoke } from '@tauri-apps/api';

// コンテンツタイプの定義
export enum ContentType {
  MP4 = 'MP4',
  HLS = 'HLS',
  DASH = 'DASH',
  YouTube = 'YouTube',
  Unknown = 'Unknown',
}

// 依存関係のステータス
export interface DependencyStatus {
  ytdlp: boolean;
  aria2c: boolean;
  ffmpeg: boolean;
}

/**
 * 依存関係をチェックする
 * @returns 依存関係のステータス
 */
export async function checkDependencies(): Promise<DependencyStatus> {
  try {
    return await invoke<DependencyStatus>('plugin:shell|check_dependencies');
  } catch (error) {
    console.error('依存関係のチェックに失敗しました:', error);
    // デフォルトのステータスを返す
    return {
      ytdlp: false,
      aria2c: false,
      ffmpeg: false,
    };
  }
}

/**
 * URLからコンテンツタイプを検出する
 * @param url 検出するURL
 * @returns 検出されたコンテンツタイプ
 */
export async function detectContentType(url: string): Promise<ContentType> {
  try {
    return await invoke<ContentType>('plugin:http|detect_content_type', { url });
  } catch (error) {
    console.error('コンテンツタイプの検出に失敗しました:', error);
    return ContentType.Unknown;
  }
}

/**
 * ダウンロードを開始する
 * @param url ダウンロードするURL
 * @param destination 保存先のパス
 * @param format 希望するフォーマット（オプション）
 * @returns ダウンロードID
 */
export async function startDownload(
  url: string,
  destination: string,
  format?: string
): Promise<string> {
  try {
    return await invoke<string>('plugin:fs|start_download', {
      url,
      destination,
      format: format || '',
    });
  } catch (error) {
    console.error('ダウンロードの開始に失敗しました:', error);
    throw error;
  }
}

/**
 * ダウンロードの進捗を取得する
 * @param downloadId ダウンロードID
 * @returns 進捗率（0-100）
 */
export async function getDownloadProgress(downloadId: string): Promise<number> {
  try {
    return await invoke<number>('plugin:fs|get_download_progress', { downloadId });
  } catch (error) {
    console.error('進捗の取得に失敗しました:', error);
    return 0;
  }
}

/**
 * ダウンロードをキャンセルする
 * @param downloadId ダウンロードID
 */
export async function cancelDownload(downloadId: string): Promise<void> {
  try {
    await invoke('plugin:fs|cancel_download', { downloadId });
  } catch (error) {
    console.error('ダウンロードのキャンセルに失敗しました:', error);
    throw error;
  }
}