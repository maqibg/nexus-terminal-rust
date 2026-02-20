<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="$emit('cancel')">
      <div class="form-card">
        <div class="form-title">批量编辑连接 ({{ connectionIds.length }} 个)</div>
        <div class="form-body">
          <p class="hint">仅修改非空字段，空字段将保持原值不变。</p>

          <label class="field-label">端口</label>
          <input class="input" type="number" v-model.number="form.port" placeholder="留空不修改" />

          <label class="field-label">SSH 密钥</label>
          <AppSelect v-model="form.ssh_key_id" :options="sshKeyOptions" variant="input" aria-label="SSH 密钥" />

          <label class="field-label">代理</label>
          <AppSelect v-model="form.proxy_id" :options="proxyOptions" variant="input" aria-label="代理" />

          <label class="field-label">标签 (逗号分隔)</label>
          <input class="input" v-model="form.tags" placeholder="留空不修改" />
        </div>
        <div class="form-actions">
          <button class="btn btn-cancel" @click="$emit('cancel')">取消</button>
          <button class="btn btn-primary" @click="submit" :disabled="saving">{{ saving ? '保存中...' : '保存' }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, reactive, onMounted } from 'vue';
import AppSelect from './AppSelect.vue';
import { connectionsApi, type SshKey, type Proxy } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const props = defineProps<{ visible: boolean; connectionIds: number[] }>();
const emit = defineEmits<{ saved: []; cancel: [] }>();
const notify = useUINotificationStore();

const sshKeys = ref<SshKey[]>([]);
const proxies = ref<Proxy[]>([]);
const saving = ref(false);
const form = reactive<{ port?: number; ssh_key_id?: number; proxy_id?: number; tags: string }>({
  port: undefined, ssh_key_id: undefined, proxy_id: undefined, tags: '',
});

const sshKeyOptions = computed(() => [
  { value: undefined, label: '不修改' },
  { value: 0, label: '无' },
  ...sshKeys.value.map((key) => ({ value: key.id, label: key.name })),
]);

const proxyOptions = computed(() => [
  { value: undefined, label: '不修改' },
  { value: 0, label: '无' },
  ...proxies.value.map((proxy) => ({ value: proxy.id, label: proxy.name })),
]);

onMounted(async () => {
  const [keys, pxs] = await Promise.all([connectionsApi.sshKeyList(), connectionsApi.proxyList()]);
  sshKeys.value = keys;
  proxies.value = pxs;
});

async function submit() {
  saving.value = true;
  try {
    for (const id of props.connectionIds) {
      const conn = await connectionsApi.get(id);
      const update: Record<string, unknown> = {
        name: conn.name, host: conn.host, port: form.port ?? conn.port,
        username: conn.username, auth_method: conn.auth_method,
      };
      if (form.ssh_key_id !== undefined) update.ssh_key_id = form.ssh_key_id || null;
      if (form.proxy_id !== undefined) update.proxy_id = form.proxy_id || null;
      if (form.tags.trim()) update.tags = form.tags.split(',').map(t => t.trim()).filter(Boolean);
      await connectionsApi.update(id, update);
    }
    notify.addNotification('success', `已更新 ${props.connectionIds.length} 个连接`);
    emit('saved');
  } catch (e: any) { notify.addNotification('error', e.message); }
  saving.value = false;
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.form-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; width: 420px; border: 1px solid var(--border); }
.form-title { font-size: 16px; font-weight: 600; margin-bottom: 16px; }
.form-body { display: flex; flex-direction: column; gap: 6px; margin-bottom: 16px; }
.hint { font-size: 12px; color: var(--text-dim); margin: 0 0 4px; }
.field-label { font-size: 12px; color: var(--text-sub); margin-top: 4px; }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 8px; color: var(--text); font-size: 13px; outline: none; }
.input:focus { border-color: var(--blue); box-shadow: 0 0 0 1px var(--blue); }
.form-actions { display: flex; justify-content: flex-end; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: 13px; }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn-primary { background: var(--blue); color: var(--bg-base); font-weight: 600; }
.btn:disabled { opacity: 0.4; cursor: default; }
</style>
