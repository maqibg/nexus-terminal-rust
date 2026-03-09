import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { historyApi, type CommandHistory } from '@/lib/api';

export const useCommandHistoryStore = defineStore('commandHistory', () => {
  const items = ref<CommandHistory[]>([]);
  const loading = ref(false);
  const searchTerm = ref('');
  const selectedIndex = ref(-1);

  const filteredItems = computed(() => {
    const term = searchTerm.value.trim().toLowerCase();
    if (!term) {
      return items.value;
    }
    return items.value.filter((entry) => entry.command.toLowerCase().includes(term));
  });

  async function fetchAll(limit?: number, offset?: number) {
    loading.value = true;
    try {
      items.value = await historyApi.list(limit, offset);
    } finally {
      loading.value = false;
    }
  }

  async function add(command: string, sessionId?: string, connectionId?: number) {
    await historyApi.add(command, sessionId, connectionId);
  }

  async function remove(id: number) {
    const deleted = await historyApi.delete(id);
    if (deleted) {
      items.value = items.value.filter((item) => item.id !== id);
    }
    return deleted;
  }

  async function clear() {
    await historyApi.clear();
    items.value = [];
    selectedIndex.value = -1;
  }

  function setSearchTerm(value: string) {
    searchTerm.value = value;
    selectedIndex.value = -1;
  }

  function selectNext() {
    const list = filteredItems.value;
    if (!list.length) {
      selectedIndex.value = -1;
      return;
    }
    selectedIndex.value = (selectedIndex.value + 1) % list.length;
  }

  function selectPrevious() {
    const list = filteredItems.value;
    if (!list.length) {
      selectedIndex.value = -1;
      return;
    }
    selectedIndex.value = (selectedIndex.value - 1 + list.length) % list.length;
  }

  function resetSelection() {
    selectedIndex.value = -1;
  }

  return {
    items,
    loading,
    searchTerm,
    selectedIndex,
    filteredItems,
    fetchAll,
    add,
    remove,
    clear,
    setSearchTerm,
    selectNext,
    selectPrevious,
    resetSelection,
  };
});
