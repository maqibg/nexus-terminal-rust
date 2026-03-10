<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="$emit('close')">
      <div class="modal-card">
        <div class="modal-header">
          <span>标签-连接管理</span>
          <span class="close-btn" @click="$emit('close')">&times;</span>
        </div>
        <div class="body">
          <div class="tag-list">
            <div v-for="tag in tags" :key="tag.id" class="tag-item" :class="{ active: selectedTag?.id === tag.id }" @click="selectTag(tag)">
              {{ tag.name }}
            </div>
            <div v-if="!tags.length" class="empty">暂无标签</div>
          </div>
          <div class="conn-panel">
            <template v-if="selectedTag">
              <div class="panel-title">{{ selectedTag.name }} 下的连接</div>
              <div v-for="c in connections" :key="c.id" class="conn-row">
                <label><input type="checkbox" :checked="isAssigned(c)" @change="toggle(c)" /> {{ c.name }}</label>
              </div>
            </template>
            <div v-else class="empty">选择左侧标签</div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { connectionsApi, type Tag, type Connection } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

defineProps<{ visible: boolean }>();
defineEmits<{ close: [] }>();
const notify = useUINotificationStore();

const tags = ref<Tag[]>([]);
const connections = ref<Connection[]>([]);
const selectedTag = ref<Tag | null>(null);

onMounted(async () => {
  [tags.value, connections.value] = await Promise.all([connectionsApi.tagList(), connectionsApi.list()]);
});

function selectTag(tag: Tag) { selectedTag.value = tag; }

function isAssigned(conn: Connection): boolean {
  return selectedTag.value ? conn.tags.includes(selectedTag.value.name) : false;
}

async function toggle(conn: Connection) {
  if (!selectedTag.value) return;
  const tagName = selectedTag.value.name;
  const newTags = isAssigned(conn) ? conn.tags.filter(t => t !== tagName) : [...conn.tags, tagName];
  try {
    await connectionsApi.update(conn.id, { name: conn.name, host: conn.host, port: conn.port, username: conn.username, auth_method: conn.auth_method, tags: newTags });
    conn.tags = newTags;
  } catch (e: any) { notify.addNotification('error', e.message); }
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.modal-card { background: var(--bg-surface0); border-radius: 8px; width: 560px; max-height: 70vh; display: flex; flex-direction: column; border: 1px solid var(--border); overflow: hidden; }
.modal-header { display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; font-size: calc(16px + var(--ui-font-size-offset)); font-weight: 600; border-bottom: 1px solid var(--border); }
.close-btn { cursor: pointer; font-size: calc(20px + var(--ui-font-size-offset)); color: var(--text-dim); }
.close-btn:hover { color: var(--red); }
.body { display: flex; flex: 1; overflow: hidden; }
.tag-list { width: 160px; border-right: 1px solid var(--border); overflow-y: auto; padding: 4px; }
.tag-item { padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: calc(13px + var(--ui-font-size-offset)); color: var(--text); }
.tag-item:hover { background: var(--bg-mantle); }
.tag-item.active { background: var(--blue); color: var(--bg-base); }
.conn-panel { flex: 1; padding: 12px; overflow-y: auto; }
.panel-title { font-size: calc(13px + var(--ui-font-size-offset)); font-weight: 600; margin-bottom: 8px; }
.conn-row { padding: 4px 0; }
.conn-row label { display: flex; align-items: center; gap: 6px; font-size: calc(13px + var(--ui-font-size-offset)); cursor: pointer; color: var(--text); }
.empty { text-align: center; color: var(--text-dim); font-size: calc(13px + var(--ui-font-size-offset)); padding: 16px; }
</style>
