<template>
  <div class="pane-title-bar">
    <span class="pane-icon">{{ icon }}</span>
    <span class="pane-label">{{ label }}</span>
    <button class="btn-icon" @click="$emit('toggle-maximize')" :title="maximized ? '还原' : '最大化'">
      {{ maximized ? '⊡' : '⊞' }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{ component: string; maximized?: boolean }>();
defineEmits<{ 'toggle-maximize': [] }>();

const icon = computed(() => {
  const map: Record<string, string> = { terminal: '⬛', sftp: '📁', editor: '📝' };
  return map[props.component] ?? '◻';
});
const label = computed(() => {
  const map: Record<string, string> = { terminal: '终端', sftp: '文件管理', editor: '编辑器' };
  return map[props.component] ?? props.component;
});
</script>

<style scoped>
.pane-title-bar { display: flex; align-items: center; gap: 6px; padding: 2px 8px; background: var(--bg-mantle); border-bottom: 1px solid var(--border); font-size: 12px; color: var(--text-sub); min-height: 24px; }
.pane-icon { font-size: 11px; }
.pane-label { flex: 1; }
.btn-icon { width: 20px; height: 20px; border-radius: 3px; border: 1px solid var(--border); background: transparent; color: var(--text-sub); cursor: pointer; font-size: 11px; display: flex; align-items: center; justify-content: center; }
.btn-icon:hover { background: var(--bg-surface1); }
</style>
