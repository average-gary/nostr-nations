import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],

  // Resolve aliases for cleaner imports
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },

  // Vite options tailored for Tauri development
  clearScreen: false,

  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Watch for file changes in these directories
      ignored: ['**/src-tauri/**'],
    },
  },

  // Build configuration
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM === 'windows'
      ? 'chrome105'
      : 'safari13',
    // Produce sourcemaps for error reporting
    sourcemap: !!process.env.TAURI_DEBUG,
    // Output to dist directory (relative to project root)
    outDir: 'dist',
    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
  },

  // Environment variables prefixes
  envPrefix: ['VITE_', 'TAURI_'],
});
