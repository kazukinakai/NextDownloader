import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import SettingsPage from '../../pages/SettingsPage';
import * as downloaderApi from '../__mocks__/downloaderApi';

// モックのインポート
jest.mock('../../api/downloader', () => require('../__mocks__/downloaderApi'));

// テスト用のラッパーコンポーネント
const SettingsPageWithRouter = () => (
  <BrowserRouter>
    <SettingsPage />
  </BrowserRouter>
);

describe('SettingsPage コンポーネント', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  test('正しくレンダリングされること', async () => {
    render(<SettingsPageWithRouter />);
    
    // タイトルが表示されていることを確認
    expect(screen.getByText('設定')).toBeInTheDocument();
    
    // 一般設定セクションが表示されていることを確認
    expect(screen.getByText('一般設定')).toBeInTheDocument();
    
    // 依存関係の管理セクションが表示されていることを確認
    expect(screen.getByText('依存関係の管理')).toBeInTheDocument();
    
    // 依存関係チェックが呼ばれることを確認
    await waitFor(() => {
      expect(downloaderApi.checkDependencies).toHaveBeenCalled();
    });
  });
  
  test('設定を変更して保存できること', async () => {
    render(<SettingsPageWithRouter />);
    
    // デフォルトのダウンロードパスを変更
    const downloadPathInput = screen.getByLabelText(/デフォルトのダウンロードパス/i);
    fireEvent.change(downloadPathInput, { target: { value: '/new/download/path' } });
    
    // 最大同時ダウンロード数を変更
    const maxDownloadsSelect = screen.getByLabelText(/最大同時ダウンロード数/i);
    fireEvent.change(maxDownloadsSelect, { target: { value: '5' } });
    
    // テーマを変更
    const themeSelect = screen.getByLabelText(/テーマ/i);
    fireEvent.change(themeSelect, { target: { value: 'dark' } });
    
    // 通知設定を変更
    const notifySwitch = screen.getByLabelText(/ダウンロード完了時に通知する/i);
    fireEvent.click(notifySwitch);
    
    // 保存ボタンをクリック
    const saveButton = screen.getByRole('button', { name: /設定を保存/i });
    fireEvent.click(saveButton);
    
    // 成功メッセージが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText(/設定を保存しました/i)).toBeInTheDocument();
    });
  });
  
  test('依存関係の状態が正しく表示されること', async () => {
    // すべての依存関係が利用可能な状態をモック
    downloaderApi.checkDependencies.mockResolvedValueOnce({
      ytdlp: true,
      aria2c: true,
      ffmpeg: true,
    });
    
    render(<SettingsPageWithRouter />);
    
    // 依存関係のステータスが表示されることを確認
    await waitFor(() => {
      expect(screen.getAllByText(/インストール済み/i).length).toBe(3);
    });
    
    // 依存関係のチェックボタンが機能することを確認
    const recheckButton = screen.getByRole('button', { name: /再チェック/i });
    fireEvent.click(recheckButton);
    
    await waitFor(() => {
      expect(downloaderApi.checkDependencies).toHaveBeenCalledTimes(2);
    });
  });
  
  test('依存関係が不足している場合、適切な情報が表示されること', async () => {
    // 一部の依存関係が不足している状態をモック
    downloaderApi.checkDependencies.mockResolvedValueOnce({
      ytdlp: false,
      aria2c: true,
      ffmpeg: false,
    });
    
    render(<SettingsPageWithRouter />);
    
    // 不足している依存関係の警告が表示されることを確認
    await waitFor(() => {
      expect(screen.getAllByText(/未インストール/i).length).toBe(2);
      expect(screen.getByText(/インストール方法: pip install yt-dlp/i)).toBeInTheDocument();
      expect(screen.getByText(/インストール方法: brew install ffmpeg/i)).toBeInTheDocument();
    });
  });
  
  test('言語設定を変更できること', async () => {
    render(<SettingsPageWithRouter />);
    
    // 言語設定を変更
    const languageSelect = screen.getByLabelText(/言語/i);
    fireEvent.change(languageSelect, { target: { value: 'en' } });
    
    // 保存ボタンをクリック
    const saveButton = screen.getByRole('button', { name: /設定を保存/i });
    fireEvent.click(saveButton);
    
    // 成功メッセージが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText(/設定を保存しました/i)).toBeInTheDocument();
    });
  });
});