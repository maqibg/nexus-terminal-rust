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

    <button class="tab-action" @click="emit('openTransfers')" title="传输进度">
      <i class="fas fa-exchange-alt"></i>
    </button>
    <button class="tab-add" @click="emit('add')" title="新建连接">+</button>

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
import { reactive } from 'vue';
import type { SessionInfo } from '@/stores/session';
import TabBarContextMenu from './TabBarContextMenu.vue';

defineProps<{ sessions: SessionInfo[]; activeSessionId: string | null }>();
const emit = defineEmits<{
  activate: [id: string];
  close: [id: string];
  add: [];
  closeOthers: [id: string];
  closeRight: [id: string];
  closeLeft: [id: string];
  openTransfers: [];
}>();

const ctx = reactive({ visible: false, x: 0, y: 0, sessionId: '' });

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
  background: var(--bg-mantle, #181825);
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
.tabs::-webkit-scrollbar { display: none; }
.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  font-size: 12px;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  border-radius: 4px 4px 0 0;
  white-space: nowrap;
  transition: background 0.15s;
}
.tab:hover { background: var(--bg-surface0, #313244); }
.tab.active {
  background: var(--bg-surface0, #313244);
  color: var(--text, #cdd6f4);
}
.tab-status {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.tab-status.connected { background: #a6e3a1; }
.tab-status.connecting { background: #f9e2af; }
.tab-status.disconnected { background: #f38ba8; }
.tab-close {
  background: none;
  border: none;
  color: var(--text-dim, #6c7086);
  cursor: pointer;
  font-size: 14px;
  padding: 0 2px;
  line-height: 1;
}
.tab-close:hover { color: var(--red, #f38ba8); }
.tab-action,
.tab-add {
  background: none;
  border: none;
  color: var(--text-dim, #6c7086);
  cursor: pointer;
  font-size: 14px;
  padding: 4px 10px;
  flex-shrink: 0;
}
.tab-add { font-size: 18px; }
.tab-action:hover,
.tab-add:hover { color: var(--blue, #89b4fa); }
</style>
