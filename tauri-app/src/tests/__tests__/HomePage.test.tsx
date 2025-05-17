import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import HomePage from '../../pages/HomePage';
import * as downloaderApi from '../__mocks__/downloaderApi';

// モックのインポート
jest.mock('../../api/downloader', () => require('../__mocks__/downloaderApi'));

// モックのナビゲーション
const mockNavigate = jest.fn();
jest.mock('react-router-dom', () => ({
  ...jest.requireActual('react-router-dom'),
  useNavigate: () => mockNavigate,
}));

// テスト用のラッパーコンポーネント
const HomePageWithRouter = () => (
  <BrowserRouter>
    <HomePage />
  </BrowserRouter>
);

describe('HomePage コンポーネント', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  test('正しくレンダリングされること', async () => {
    render(<HomePageWithRouter />);
    
    // タイトルが表示されていることを確認
    expect(screen.getByText('NextDownloader')).toBeInTheDocument();
    
    // フォーム要素が存在することを確認
    expect(screen.getByLabelText(/ダウンロードURL/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/保存先/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/希望するフォーマット/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /ダウンロード開始/i })).toBeInTheDocument();
    
    // 依存関係チェックが呼ばれることを確認
    await waitFor(() => {
      expect(downloaderApi.checkDependencies).toHaveBeenCalled();
    });
  });
  
  test('URLを入力するとコンテンツタイプが検出されること', async () => {
    render(<HomePageWithRouter />);
    
    const urlInput = screen.getByLabelText(/ダウンロードURL/i);
    fireEvent.change(urlInput, { target: { value: 'https://example.com/video.mp4' } });
    
    // コンテンツタイプ検出APIが呼ばれることを確認
    await waitFor(() => {
      expect(downloaderApi.detectContentType).toHaveBeenCalledWith('https://example.com/video.mp4');
    });
    
    // 検出されたコンテンツタイプが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText(/検出されたコンテンツタイプ/i)).toBeInTheDocument();
      expect(screen.getByText(/MP4/i)).toBeInTheDocument();
    });
  });
  
  test('必須フィールドが入力されていない場合、ダウンロードボタンが無効になること', () => {
    render(<HomePageWithRouter />);
    
    const downloadButton = screen.getByRole('button', { name: /ダウンロード開始/i });
    
    // 初期状態ではボタンが無効
    expect(downloadButton).toBeDisabled();
    
    // URLのみ入力した場合もボタンは無効
    const urlInput = screen.getByLabelText(/ダウンロードURL/i);
    fireEvent.change(urlInput, { target: { value: 'https://example.com/video.mp4' } });
    expect(downloadButton).toBeDisabled();
    
    // 保存先も入力するとボタンが有効になる
    const destinationInput = screen.getByLabelText(/保存先/i);
    fireEvent.change(destinationInput, { target: { value: '/tmp/video.mp4' } });
    expect(downloadButton).not.toBeDisabled();
  });
  
  test('ダウンロードを開始すると、ダウンロード管理ページに遷移すること', async () => {
    render(<HomePageWithRouter />);
    
    // 必須フィールドを入力
    const urlInput = screen.getByLabelText(/ダウンロードURL/i);
    const destinationInput = screen.getByLabelText(/保存先/i);
    
    fireEvent.change(urlInput, { target: { value: 'https://example.com/video.mp4' } });
    fireEvent.change(destinationInput, { target: { value: '/tmp/video.mp4' } });
    
    // ダウンロードボタンをクリック
    const downloadButton = screen.getByRole('button', { name: /ダウンロード開始/i });
    fireEvent.click(downloadButton);
    
    // ダウンロード開始APIが呼ばれることを確認
    await waitFor(() => {
      expect(downloaderApi.startDownload).toHaveBeenCalledWith(
        'https://example.com/video.mp4',
        '/tmp/video.mp4',
        undefined
      );
    });
    
    // ダウンロード管理ページに遷移することを確認
    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/downloads');
    });
  });
  
  test('フォーマットを指定してダウンロードできること', async () => {
    render(<HomePageWithRouter />);
    
    // 必須フィールドとフォーマットを入力
    const urlInput = screen.getByLabelText(/ダウンロードURL/i);
    const destinationInput = screen.getByLabelText(/保存先/i);
    const formatInput = screen.getByLabelText(/希望するフォーマット/i);
    
    fireEvent.change(urlInput, { target: { value: 'https://example.com/video.mp4' } });
    fireEvent.change(destinationInput, { target: { value: '/tmp/video.mp4' } });
    fireEvent.change(formatInput, { target: { value: 'mp4' } });
    
    // ダウンロードボタンをクリック
    const downloadButton = screen.getByRole('button', { name: /ダウンロード開始/i });
    fireEvent.click(downloadButton);
    
    // フォーマットを指定してダウンロード開始APIが呼ばれることを確認
    await waitFor(() => {
      expect(downloaderApi.startDownload).toHaveBeenCalledWith(
        'https://example.com/video.mp4',
        '/tmp/video.mp4',
        'mp4'
      );
    });
  });
  
  test('依存関係の問題がある場合、警告が表示されること', async () => {
    // 依存関係の一部が不足している状態をモック
    downloaderApi.checkDependencies.mockResolvedValueOnce({
      ytdlp: false,
      aria2c: true,
      ffmpeg: true,
    });
    
    render(<HomePageWithRouter />);
    
    // 依存関係の警告が表示されることを確認
    await waitFor(() => {
      expect(screen.getByText(/依存関係の問題/i)).toBeInTheDocument();
    });
  });
});