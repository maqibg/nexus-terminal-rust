import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';
import { readFileSync } from 'node:fs';

const FALLBACK_APP_VERSION = '0.1.0';

function normalizeVersion(value: unknown): string | null {
  if (typeof value !== 'string') {
    return null;
  }
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  return trimmed.replace(/^v/i, '');
}

function getAppVersion(): string {
  try {
    const raw = readFileSync(resolve(__dirname, '../src-tauri/tauri.conf.json'), 'utf-8');
    const parsed = JSON.parse(raw) as { version?: unknown };
    const normalized = normalizeVersion(parsed.version);
    if (normalized) {
      return normalized;
    }
  } catch {
    // ignore
  }

  try {
    const raw = readFileSync(resolve(__dirname, 'package.json'), 'utf-8');
    const parsed = JSON.parse(raw) as { version?: unknown };
    const normalized = normalizeVersion(parsed.version);
    if (normalized) {
      return normalized;
    }
  } catch {
    // ignore
  }

  return FALLBACK_APP_VERSION;
}

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  define: {
    'import.meta.env.VITE_APP_VERSION': JSON.stringify(getAppVersion()),
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'esnext',
  },
  optimizeDeps: {
    include: ['monaco-editor'],
  },
});
