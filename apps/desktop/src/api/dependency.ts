import { invoke } from '@tauri-apps/api/tauri';
import { DependencyStatus } from '../types/dependency';

/**
 * 依存関係をチェックします
 * @returns 依存関係のステータス
 */
export async function checkDependencies(): Promise<DependencyStatus> {
  try {
    return await invoke<DependencyStatus>('check_dependencies');
  } catch (error) {
    console.error('依存関係のチェックに失敗しました:', error);
    throw error;
  }
}