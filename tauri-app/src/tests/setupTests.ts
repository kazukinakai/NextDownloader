// テスト環境のセットアップ
import '@testing-library/jest-dom';

// Tauriのモック
jest.mock('@tauri-apps/api/tauri', () => ({
  invoke: jest.fn(),
  convertFileSrc: jest.fn((path) => path),
}));

jest.mock('@tauri-apps/api/event', () => ({
  listen: jest.fn(() => Promise.resolve(() => {})),
  emit: jest.fn(),
}));

jest.mock('@tauri-apps/api/dialog', () => ({
  open: jest.fn(() => Promise.resolve('/mock/path/to/file')),
  save: jest.fn(() => Promise.resolve('/mock/path/to/save')),
}));

// グローバルなモック
global.ResizeObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));

// コンソールエラーのモック（テスト中のエラー出力を抑制）
const originalConsoleError = console.error;
console.error = (...args) => {
  // React Testing Libraryの警告を無視
  if (
    args[0].includes('Warning: ReactDOM.render is no longer supported') ||
    args[0].includes('Warning: The current testing environment is not configured')
  ) {
    return;
  }
  originalConsoleError(...args);
};