<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="$emit('close')">
      <div class="transfer-card">
        <div class="transfer-title">传输进度</div>
        <div v-if="!tasks.length" class="empty">无活跃传输</div>
        <div v-for="t in tasks" :key="t.id" class="transfer-item">
          <div class="transfer-info">
            <span class="file-name">{{ t.fileName }}</span>
            <span class="transfer-status" :class="t.status">{{ statusLabel[t.status] }}</span>
          </div>
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: t.percent + '%' }"></div>
          </div>
          <div class="transfer-meta">
            <span>{{ t.percent }}%</span>
            <button v-if="t.status === 'active' || t.status === 'paused'" class="btn-cancel" @click="$emit('cancel', t.id)">取消</button>
          </div>
        </div>
        <div class="transfer-actions">
          <button class="btn" :disabled="!hasRunningTasks" @click="handlePauseToggle">
            {{ allPaused ? '全部继续' : '全部暂停' }}
          </button>
          <button class="btn btn-danger" :disabled="!hasActiveTasks" @click="$emit('cancel-all')">全部取消</button>
          <button class="btn" :disabled="!hasCompletedTasks" @click="$emit('clear-completed')">清空已完成</button>
          <button class="btn" @click="$emit('close')">关闭</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { TransferTask } from '@/lib/api';

const props = defineProps<{ visible: boolean; tasks: TransferTask[] }>();
const emit = defineEmits<{
  close: [];
  cancel: [id: string];
  'pause-all': [];
  'resume-all': [];
  'cancel-all': [];
  'clear-completed': [];
}>();

const statusLabel: Record<string, string> = {
  active: '传输中', paused: '已暂停', completed: '完成', failed: '失败', cancelled: '已取消',
};

const hasCompletedTasks = computed(() =>
  props.tasks.some((task) => task.status === 'completed' || task.status === 'failed' || task.status === 'cancelled'),
);

const hasActiveTasks = computed(() =>
  props.tasks.some((task) => task.status === 'active' || task.status === 'paused'),
);

const hasRunningTasks = computed(() =>
  props.tasks.some((task) => task.status === 'active' || task.status === 'paused'),
);

const allPaused = computed(() =>
  hasRunningTasks.value && props.tasks.every((task) => task.status !== 'active'),
);

function handlePauseToggle(): void {
  if (allPaused.value) {
    emit('resume-all');
    return;
  }
  emit('pause-all');
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.transfer-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; min-width: 440px; max-height: 60vh; overflow-y: auto; border: 1px solid var(--border); }
.transfer-title { font-size: calc(16px + var(--ui-font-size-offset)); font-weight: 600; margin-bottom: 16px; }
.empty { color: var(--text-dim); font-size: calc(13px + var(--ui-font-size-offset)); text-align: center; padding: 20px; }
.transfer-item { margin-bottom: 12px; }
.transfer-info { display: flex; justify-content: space-between; margin-bottom: 4px; }
.file-name { font-size: calc(13px + var(--ui-font-size-offset)); color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.transfer-status { font-size: calc(11px + var(--ui-font-size-offset)); }
.transfer-status.active { color: var(--blue); }
.transfer-status.paused { color: var(--yellow); }
.transfer-status.completed { color: var(--green); }
.transfer-status.failed { color: var(--red); }
.progress-bar { height: 3px; background: var(--bg-surface1); border-radius: 2px; overflow: hidden; }
.progress-fill { height: 100%; background: var(--blue); transition: width 0.2s; }
.transfer-meta { display: flex; justify-content: space-between; align-items: center; margin-top: 4px; font-size: calc(11px + var(--ui-font-size-offset)); color: var(--text-dim); }
.btn-cancel { background: none; border: none; color: var(--red); cursor: pointer; font-size: calc(11px + var(--ui-font-size-offset)); }
.transfer-actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 18px; }
.btn { padding: 6px 16px; border-radius: 4px; border: 1px solid var(--border); background: var(--bg-surface1); color: var(--text); cursor: pointer; font-size: calc(13px + var(--ui-font-size-offset)); }
.btn-danger { color: var(--red); border-color: color-mix(in srgb, var(--red) 45%, var(--border)); }
.btn:disabled { opacity: 0.45; cursor: not-allowed; }
</style>
