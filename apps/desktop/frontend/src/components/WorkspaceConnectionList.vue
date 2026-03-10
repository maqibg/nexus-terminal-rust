<template>
  <div class="ws-conn-list">
    <div class="search-wrap">
      <input
        ref="searchInputRef"
        v-model="searchTerm"
        class="search-input"
        type="text"
        placeholder="搜索名称或主机..."
        data-focus-id="connectionListSearch"
      />
      <i class="fas fa-search search-icon"></i>
    </div>

    <div v-if="groupedConnections.length === 0" class="empty-state">暂无匹配连接</div>

    <div v-for="group in groupedConnections" :key="group.key" class="ws-conn-group">
      <button class="ws-group-header" type="button" @click="toggleGroup(group.key)">
        <i :class="['fas', isGroupExpanded(group.key) ? 'fa-chevron-down' : 'fa-chevron-right']"></i>
        <span class="ws-group-name">{{ group.label }}</span>
        <span class="ws-group-count">({{ group.connections.length }})</span>
      </button>

      <div v-show="isGroupExpanded(group.key)" class="ws-group-items">
        <button
          v-for="conn in group.connections"
          :key="conn.id"
          class="ws-conn-item"
          type="button"
          @click="emit('select', conn)"
        >
          <span class="ws-conn-name">{{ conn.name || conn.host }}</span>
          <span class="ws-conn-host">{{ (conn.type || 'SSH').toUpperCase() }} · {{ conn.host }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useConnectionsStore } from '@/stores/connections';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import type { Connection } from '@/lib/api';

interface ConnectionGroup {
  key: string;
  label: string;
  connections: Connection[];
}

const emit = defineEmits<{ select: [conn: Connection] }>();

const store = useConnectionsStore();
const focusSwitcherStore = useFocusSwitcherStore();
const { list, tags } = storeToRefs(store);

const searchInputRef = ref<HTMLInputElement>();
const searchTerm = ref('');
const expandedGroups = ref<Record<string, boolean>>({});
let unregisterFocusAction: (() => void) | null = null;

function matchesQuery(conn: Connection, query: string) {
  if (!query) {
    return true;
  }

  const fields = [conn.name, conn.host, conn.type, conn.username];
  return fields.some((field) => String(field ?? '').toLowerCase().includes(query));
}

const groupedConnections = computed<ConnectionGroup[]>(() => {
  const query = searchTerm.value.toLowerCase().trim();
  const groupMap = new Map<string, ConnectionGroup>();

  const ensureGroup = (key: string, label: string) => {
    if (!groupMap.has(key)) {
      groupMap.set(key, { key, label, connections: [] });
    }
    return groupMap.get(key)!;
  };

  for (const conn of list.value) {
    if (!matchesQuery(conn, query)) {
      continue;
    }

    const connTags = Array.isArray(conn.tags) ? [...new Set(conn.tags.map((name) => String(name).trim()).filter(Boolean))] : [];
    if (connTags.length === 0) {
      ensureGroup('untagged', '未标记').connections.push(conn);
      continue;
    }

    for (const tagName of connTags) {
      ensureGroup(`tag:${tagName}`, tagName).connections.push(conn);
    }
  }

  const result: ConnectionGroup[] = [];
  const untagged = groupMap.get('untagged');
  if (untagged) {
    result.push(untagged);
    groupMap.delete('untagged');
  }

  for (const tag of tags.value) {
    const key = `tag:${tag.name}`;
    const group = groupMap.get(key);
    if (group) {
      result.push(group);
      groupMap.delete(key);
    }
  }

  result.push(...Array.from(groupMap.values()).sort((a, b) => a.label.localeCompare(b.label, 'zh-CN')));

  return result;
});

function isGroupExpanded(groupKey: string) {
  return expandedGroups.value[groupKey] ?? true;
}

function toggleGroup(groupKey: string) {
  expandedGroups.value[groupKey] = !isGroupExpanded(groupKey);
}

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
  padding: 8px;
  display: flex;
  flex-direction: column;
  max-height: 100%;
  overflow: auto;
}

.search-wrap {
  position: sticky;
  top: 0;
  z-index: 2;
  display: flex;
  align-items: center;
  margin-bottom: 8px;
  background: var(--bg-mantle);
  padding-bottom: 4px;
}

.search-input {
  width: 100%;
  height: 30px;
  padding: 0 28px 0 10px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  font-size: calc(12px + var(--ui-font-size-offset));
  outline: none;
}

.search-input:focus {
  border-color: var(--blue);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.search-icon {
  position: absolute;
  right: 9px;
  font-size: calc(11px + var(--ui-font-size-offset));
  color: var(--text-dim);
}

.ws-conn-group {
  margin-bottom: 4px;
}

.ws-group-header {
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--text-sub);
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  font-size: calc(12px + var(--ui-font-size-offset));
  text-align: left;
  cursor: pointer;
}

.ws-group-header:hover {
  color: var(--text);
}

.ws-group-name {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.ws-group-count {
  color: var(--text-dim);
}

.ws-group-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 2px;
}

.ws-conn-item {
  width: 100%;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  padding: 7px 10px;
  text-align: left;
}

.ws-conn-item:hover {
  background: var(--bg-surface0);
  border-color: var(--border);
}

.ws-conn-name {
  color: var(--text);
  width: 100%;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: calc(13px + var(--ui-font-size-offset));
}

.ws-conn-host {
  color: var(--text-dim);
  width: 100%;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: calc(11px + var(--ui-font-size-offset));
}

.empty-state {
  padding: 20px 0;
  text-align: center;
  color: var(--text-dim);
  font-size: calc(12px + var(--ui-font-size-offset));
}
</style>
