import { defineStore } from 'pinia';
import { ref } from 'vue';
import { settingsApi } from '@/lib/api';

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Record<string, string>>({});
  const loaded = ref(false);

  async function loadAll() {
    const items = await settingsApi.getAll();
    const map: Record<string, string> = {};
    for (const item of items) map[item.key] = item.value;
    settings.value = map;
    loaded.value = true;
  }

  async function set(key: string, value: string) {
    await settingsApi.set(key, value);
    settings.value[key] = value;
  }

  function get(key: string, fallback = ''): string {
    return settings.value[key] ?? fallback;
  }

  return { settings, loaded, loadAll, set, get };
});
