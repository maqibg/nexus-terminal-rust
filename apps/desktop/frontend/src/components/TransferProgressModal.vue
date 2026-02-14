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
            <button v-if="t.status === 'active'" class="btn-cancel" @click="$emit('cancel', t.id)">取消</button>
          </div>
        </div>
        <div class="transfer-actions">
          <button class="btn" @click="$emit('close')">关闭</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
export interface TransferTask {
  id: string;
  kind: 'upload' | 'download';
  fileName: string;
  totalBytes: number;
  transferredBytes: number;
  percent: number;
  status: 'active' | 'completed' | 'failed' | 'cancelled';
}

defineProps<{ visible: boolean; tasks: TransferTask[] }>();
defineEmits<{ close: []; cancel: [id: string] }>();

const statusLabel: Record<string, string> = {
  active: '传输中', completed: '完成', failed: '失败', cancelled: '已取消',
};
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.transfer-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; min-width: 440px; max-height: 60vh; overflow-y: auto; border: 1px solid var(--border); }
.transfer-title { font-size: 16px; font-weight: 600; margin-bottom: 16px; }
.empty { color: var(--text-dim); font-size: 13px; text-align: center; padding: 20px; }
.transfer-item { margin-bottom: 12px; }
.transfer-info { display: flex; justify-content: space-between; margin-bottom: 4px; }
.file-name { font-size: 13px; color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.transfer-status { font-size: 11px; }
.transfer-status.active { color: var(--blue); }
.transfer-status.completed { color: var(--green); }
.transfer-status.failed { color: var(--red); }
.progress-bar { height: 3px; background: var(--bg-surface1); border-radius: 2px; overflow: hidden; }
.progress-fill { height: 100%; background: var(--blue); transition: width 0.2s; }
.transfer-meta { display: flex; justify-content: space-between; align-items: center; margin-top: 4px; font-size: 11px; color: var(--text-dim); }
.btn-cancel { background: none; border: none; color: var(--red); cursor: pointer; font-size: 11px; }
.transfer-actions { display: flex; justify-content: flex-end; margin-top: 16px; }
.btn { padding: 6px 16px; border-radius: 4px; border: 1px solid var(--border); background: var(--bg-surface1); color: var(--text); cursor: pointer; font-size: 13px; }
</style>
