import { defineStore } from 'pinia';
import { ref } from 'vue';
import { settingsApi } from '@/lib/api';

export const useAppearanceStore = defineStore('appearance', () => {
  const appearance = ref<Record<string, string>>({});
  const loaded = ref(false);

  async function loadAll() {
    const items = await settingsApi.appearanceGetAll();
    const map: Record<string, string> = {};
    for (const item of items) map[item.key] = item.value;
    appearance.value = map;
    loaded.value = true;
    applyTheme();
  }

  async function set(key: string, value: string) {
    await settingsApi.appearanceSet(key, value);
    appearance.value[key] = value;
    applyTheme();
  }

  function get(key: string, fallback = ''): string {
    return appearance.value[key] ?? fallback;
  }

  function applyTheme() {
    const root = document.documentElement;
    for (const [key, value] of Object.entries(appearance.value)) {
      if (key.startsWith('--')) {
        root.style.setProperty(key, value);
      }
    }
  }

  return { appearance, loaded, loadAll, set, get, applyTheme };
});
