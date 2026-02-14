import { defineStore } from 'pinia';
import { ref } from 'vue';
import { auditApi, type AuditLog } from '@/lib/api';

export const useAuditStore = defineStore('audit', () => {
  const items = ref<AuditLog[]>([]);
  const total = ref(0);
  const loading = ref(false);

  async function fetchAll(limit?: number, offset?: number) {
    loading.value = true;
    try {
      const [logs, count] = await Promise.all([
        auditApi.list(limit, offset),
        auditApi.count(),
      ]);
      items.value = logs;
      total.value = count;
    } finally {
      loading.value = false;
    }
  }

  async function clear() {
    await auditApi.clear();
    items.value = [];
    total.value = 0;
  }

  return { items, total, loading, fetchAll, clear };
});
