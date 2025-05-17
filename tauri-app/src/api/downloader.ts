/**
 * NextDownloader ダウンロードAPI
 * Tauriプラグインを使用してRustのダウンロード機能を呼び出すためのラッパー
 */
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

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

// ダウンロードの進捗リスナーの登録
export async function setupDownloadListeners(
  onProgress: (id: string, progress: number) => void,
  onComplete: (id: string) => void,
): Promise<() => void> {
  // 進捗イベントのリスナー
  const progressUnlisten = await listen<[string, number]>('download-progress', (event) => {
    const [id, progress] = event.payload;
    onProgress(id, progress);
  });

  // 完了イベントのリスナー
  const completeUnlisten = await listen<string>('download-complete', (event) => {
    const id = event.payload;
    onComplete(id);
  });

  // クリーンアップ関数を返す
  return () => {
    progressUnlisten();
    completeUnlisten();
  };
}

/**
 * システムの依存関係をチェックする
 * @returns 依存関係のステータス
 */
export async function checkDependencies(): Promise<DependencyStatus> {
  return await invoke<DependencyStatus>('check_dependencies');
}

/**
 * URLからコンテンツタイプを検出する
 * @param url 検出するURL
 * @returns コンテンツタイプ
 */
export async function detectContentType(url: string): Promise<ContentType> {
  return await invoke<ContentType>('detect_content_type', { url });
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
  format?: string,
): Promise<string> {
  return await invoke<string>('start_download', { url, destination, format });
}

/**
 * ダウンロードの進捗を取得する
 * @param downloadId ダウンロードID
 * @returns 進捗（0.0〜1.0）
 */
export async function getDownloadProgress(downloadId: string): Promise<number> {
  return await invoke<number>('get_download_progress', { downloadId });
}

/**
 * ダウンロードをキャンセルする
 * @param downloadId ダウンロードID
 */
export async function cancelDownload(downloadId: string): Promise<void> {
  await invoke<void>('cancel_download', { downloadId });
}