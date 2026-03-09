import { defineStore } from 'pinia';
import { ref } from 'vue';
import { connectionsApi, type Connection, type Tag } from '@/lib/api';
import { useUiNotificationsStore } from './uiNotifications';
import { toAppError } from '@/lib/errors';

export const useConnectionsStore = defineStore('connections', () => {
  const list = ref<Connection[]>([]);
  const tags = ref<Tag[]>([]);
  const loading = ref(false);
  let fetchPromise: Promise<void> | null = null;

  async function fetch() {
    if (fetchPromise) {
      return fetchPromise;
    }

    fetchPromise = (async () => {
      loading.value = true;
      try {
        const [conns, t] = await Promise.all([connectionsApi.list(), connectionsApi.tagList()]);
        list.value = conns;
        tags.value = t;
      } catch (e: unknown) {
        const ui = useUiNotificationsStore();
        ui.addNotification('error', toAppError(e).message);
      } finally {
        loading.value = false;
      }
    })();

    try {
      await fetchPromise;
    } finally {
      fetchPromise = null;
    }
  }

  async function remove(id: number) {
    try {
      await connectionsApi.delete(id);
      list.value = list.value.filter(c => c.id !== id);
    } catch (e: unknown) {
      const ui = useUiNotificationsStore();
      ui.addNotification('error', toAppError(e).message);
    }
  }

  function getTagsForConnection(conn: Connection): Tag[] {
    if (!conn.tags?.length) return [];
    return tags.value.filter(t => conn.tags.includes(t.name));
  }

  return { list, tags, loading, fetch, remove, getTagsForConnection };
});
