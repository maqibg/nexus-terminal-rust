<template>
  <div class="tab-bar">
    <div class="tabs">
      <div
        v-for="s in sessions"
        :key="s.id"
        class="tab"
        :class="{ active: s.id === activeSessionId }"
        @click="emit('activate', s.id)"
        @contextmenu.prevent="onContext($event, s.id)"
      >
        <span class="tab-status" :class="s.status"></span>
        <span class="tab-name">{{ s.connectionName }}</span>
        <button class="tab-close" @click.stop="emit('close', s.id)">&times;</button>
      </div>
    </div>

    <div class="tab-actions">
      <button class="tab-action" :title="headerToggleTitle" @click="emit('toggleHeader')">
        <i :class="headerToggleIconClass"></i>
      </button>

      <button class="tab-action" title="查看传输进度" @click="emit('openTransfers')">
        <i class="fas fa-tasks"></i>
      </button>

      <button class="tab-action" title="配置布局" @click="emit('openLayoutConfigurator')">
        <i class="fas fa-th-large"></i>
      </button>

    </div>

    <TabBarContextMenu
      :visible="ctx.visible"
      :x="ctx.x"
      :y="ctx.y"
      :session-id="ctx.sessionId"
      @close="ctx.visible = false"
      @action="handleContextAction"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, reactive } from 'vue';
import type { SessionInfo } from '@/stores/session';
import TabBarContextMenu from './TabBarContextMenu.vue';

const props = defineProps<{
  sessions: SessionInfo[];
  activeSessionId: string | null;
  headerVisible?: boolean;
}>();

const emit = defineEmits<{
  activate: [id: string];
  close: [id: string];
  add: [];
  closeOthers: [id: string];
  closeRight: [id: string];
  closeLeft: [id: string];
  openTransfers: [];
  toggleHeader: [];
  openLayoutConfigurator: [];
}>();

const ctx = reactive({ visible: false, x: 0, y: 0, sessionId: '' });

const isHeaderVisible = computed(() => props.headerVisible !== false);
const headerToggleIconClass = computed(() => (isHeaderVisible.value ? 'fas fa-eye' : 'fas fa-eye-slash'));
const headerToggleTitle = computed(() => (isHeaderVisible.value ? '隐藏顶部导航' : '显示顶部导航'));

function onContext(e: MouseEvent, id: string) {
  ctx.visible = true;
  ctx.x = e.clientX;
  ctx.y = e.clientY;
  ctx.sessionId = id;
}

function handleContextAction(type: string) {
  const sid = ctx.sessionId;
  ctx.visible = false;
  if (!sid) return;

  if (type === 'close') emit('close', sid);
  else if (type === 'close-others') emit('closeOthers', sid);
  else if (type === 'close-right') emit('closeRight', sid);
  else if (type === 'close-left') emit('closeLeft', sid);
  else if (type === 'duplicate') emit('add');
}
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  background: var(--bg-mantle);
  border-bottom: 1px solid var(--border);
  height: 36px;
  padding: 0 4px;
  flex-shrink: 0;
}

.tabs {
  display: flex;
  flex: 1;
  overflow-x: auto;
  gap: 2px;
  scrollbar-width: none;
}

.tabs::-webkit-scrollbar {
  display: none;
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  font-size: 12px;
  color: var(--text-sub);
  cursor: pointer;
  border-radius: 4px 4px 0 0;
  white-space: nowrap;
  transition: background 0.15s;
}

.tab:hover {
  background: var(--bg-surface1);
}

.tab.active {
  background: var(--bg-surface0);
  color: var(--text);
}

.tab-status {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.tab-status.connected {
  background: var(--color-success);
}

.tab-status.connecting {
  background: var(--color-warning);
}

.tab-status.disconnected {
  background: var(--color-error);
}

.tab-close {
  background: none;
  border: none;
  color: var(--text-sub);
  cursor: pointer;
  font-size: 14px;
  padding: 0 2px;
  line-height: 1;
}

.tab-close:hover {
  color: var(--text);
}

.tab-actions {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.tab-action {
  background: none;
  border: none;
  border-left: 1px solid var(--border);
  color: var(--text-sub);
  cursor: pointer;
  font-size: 14px;
  padding: 4px 10px;
  height: 36px;
  min-width: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.tab-action:hover {
  color: var(--text);
  background: var(--link-active-bg-color);
}

.tab-action i {
  font-size: 13px;
}
</style>
