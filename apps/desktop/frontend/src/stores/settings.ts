import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { settingsApi } from '@/lib/api';

const AVAILABLE_LOCALES = ['zh-CN'] as const;
type AppLocale = (typeof AVAILABLE_LOCALES)[number];

function normalizeLocale(_value: string | undefined | null): AppLocale {
  return 'zh-CN';
}

function applyDocumentLocale(locale: AppLocale): void {
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale;
  }
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Record<string, string>>({});
  const loaded = ref(false);
  let loadPromise: Promise<void> | null = null;

  const locale = computed<AppLocale>(() => 'zh-CN');

  async function loadAll() {
    if (loadPromise) {
      return loadPromise;
    }

    loadPromise = (async () => {
      const items = await settingsApi.getAll();
      const map: Record<string, string> = {};
      for (const item of items) {
        map[item.key] = item.value;
      }
      map.language = 'zh-CN';
      settings.value = map;
      loaded.value = true;
      applyDocumentLocale('zh-CN');
    })();

    try {
      await loadPromise;
    } finally {
      loadPromise = null;
    }
  }

  async function set(key: string, value: string) {
    const persistedValue = key === 'language' ? 'zh-CN' : value;
    await settingsApi.set(key, persistedValue);
    settings.value[key] = persistedValue;

    if (key === 'language') {
      applyDocumentLocale(normalizeLocale(persistedValue));
    }
  }

  function get(key: string, fallback = ''): string {
    return settings.value[key] ?? fallback;
  }

  function getBoolean(key: string, fallback = false): boolean {
    const raw = get(key, fallback ? 'true' : 'false').trim().toLowerCase();
    if (['1', 'true', 'yes', 'on'].includes(raw)) {
      return true;
    }
    if (['0', 'false', 'no', 'off'].includes(raw)) {
      return false;
    }
    return fallback;
  }

  function getInteger(key: string, fallback: number, min?: number): number {
    const parsed = Number.parseInt(get(key, String(fallback)), 10);
    if (!Number.isInteger(parsed)) {
      return fallback;
    }
    if (min !== undefined && parsed < min) {
      return fallback;
    }
    return parsed;
  }

  function setRuntimeLocale(_value: string): AppLocale {
    const normalized: AppLocale = 'zh-CN';
    settings.value.language = normalized;
    applyDocumentLocale(normalized);
    return normalized;
  }

  return {
    settings,
    loaded,
    locale,
    loadAll,
    set,
    get,
    getBoolean,
    getInteger,
    setRuntimeLocale,
  };
});
