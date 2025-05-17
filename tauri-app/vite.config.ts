import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  
  // Vite の設定
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  
  // Tauri プラグインの設定
  server: {
    port: 3000,
    strictPort: true,
  },
  
  // ビルド設定
  build: {
    outDir: 'dist',
    target: 'esnext',
    minify: 'esbuild',
  },
});