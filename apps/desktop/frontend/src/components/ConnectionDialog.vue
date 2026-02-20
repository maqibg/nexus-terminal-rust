<template>
  <div class="overlay" @click.self="$emit('close')">
    <div class="dialog">
      <h3>{{ editing ? '编辑连接' : '新建连接' }}</h3>
      <div v-if="error" class="error">{{ error }}</div>
      <div class="form">
        <label>名称 <input v-model="form.name" placeholder="My Server" /></label>
        <div class="row">
          <label class="flex">主机 <input v-model="form.host" placeholder="192.168.1.1" /></label>
          <label class="port">端口 <input v-model.number="form.port" type="number" /></label>
        </div>
        <label>用户名 <input v-model="form.username" placeholder="root" /></label>
        <label>认证方式
          <AppSelect
            v-model="form.auth_method"
            :options="authMethodOptions"
            class="dialog-select"
            aria-label="认证方式"
          />
        </label>
        <label v-if="form.auth_method === 'password'">密码 <input v-model="form.password" type="password" /></label>
        <label v-if="form.auth_method === 'key'">SSH 密钥
          <AppSelect
            v-model="form.ssh_key_id"
            :options="sshKeyOptions"
            class="dialog-select"
            aria-label="SSH 密钥"
          />
        </label>
        <label>代理
          <AppSelect
            v-model="form.proxy_id"
            :options="proxyOptions"
            class="dialog-select"
            aria-label="代理"
          />
        </label>
      </div>
      <div class="actions">
        <button class="btn-cancel" @click="$emit('close')">取消</button>
        <button class="btn-save" @click="handleSave" :disabled="busy">{{ editing ? '保存' : '创建' }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, reactive, onMounted } from 'vue';
import AppSelect from './AppSelect.vue';
import { connectionsApi } from '@/lib/api';
import type { SshKey, Proxy } from '@/lib/api';

const props = defineProps<{ connection?: { id: number; name: string; host: string; port: number; username: string; auth_method: string; ssh_key_id?: number; proxy_id?: number } }>();
const emit = defineEmits<{ close: []; saved: [] }>();

const editing = !!props.connection;
const error = ref('');
const busy = ref(false);
const sshKeys = ref<SshKey[]>([]);
const proxies = ref<Proxy[]>([]);
const authMethodOptions = [
  { value: 'password', label: '密码' },
  { value: 'key', label: 'SSH 密钥' },
];

const sshKeyOptions = computed(() => [
  { value: undefined, label: '无' },
  ...sshKeys.value.map((key) => ({ value: key.id, label: key.name })),
]);

const proxyOptions = computed(() => [
  { value: undefined, label: '直连（无代理）' },
  ...proxies.value.map((proxy) => ({ value: proxy.id, label: `${proxy.name} (${proxy.host}:${proxy.port})` })),
]);

const form = reactive({
  name: props.connection?.name ?? '',
  host: props.connection?.host ?? '',
  port: props.connection?.port ?? 22,
  username: props.connection?.username ?? 'root',
  auth_method: props.connection?.auth_method ?? 'password',
  password: '',
  ssh_key_id: props.connection?.ssh_key_id as number | undefined,
  proxy_id: props.connection?.proxy_id as number | undefined,
});

onMounted(async () => {
  try { sshKeys.value = await connectionsApi.sshKeyList(); } catch { /* ignore */ }
  try { proxies.value = await connectionsApi.proxyList(); } catch { /* ignore */ }
});

async function handleSave() {
  if (!form.name || !form.host || !form.username) { error.value = '请填写必填字段'; return; }
  busy.value = true;
  error.value = '';
  try {
    const data: Record<string, unknown> = { ...form };
    if (form.auth_method !== 'key') delete data.ssh_key_id;
    if (editing) {
      await connectionsApi.update(props.connection!.id, data);
    } else {
      await connectionsApi.create(data);
    }
    emit('saved');
    emit('close');
  } catch (e: any) {
    error.value = e.message;
  } finally {
    busy.value = false;
  }
}
</script>

<style scoped>
.overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.5);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.dialog {
  background: var(--bg-surface0);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 1.5rem;
  min-width: 400px; max-width: 480px; box-shadow: 0 8px 32px rgba(0,0,0,0.4);
}
h3 { margin: 0 0 1rem; color: var(--text); font-weight: 500; }
.form { display: flex; flex-direction: column; gap: 0.6rem; }
label { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: var(--text-sub); }
input {
  padding: 0.5rem 0.6rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  font-size: 0.85rem;
  outline: none;
}
input:focus {
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}
.dialog-select :deep(.app-select-trigger) {
  padding: 0.5rem 0.6rem;
  min-height: 0;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  font-size: 0.85rem;
}
.dialog-select :deep(.app-select-trigger:focus-visible) {
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}
.row { display: flex; gap: 0.5rem; }
.flex { flex: 1; }
.port { width: 80px; }
.actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem; }
.btn-cancel {
  padding: 0.5rem 1rem; border-radius: 6px; border: 1px solid var(--border);
  background: transparent; color: var(--text-sub); cursor: pointer; font-size: 0.85rem;
}
.btn-save {
  padding: 0.5rem 1rem; border-radius: 6px; border: none;
  background: var(--blue); color: var(--button-text-color); cursor: pointer; font-weight: 600; font-size: 0.85rem;
}
.btn-save:hover { filter: brightness(1.05); }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-cancel:hover { background: var(--bg-surface1); color: var(--text); }
.error { color: var(--red); font-size: 0.8rem; margin-bottom: 0.5rem; }
</style>
