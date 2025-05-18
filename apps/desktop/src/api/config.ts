import { invoke } from '@tauri-apps/api/tauri';
import { AppConfig, GetConfigResponse } from '../types/config';

/**
 * アプリケーション設定を取得します
 * @returns 設定取得レスポンス
 */
export async function getConfig(): Promise<GetConfigResponse> {
  try {
    return await invoke<GetConfigResponse>('get_config');
  } catch (error) {
    console.error('設定の取得に失敗しました:', error);
    throw error;
  }
}

/**
 * アプリケーション設定を保存します
 * @param config アプリケーション設定
 */
export async function saveConfig(config: AppConfig): Promise<void> {
  try {
    await invoke<void>('save_config', { 
      request: { config } 
    });
  } catch (error) {
    console.error('設定の保存に失敗しました:', error);
    throw error;
  }
}