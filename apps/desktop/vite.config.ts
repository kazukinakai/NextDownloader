import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

// Tauriの環境変数からホスト情報を取得
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  
  // パス解決の設定
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  
  // 開発サーバーの設定
  server: {
    host: host || false,
    port: 3000,
    strictPort: true,
    hmr: host
      ? {
          protocol: 'ws',
          host: host,
          port: 3001,
        }
      : undefined,
  },
  
  // ビルド設定
  build: {
    outDir: 'dist',
    target: 'esnext',
    minify: 'esbuild',
  },
});