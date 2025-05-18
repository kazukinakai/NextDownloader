import { invoke } from '@tauri-apps/api/tauri';
import { 
  DownloadInfo, 
  StartDownloadRequest, 
  StartDownloadResponse 
} from '../types/download';

/**
 * ダウンロードを開始します
 * @param request ダウンロード開始リクエスト
 * @returns ダウンロード開始レスポンス
 */
export async function startDownload(request: StartDownloadRequest): Promise<StartDownloadResponse> {
  try {
    return await invoke<StartDownloadResponse>('start_download', { request });
  } catch (error) {
    console.error('ダウンロードの開始に失敗しました:', error);
    throw error;
  }
}

/**
 * ダウンロード進捗を取得します
 * @param id ダウンロードID
 * @returns ダウンロード情報
 */
export async function getDownloadProgress(id: string): Promise<DownloadInfo> {
  try {
    return await invoke<DownloadInfo>('get_download_progress', { 
      request: { id } 
    });
  } catch (error) {
    console.error('ダウンロード進捗の取得に失敗しました:', error);
    throw error;
  }
}

/**
 * ダウンロードをキャンセルします
 * @param id ダウンロードID
 */
export async function cancelDownload(id: string): Promise<void> {
  try {
    await invoke<void>('cancel_download', { 
      request: { id } 
    });
  } catch (error) {
    console.error('ダウンロードのキャンセルに失敗しました:', error);
    throw error;
  }
}

/**
 * ダウンロード一覧を取得します
 * @returns ダウンロード情報の配列
 */
export async function getDownloads(): Promise<DownloadInfo[]> {
  try {
    return await invoke<DownloadInfo[]>('get_downloads');
  } catch (error) {
    console.error('ダウンロード一覧の取得に失敗しました:', error);
    throw error;
  }
}