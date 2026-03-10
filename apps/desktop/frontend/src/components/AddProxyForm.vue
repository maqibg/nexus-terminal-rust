<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="$emit('cancel')">
      <div class="form-card">
        <div class="form-title">{{ proxy ? '编辑代理' : '新建代理' }}</div>
        <div class="form-body">
          <input v-model="form.name" class="input" placeholder="名称" />
          <AppSelect
            v-model="form.proxy_type"
            :options="proxyTypeOptions"
            variant="input"
            aria-label="代理类型"
          />
          <input v-model="form.host" class="input" placeholder="主机" />
          <input v-model.number="form.port" class="input" type="number" placeholder="端口" />
          <input v-model="form.username" class="input" placeholder="用户名（可选）" />
          <input v-model="form.password" class="input" type="password" placeholder="密码（可选）" />
        </div>
        <div class="form-actions">
          <button class="btn btn-cancel" @click="$emit('cancel')">取消</button>
          <button class="btn btn-primary" @click="submit" :disabled="!form.name || !form.host">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import AppSelect from './AppSelect.vue';
import { connectionsApi, type Proxy } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const props = defineProps<{ visible: boolean; proxy?: Proxy }>();
const emit = defineEmits<{ saved: []; cancel: [] }>();
const notify = useUINotificationStore();
const proxyTypeOptions = [
  { value: 'SOCKS5', label: 'SOCKS5' },
  { value: 'HTTP', label: 'HTTP' },
];

const form = reactive({
  name: '', proxy_type: 'SOCKS5', host: '', port: 1080,
  username: '', password: '',
});

watch(() => props.proxy, (p) => {
  if (p) Object.assign(form, { name: p.name, proxy_type: p.proxy_type, host: p.host, port: p.port, username: '', password: '' });
  else Object.assign(form, { name: '', proxy_type: 'SOCKS5', host: '', port: 1080, username: '', password: '' });
}, { immediate: true });

async function submit() {
  try {
    const data = {
      id: props.proxy?.id ?? 0, name: form.name, type: form.proxy_type,
      host: form.host, port: form.port, username: form.username || null,
      auth_method: form.username ? 'password' : 'none',
      encrypted_password: form.password || null, encrypted_private_key: null,
    };
    if (props.proxy) await connectionsApi.proxyUpdate(data);
    else await connectionsApi.proxyCreate(data);
    notify.addNotification('success', props.proxy ? '代理已更新' : '代理已创建');
    emit('saved');
  } catch (e: any) { notify.addNotification('error', e.message); }
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.form-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; width: 400px; border: 1px solid var(--border); }
.form-title { font-size: calc(16px + var(--ui-font-size-offset)); font-weight: 600; margin-bottom: 16px; }
.form-body { display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 8px; color: var(--text); font-size: calc(13px + var(--ui-font-size-offset)); outline: none; }
.input:focus { border-color: var(--blue); box-shadow: 0 0 0 1px var(--blue); }
.form-actions { display: flex; justify-content: flex-end; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: calc(13px + var(--ui-font-size-offset)); }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn-primary { background: var(--blue); color: var(--bg-base); }
.btn:disabled { opacity: 0.4; cursor: default; }
</style>
