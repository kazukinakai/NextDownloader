import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import {
  checkDependencies,
  detectContentType,
  startDownload,
  getDownloadProgress,
  cancelDownload,
  setupDownloadListeners,
  ContentType,
} from '../../api/downloader';

// Tauriのモック
jest.mock('@tauri-apps/api/tauri', () => ({
  invoke: jest.fn(),
}));

jest.mock('@tauri-apps/api/event', () => ({
  listen: jest.fn(() => Promise.resolve(() => {})),
}));

describe('ダウンローダーAPI', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('checkDependencies', () => {
    test('依存関係のチェックが正常に行われること', async () => {
      const mockResponse = {
        ytdlp: true,
        aria2c: true,
        ffmpeg: true,
      };
      
      (invoke as jest.Mock).mockResolvedValueOnce(mockResponse);
      
      const result = await checkDependencies();
      
      expect(invoke).toHaveBeenCalledWith('plugin:download|check_dependencies');
      expect(result).toEqual(mockResponse);
    });
    
    test('エラーが発生した場合、適切に処理されること', async () => {
      (invoke as jest.Mock).mockRejectedValueOnce(new Error('Failed to check dependencies'));
      
      await expect(checkDependencies()).rejects.toThrow('Failed to check dependencies');
    });
  });
  
  describe('detectContentType', () => {
    test('コンテンツタイプの検出が正常に行われること', async () => {
      const mockResponse = {
        contentType: ContentType.MP4,
      };
      
      (invoke as jest.Mock).mockResolvedValueOnce(mockResponse);
      
      const result = await detectContentType('https://example.com/video.mp4');
      
      expect(invoke).toHaveBeenCalledWith('plugin:download|detect_content_type', {
        url: 'https://example.com/video.mp4',
      });
      expect(result).toEqual(ContentType.MP4);
    });
    
    test('無効なURLの場合、エラーが発生すること', async () => {
      (invoke as jest.Mock).mockRejectedValueOnce(new Error('Invalid URL'));
      
      await expect(detectContentType('')).rejects.toThrow('Invalid URL');
    });
  });
  
  describe('startDownload', () => {
    test('ダウンロードが正常に開始されること', async () => {
      const mockDownloadId = 'download-123';
      (invoke as jest.Mock).mockResolvedValueOnce(mockDownloadId);
      
      const result = await startDownload(
        'https://example.com/video.mp4',
        '/tmp/video.mp4'
      );
      
      expect(invoke).toHaveBeenCalledWith('plugin:download|start_download', {
        url: 'https://example.com/video.mp4',
        destination: '/tmp/video.mp4',
        options: undefined,
      });
      expect(result).toEqual(mockDownloadId);
    });
    
    test('オプションを指定してダウンロードを開始できること', async () => {
      const mockDownloadId = 'download-123';
      (invoke as jest.Mock).mockResolvedValueOnce(mockDownloadId);
      
      const result = await startDownload(
        'https://example.com/video.mp4',
        '/tmp/video.mp4',
        'mp4'
      );
      
      expect(invoke).toHaveBeenCalledWith('plugin:download|start_download', {
        url: 'https://example.com/video.mp4',
        destination: '/tmp/video.mp4',
        options: 'mp4',
      });
      expect(result).toEqual(mockDownloadId);
    });
    
    test('ダウンロード開始に失敗した場合、エラーが発生すること', async () => {
      (invoke as jest.Mock).mockRejectedValueOnce(new Error('Failed to start download'));
      
      await expect(startDownload('https://example.com/video.mp4', '/tmp/video.mp4')).rejects.toThrow(
        'Failed to start download'
      );
    });
  });
  
  describe('getDownloadProgress', () => {
    test('ダウンロード進捗が正常に取得できること', async () => {
      const mockProgress = 0.75;
      (invoke as jest.Mock).mockResolvedValueOnce(mockProgress);
      
      const result = await getDownloadProgress('download-123');
      
      expect(invoke).toHaveBeenCalledWith('plugin:download|get_download_progress', {
        downloadId: 'download-123',
      });
      expect(result).toEqual(mockProgress);
    });
    
    test('無効なダウンロードIDの場合、エラーが発生すること', async () => {
      (invoke as jest.Mock).mockRejectedValueOnce(new Error('Invalid download ID'));
      
      await expect(getDownloadProgress('invalid-id')).rejects.toThrow('Invalid download ID');
    });
  });
  
  describe('cancelDownload', () => {
    test('ダウンロードが正常にキャンセルされること', async () => {
      (invoke as jest.Mock).mockResolvedValueOnce(undefined);
      
      await cancelDownload('download-123');
      
      expect(invoke).toHaveBeenCalledWith('plugin:download|cancel_download', {
        downloadId: 'download-123',
      });
    });
    
    test('無効なダウンロードIDの場合、エラーが発生すること', async () => {
      (invoke as jest.Mock).mockRejectedValueOnce(new Error('Invalid download ID'));
      
      await expect(cancelDownload('invalid-id')).rejects.toThrow('Invalid download ID');
    });
  });
  
  describe('setupDownloadListeners', () => {
    test('ダウンロードリスナーが正常に設定されること', async () => {
      const mockUnlisten = jest.fn();
      (listen as jest.Mock).mockResolvedValueOnce(mockUnlisten);
      
      const onProgress = jest.fn();
      const onComplete = jest.fn();
      const onError = jest.fn();
      
      const unlisten = await setupDownloadListeners({
        onProgress,
        onComplete,
        onError,
      });
      
      expect(listen).toHaveBeenCalledWith('download://progress', expect.any(Function));
      expect(listen).toHaveBeenCalledWith('download://complete', expect.any(Function));
      expect(listen).toHaveBeenCalledWith('download://error', expect.any(Function));
      expect(unlisten).toBe(mockUnlisten);
    });
  });
});