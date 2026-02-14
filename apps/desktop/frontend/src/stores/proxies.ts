import { defineStore } from 'pinia';
import { ref } from 'vue';
import { connectionsApi, type Proxy } from '@/lib/api';

export const useProxiesStore = defineStore('proxies', () => {
  const items = ref<Proxy[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try { items.value = await connectionsApi.proxyList(); }
    finally { loading.value = false; }
  }

  async function create(data: Record<string, unknown>) {
    await connectionsApi.proxyCreate(data);
    await fetchAll();
  }

  async function update(data: Record<string, unknown>) {
    await connectionsApi.proxyUpdate(data);
    await fetchAll();
  }

  async function remove(id: number) {
    await connectionsApi.proxyDelete(id);
    items.value = items.value.filter(p => p.id !== id);
  }

  return { items, loading, fetchAll, create, update, remove };
});
