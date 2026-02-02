/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_APP_TITLE: string;
  readonly TAURI_DEBUG: string;
  readonly TAURI_PLATFORM: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

// Tauri window object
declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}
