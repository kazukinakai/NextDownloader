import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import DownloadsPage from '../../pages/DownloadsPage';
import * as downloaderApi from '../__mocks__/downloaderApi';

// モックのインポート
jest.mock('../../api/downloader', () => require('../__mocks__/downloaderApi'));

// テスト用のラッパーコンポーネント
const DownloadsPageWithRouter = () => (
  <BrowserRouter>
    <DownloadsPage />
  </BrowserRouter>
);

describe('DownloadsPage コンポーネント', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  test('正しくレンダリングされること', async () => {
    render(<DownloadsPageWithRouter />);
    
    // タイトルが表示されていることを確認
    expect(screen.getByText('ダウンロード管理')).toBeInTheDocument();
    
    // 進行中のダウンロードセクションが表示されていることを確認
    expect(screen.getByText(/進行中のダウンロード/i)).toBeInTheDocument();
    
    // 完了したダウンロードセクションが表示されていることを確認
    expect(screen.getByText(/完了したダウンロード/i)).toBeInTheDocument();
    
    // ダウンロードリスナーが設定されることを確認
    await waitFor(() => {
      expect(downloaderApi.setupDownloadListeners).toHaveBeenCalled();
    });
  });
  
  test('進行中のダウンロードが表示されること', async () => {
    render(<DownloadsPageWithRouter />);
    
    // 進行中のダウンロードが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText('https://example.com/video1.mp4')).toBeInTheDocument();
      expect(screen.getByText('75%')).toBeInTheDocument();
      expect(screen.getByText('ダウンロード中')).toBeInTheDocument();
    });
  });
  
  test('完了したダウンロードが表示されること', async () => {
    render(<DownloadsPageWithRouter />);
    
    // 完了したダウンロードが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText('https://example.com/video2.mp4')).toBeInTheDocument();
      expect(screen.getByText('/Users/username/Downloads/video2.mp4')).toBeInTheDocument();
      expect(screen.getAllByRole('button', { name: /削除/i })).toHaveLength(1);
    });
  });
  
  test('ダウンロードをキャンセルできること', async () => {
    render(<DownloadsPageWithRouter />);
    
    // キャンセルボタンが表示されることを確認
    const cancelButtons = await screen.findAllByRole('button', { name: /キャンセル/i });
    expect(cancelButtons.length).toBeGreaterThan(0);
    
    // キャンセルボタンをクリック
    fireEvent.click(cancelButtons[0]);
    
    // キャンセルAPIが呼ばれることを確認
    await waitFor(() => {
      expect(downloaderApi.cancelDownload).toHaveBeenCalled();
    });
  });
  
  test('一時停止/再開機能が準備中であることを通知すること', async () => {
    render(<DownloadsPageWithRouter />);
    
    // 一時停止ボタンが表示されることを確認
    const pauseButtons = await screen.findAllByRole('button', { name: /一時停止\/再開/i });
    expect(pauseButtons.length).toBeGreaterThan(0);
    
    // 一時停止ボタンをクリック
    fireEvent.click(pauseButtons[0]);
    
    // 機能準備中の通知が表示されることを確認
    await waitFor(() => {
      expect(screen.getByText(/機能準備中/i)).toBeInTheDocument();
    });
  });
  
  test('進行中のダウンロードがない場合、適切なメッセージが表示されること', async () => {
    // 進行中のダウンロードがない状態をモック
    jest.spyOn(React, 'useState').mockImplementationOnce(() => [[], jest.fn()]);
    
    render(<DownloadsPageWithRouter />);
    
    // メッセージが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText(/現在進行中のダウンロードはありません/i)).toBeInTheDocument();
    });
  });
  
  test('完了したダウンロードがない場合、適切なメッセージが表示されること', async () => {
    // 完了したダウンロードがない状態をモック
    jest.spyOn(React, 'useState').mockImplementationOnce(() => [
      [
        {
          id: 'download-1',
          url: 'https://example.com/video1.mp4',
          destination: '/Users/username/Downloads/video1.mp4',
          progress: 0.75,
          status: 'downloading',
          createdAt: new Date(),
        }
      ],
      jest.fn()
    ]);
    
    render(<DownloadsPageWithRouter />);
    
    // メッセージが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText(/完了したダウンロードはありません/i)).toBeInTheDocument();
    });
  });
});