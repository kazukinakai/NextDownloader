import { invoke } from '@tauri-apps/api/tauri';

/**
 * ディレクトリを選択します
 * @param title ダイアログのタイトル
 * @param defaultPath 初期ディレクトリ
 * @returns 選択されたディレクトリのパス（キャンセルされた場合はnull）
 */
export async function selectDirectory(
  title?: string,
  defaultPath?: string
): Promise<string | null> {
  try {
    const response = await invoke<{ path: string | null }>('select_directory', {
      request: {
        title,
        default_path: defaultPath,
      },
    });
    return response.path;
  } catch (error) {
    console.error('ディレクトリ選択に失敗しました:', error);
    throw error;
  }
}

/**
 * アプリケーションのバージョンを取得します
 * @returns バージョン文字列
 */
export async function getVersion(): Promise<string> {
  try {
    return await invoke<string>('get_version');
  } catch (error) {
    console.error('バージョン情報の取得に失敗しました:', error);
    throw error;
  }
}

/**
 * ファイルサイズを人間が読みやすい形式にフォーマットします
 * @param bytes バイト数
 * @returns フォーマットされたサイズ文字列
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * 秒数を人間が読みやすい形式にフォーマットします
 * @param seconds 秒数
 * @returns フォーマットされた時間文字列
 */
export function formatTime(seconds: number): string {
  if (seconds < 60) {
    return `${Math.floor(seconds)}秒`;
  } else if (seconds < 3600) {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes}分${remainingSeconds > 0 ? ` ${remainingSeconds}秒` : ''}`;
  } else {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}時間${minutes > 0 ? ` ${minutes}分` : ''}`;
  }
}