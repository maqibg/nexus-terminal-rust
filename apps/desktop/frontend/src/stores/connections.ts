import { defineStore } from 'pinia';
import { ref } from 'vue';
import { connectionsApi, type Connection, type Tag } from '@/lib/api';

export const useConnectionsStore = defineStore('connections', () => {
  const list = ref<Connection[]>([]);
  const tags = ref<Tag[]>([]);
  const loading = ref(false);

  async function fetch() {
    loading.value = true;
    try {
      const [conns, t] = await Promise.all([connectionsApi.list(), connectionsApi.tagList()]);
      list.value = conns;
      tags.value = t;
    } finally {
      loading.value = false;
    }
  }

  async function remove(id: number) {
    await connectionsApi.delete(id);
    list.value = list.value.filter(c => c.id !== id);
  }

  function getTagsForConnection(conn: Connection): Tag[] {
    if (!conn.tags?.length) return [];
    return tags.value.filter(t => conn.tags.includes(t.name));
  }

  return { list, tags, loading, fetch, remove, getTagsForConnection };
});
