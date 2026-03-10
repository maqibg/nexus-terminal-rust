<template>
  <div class="audit-page">
    <div class="audit-header">
      <button class="btn-back" @click="$router.push('/')">← 返回</button>
      <h2>审计日志</h2>
      <span class="audit-count" v-if="total > 0">共 {{ total }} 条</span>
      <button class="btn-danger" @click="handleClear" v-if="total > 0">清空</button>
    </div>
    <div class="audit-body">
      <div v-if="loading" class="status">加载中...</div>
      <table v-else-if="logs.length">
        <thead>
          <tr><th>时间</th><th>操作</th><th>详情</th></tr>
        </thead>
        <tbody>
          <tr v-for="log in logs" :key="log.id">
            <td class="col-time">{{ log.timestamp }}</td>
            <td>{{ log.action_type }}</td>
            <td class="col-detail">{{ log.details ?? '' }}</td>
          </tr>
        </tbody>
      </table>
      <div v-else class="status">暂无日志</div>
      <div class="pager" v-if="logs.length">
        <button :disabled="offset === 0" @click="loadPage(offset - limit)">上一页</button>
        <button :disabled="logs.length < limit" @click="loadPage(offset + limit)">下一页</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { auditApi } from '@/lib/api';
import type { AuditLog } from '@/lib/api';

const logs = ref<AuditLog[]>([]);
const total = ref(0);
const loading = ref(false);
const limit = 50;
const offset = ref(0);

async function loadPage(newOffset: number) {
  offset.value = Math.max(0, newOffset);
  loading.value = true;
  try {
    logs.value = await auditApi.list(limit, offset.value);
  } catch { /* ignore */ }
  finally { loading.value = false; }
}

async function loadCount() {
  try { total.value = await auditApi.count(); } catch { /* ignore */ }
}

async function handleClear() {
  if (!confirm('确定清空所有审计日志？')) return;
  try { await auditApi.clear(); logs.value = []; total.value = 0; } catch { /* ignore */ }
}

onMounted(() => { loadPage(0); loadCount(); });
</script>

<style scoped>
.audit-page { display: flex; flex-direction: column; height: 100%; background: var(--bg-base); color: var(--text); }
.audit-header {
  display: flex; align-items: center; gap: 12px; padding: 12px 16px;
  border-bottom: 1px solid var(--border); background: var(--bg-surface0);
}
.audit-header h2 { margin: 0; font-size: calc(1rem + var(--ui-font-size-offset)); font-weight: 500; }
.audit-count { font-size: calc(0.8rem + var(--ui-font-size-offset)); color: var(--text-dim); margin-left: auto; }
.btn-back {
  background: none; border: none; color: var(--blue); cursor: pointer;
  font-size: calc(0.85rem + var(--ui-font-size-offset)); padding: 4px 8px; border-radius: 4px;
}
.btn-back:hover { background: var(--bg-surface1); }
.btn-danger {
  padding: 4px 10px; border-radius: 4px; border: none;
  background: var(--red); color: var(--button-text-color); cursor: pointer; font-size: calc(0.8rem + var(--ui-font-size-offset)); font-weight: 600;
}
.btn-danger:hover { filter: brightness(0.95); }
.audit-body { flex: 1; overflow-y: auto; padding: 8px 16px; }
table { width: 100%; border-collapse: collapse; font-size: calc(0.8rem + var(--ui-font-size-offset)); }
th { text-align: left; padding: 6px 8px; border-bottom: 1px solid var(--border); color: var(--text-sub); font-weight: 500; }
td { padding: 5px 8px; border-bottom: 1px solid color-mix(in srgb, var(--border) 75%, transparent); }
.col-time { white-space: nowrap; color: var(--text-dim); width: 160px; }
.col-detail { max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-dim); }
.status { padding: 1rem; text-align: center; color: var(--text-dim); font-size: calc(0.8rem + var(--ui-font-size-offset)); }
.pager { display: flex; justify-content: center; gap: 8px; padding: 12px; }
.pager button {
  padding: 4px 12px; border-radius: 4px; border: 1px solid var(--border);
  background: transparent; color: var(--text-sub); cursor: pointer; font-size: calc(0.8rem + var(--ui-font-size-offset));
}
.pager button:hover { background: var(--bg-surface1); }
.pager button:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
