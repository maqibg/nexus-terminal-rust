<template>
  <div class="ws-conn-list">
    <div class="search-wrap">
      <input
        ref="searchInputRef"
        v-model="searchTerm"
        class="search-input"
        type="text"
        placeholder="搜索连接名称或地址..."
        data-focus-id="connectionListSearch"
      />
      <i class="fas fa-search search-icon"></i>
    </div>

    <div v-if="filteredList.length === 0" class="empty-state">暂无匹配连接</div>

    <div
      v-for="conn in filteredList"
      :key="conn.id"
      class="ws-conn-item"
      @click="emit('select', conn)"
    >
      <span class="ws-conn-name">{{ conn.name }}</span>
      <span class="ws-conn-host">{{ (conn.type || 'SSH') }} · {{ conn.host }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useConnectionsStore } from '@/stores/connections';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import type { Connection } from '@/lib/api';

const emit = defineEmits<{ select: [conn: Connection] }>();

const store = useConnectionsStore();
const focusSwitcherStore = useFocusSwitcherStore();
const { list } = storeToRefs(store);

const searchInputRef = ref<HTMLInputElement>();
const searchTerm = ref('');
let unregisterFocusAction: (() => void) | null = null;

const filteredList = computed(() => {
  const q = searchTerm.value.toLowerCase().trim();
  if (!q) {
    return list.value;
  }

  return list.value.filter((conn) => {
    const fields = [conn.name, conn.host, conn.type];
    return fields.some((field) => String(field ?? '').toLowerCase().includes(q));
  });
});

function isVisibleInput(input: HTMLInputElement | undefined): input is HTMLInputElement {
  if (!input || !input.isConnected || input.disabled) {
    return false;
  }
  const style = window.getComputedStyle(input);
  if (style.display === 'none' || style.visibility === 'hidden') {
    return false;
  }
  const rect = input.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0;
}

function focusSearchInput(): boolean | undefined {
  if (!isVisibleInput(searchInputRef.value)) {
    return undefined;
  }

  searchInputRef.value.focus();
  searchInputRef.value.select();
  return document.activeElement === searchInputRef.value;
}

onMounted(() => {
  store.fetch();
  unregisterFocusAction = focusSwitcherStore.registerFocusAction('connectionListSearch', focusSearchInput);
});

onUnmounted(() => {
  unregisterFocusAction?.();
  unregisterFocusAction = null;
});
</script>

<style scoped>
.ws-conn-list {
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 100%;
  overflow: auto;
}

.search-wrap {
  position: sticky;
  top: 0;
  z-index: 1;
  display: flex;
  align-items: center;
  margin-bottom: 6px;
}

.search-input {
  width: 100%;
  height: 30px;
  padding: 0 28px 0 10px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  font-size: 12px;
  outline: none;
}

.search-input:focus {
  border-color: var(--blue);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.search-icon {
  position: absolute;
  right: 9px;
  font-size: 11px;
  color: var(--text-dim);
}

.ws-conn-item {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  padding: 7px 10px;
  cursor: pointer;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid transparent;
}

.ws-conn-item:hover {
  background: var(--bg-surface0);
  border-color: var(--border);
}

.ws-conn-name {
  color: var(--text);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ws-conn-host {
  color: var(--text-dim);
  font-size: 11px;
  flex-shrink: 0;
}

.empty-state {
  padding: 20px 0;
  text-align: center;
  color: var(--text-dim);
  font-size: 12px;
}
</style>