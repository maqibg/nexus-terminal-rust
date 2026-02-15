<template>
  <div class="favorite-paths-dropdown">
    <div class="favorite-toolbar">
      <input
        v-model="searchTerm"
        type="text"
        class="favorite-search"
        placeholder="搜索名称或路径..."
      />
      <button class="toolbar-btn" :title="sortButtonTitle" @click="toggleSort">
        <i class="fas" :class="sortButtonIcon"></i>
      </button>
      <button class="toolbar-btn add-btn" title="添加收藏路径" @click="openAddModal">
        <i class="fas fa-plus"></i>
      </button>
    </div>

    <div class="favorite-list">
      <div v-if="favoritePathsStore.loading && filteredPaths.length === 0" class="favorite-state">
        <i class="fas fa-spinner fa-spin"></i>
        <span>加载收藏路径...</span>
      </div>

      <div v-else-if="!favoritePathsStore.loading && filteredPaths.length === 0" class="favorite-state">
        <i class="fas fa-star-half-alt"></i>
        <span>{{ searchTerm.trim() ? '没有匹配的收藏路径' : '暂无收藏路径' }}</span>
      </div>

      <ul v-else class="favorite-items">
        <li
          v-for="item in filteredPaths"
          :key="item.id"
          class="favorite-item"
          :title="item.path"
          @click="handleItemClick(item)"
        >
          <div class="favorite-main">
            <p class="favorite-name">{{ item.name || item.path }}</p>
            <p v-if="item.name" class="favorite-path">{{ item.path }}</p>
          </div>
          <div class="favorite-actions">
            <button
              class="item-btn"
              title="发送到终端"
              @click.stop="handleSendToTerminal(item)"
            >
              <i class="fas fa-terminal"></i>
            </button>
            <button class="item-btn" title="编辑" @click.stop="openEditModal(item)">
              <i class="fas fa-pen"></i>
            </button>
            <button class="item-btn danger" title="删除" @click.stop="handleDelete(item)">
              <i class="fas fa-trash-alt"></i>
            </button>
          </div>
        </li>
      </ul>
    </div>

    <AddEditFavoritePathForm
      :visible="showAddEditModal"
      :edit-data="editingPathItem"
      :connection-id="connectionId"
      @close="showAddEditModal = false"
      @save="handleFormSave"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import AddEditFavoritePathForm from '@/components/AddEditFavoritePathForm.vue';
import { sshApi, type FavoritePath } from '@/lib/api';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { useFavoritePathsStore } from '@/stores/favoritePaths';
import { useSessionStore } from '@/stores/session';
import { useUINotificationStore } from '@/stores/uiNotifications';

const props = defineProps<{ connectionId?: number }>();
const emit = defineEmits<{ navigate: [path: string]; close: []; modalVisibilityChange: [visible: boolean] }>();

const favoritePathsStore = useFavoritePathsStore();
const sessionStore = useSessionStore();
const notificationsStore = useUINotificationStore();
const { confirm } = useConfirmDialog();

const searchTerm = ref('');
const sortBy = ref<'name' | 'last_used_at'>('name');
const showAddEditModal = ref(false);
const editingPathItem = ref<FavoritePath | null>(null);

const sortButtonIcon = computed(() =>
  sortBy.value === 'name' ? 'fa-sort-alpha-down' : 'fa-clock',
);

const sortButtonTitle = computed(() =>
  sortBy.value === 'name' ? '按名称排序' : '按最近使用排序',
);

const filteredPaths = computed(() => {
  const keyword = searchTerm.value.trim().toLowerCase();
  const filtered = favoritePathsStore.items.filter((item) => {
    if (!keyword) {
      return true;
    }
    return item.path.toLowerCase().includes(keyword) || item.name.toLowerCase().includes(keyword);
  });

  return filtered.sort((a, b) => {
    if (sortBy.value === 'name') {
      return a.name.localeCompare(b.name, 'zh-Hans-CN', { sensitivity: 'base' });
    }

    const timeA = a.last_used_at ? new Date(a.last_used_at).getTime() : 0;
    const timeB = b.last_used_at ? new Date(b.last_used_at).getTime() : 0;
    if (timeA !== timeB) {
      return timeB - timeA;
    }

    return a.name.localeCompare(b.name, 'zh-Hans-CN', { sensitivity: 'base' });
  });
});

async function load() {
  await favoritePathsStore.fetchAll(props.connectionId);
}

function toggleSort() {
  sortBy.value = sortBy.value === 'name' ? 'last_used_at' : 'name';
}

function openAddModal() {
  editingPathItem.value = null;
  showAddEditModal.value = true;
}

function openEditModal(item: FavoritePath) {
  editingPathItem.value = { ...item };
  showAddEditModal.value = true;
}

async function handleDelete(item: FavoritePath) {
  const confirmed = await confirm('删除收藏路径', `确定删除“${item.name || item.path}”吗？`);
  if (!confirmed) {
    return;
  }

  await favoritePathsStore.remove(item.id);
}

async function handleFormSave(payload: { name: string; path: string }) {
  const path = payload.path.trim();
  if (!path) {
    return;
  }

  const normalizedName = payload.name.trim() || path;

  if (editingPathItem.value) {
    await favoritePathsStore.update(editingPathItem.value.id, normalizedName, path, props.connectionId);
  } else {
    await favoritePathsStore.create(normalizedName, path, props.connectionId);
  }

  showAddEditModal.value = false;
  editingPathItem.value = null;
}

async function markUsed(itemId: number) {
  try {
    await favoritePathsStore.markUsed(itemId);
  } catch {
    // ignore
  }
}

async function handleItemClick(item: FavoritePath) {
  await markUsed(item.id);
  emit('navigate', item.path);
}

async function handleSendToTerminal(item: FavoritePath) {
  const sessionId = sessionStore.activeSessionId;
  if (!sessionId) {
    notificationsStore.addNotification('warning', '没有活动会话，无法发送到终端');
    return;
  }

  const escapedPath = item.path.replace(/"/g, '\\"');
  const command = `cd \"${escapedPath}\"`;
  const data = btoa(unescape(encodeURIComponent(`${command}\n`)));

  try {
    await sshApi.write(sessionId, data);
    await markUsed(item.id);
    notificationsStore.addNotification('success', '已发送到终端');
    emit('close');
  } catch {
    notificationsStore.addNotification('error', '发送失败，请稍后重试');
  }
}

watch(
  () => props.connectionId,
  () => {
    void load();
  },
  { immediate: true },
);

watch(
  () => showAddEditModal.value,
  (visible) => {
    emit('modalVisibilityChange', visible);
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  emit('modalVisibilityChange', false);
});
</script>

<style scoped>
.favorite-paths-dropdown {
  width: 100%;
  max-height: 320px;
  display: flex;
  flex-direction: column;
  background: var(--bg-surface0, #313244);
}

.favorite-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border-bottom: 1px solid rgba(69, 71, 90, 0.7);
}

.favorite-search {
  flex: 1;
  min-width: 0;
  height: 32px;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  font-size: 13px;
  padding: 0 10px;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.favorite-search:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 1px rgba(137, 180, 250, 0.28);
}

.favorite-search::placeholder {
  color: var(--text-dim, #6c7086);
}

.toolbar-btn {
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  border: 1px solid var(--border, #45475a);
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.15s;
}

.toolbar-btn:hover {
  background: rgba(137, 180, 250, 0.1);
  color: var(--text, #cdd6f4);
}

.toolbar-btn.add-btn {
  background: var(--blue, #89b4fa);
  color: var(--bg-base, #1e1e2e);
  border-color: var(--blue, #89b4fa);
}

.toolbar-btn.add-btn:hover {
  opacity: 0.88;
}

.favorite-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.favorite-state {
  min-height: 112px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-dim, #6c7086);
  font-size: 13px;
}

.favorite-state i {
  font-size: 15px;
  opacity: 0.8;
}

.favorite-items {
  margin: 0;
  padding: 0;
  list-style: none;
}

.favorite-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.favorite-item:hover {
  background: rgba(137, 180, 250, 0.1);
}

.favorite-main {
  min-width: 0;
  flex: 1;
}

.favorite-name,
.favorite-path {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.favorite-name {
  color: var(--text, #cdd6f4);
  font-size: 13px;
  font-weight: 600;
}

.favorite-path {
  margin-top: 2px;
  color: var(--text-dim, #6c7086);
  font-size: 12px;
}

.favorite-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
}

.favorite-item:hover .favorite-actions,
.favorite-item:focus-within .favorite-actions {
  opacity: 1;
}

.item-btn {
  width: 25px;
  height: 25px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 11px;
}

.item-btn:hover {
  background: rgba(137, 180, 250, 0.18);
  color: var(--blue, #89b4fa);
}

.item-btn.danger:hover {
  background: rgba(243, 139, 168, 0.18);
  color: var(--red, #f38ba8);
}
</style>
