/**
 * ダウンロードAPIのモック
 * テスト用にダウンロードAPIの動作をシミュレートします
 */
import { ContentType, DependencyStatus } from '../../api/downloader';

// モックデータ
export const mockDependencies: DependencyStatus = {
  ytdlp: true,
  aria2c: true,
  ffmpeg: true,
};

export const mockContentTypes: Record<string, ContentType> = {
  'https://example.com/video.mp4': ContentType.MP4,
  'https://example.com/stream.m3u8': ContentType.HLS,
  'https://example.com/manifest.mpd': ContentType.DASH,
  'https://www.youtube.com/watch?v=dQw4w9WgXcQ': ContentType.YouTube,
  'https://example.com/unknown': ContentType.Unknown,
};

export const mockDownloads: Record<string, { progress: number; status: string }> = {
  'download-1': { progress: 0.75, status: 'downloading' },
  'download-2': { progress: 1.0, status: 'completed' },
};

// モック関数
export const checkDependencies = jest.fn().mockImplementation(
  () => Promise.resolve(mockDependencies)
);

export const detectContentType = jest.fn().mockImplementation(
  (url: string) => {
    if (!url) {
      return Promise.reject(new Error('Invalid URL'));
    }
    
    return Promise.resolve(
      mockContentTypes[url] || ContentType.Unknown
    );
  }
);

export const startDownload = jest.fn().mockImplementation(
  (url: string, destination: string) => {
    if (!url || !destination) {
      return Promise.reject(new Error('Invalid parameters'));
    }
    
    const downloadId = `download-${Date.now()}`;
    mockDownloads[downloadId] = { progress: 0, status: 'downloading' };
    
    // 進捗をシミュレート
    setTimeout(() => {
      mockDownloads[downloadId].progress = 0.5;
    }, 1000);
    
    setTimeout(() => {
      mockDownloads[downloadId].progress = 1.0;
      mockDownloads[downloadId].status = 'completed';
    }, 2000);
    
    return Promise.resolve(downloadId);
  }
);

export const getDownloadProgress = jest.fn().mockImplementation(
  (downloadId: string) => {
    if (!downloadId || !mockDownloads[downloadId]) {
      return Promise.reject(new Error('Invalid download ID'));
    }
    
    return Promise.resolve(mockDownloads[downloadId].progress);
  }
);

export const cancelDownload = jest.fn().mockImplementation(
  (downloadId: string) => {
    if (!downloadId || !mockDownloads[downloadId]) {
      return Promise.reject(new Error('Invalid download ID'));
    }
    
    delete mockDownloads[downloadId];
    return Promise.resolve();
  }
);

// イベントリスナー
export const setupDownloadListeners = jest.fn().mockImplementation(
  () => Promise.resolve(() => {})
);