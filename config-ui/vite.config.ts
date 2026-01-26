import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  build: {
    outDir: '../rust-mcp/static/dist',
    emptyOutDir: true,
    sourcemap: false,
  },
  optimizeDeps: {
    exclude: ['lucide-react'],
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3008',
        changeOrigin: true,
      },
    },
  },
  base: '/',
});
