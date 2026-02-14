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
          <select v-model="form.auth_method">
            <option value="password">密码</option>
            <option value="key">SSH 密钥</option>
          </select>
        </label>
        <label v-if="form.auth_method === 'password'">密码 <input v-model="form.password" type="password" /></label>
        <label v-if="form.auth_method === 'key'">SSH 密钥
          <select v-model="form.ssh_key_id">
            <option :value="undefined">无</option>
            <option v-for="k in sshKeys" :key="k.id" :value="k.id">{{ k.name }}</option>
          </select>
        </label>
        <label>代理
          <select v-model="form.proxy_id">
            <option :value="undefined">直连（无代理）</option>
            <option v-for="p in proxies" :key="p.id" :value="p.id">{{ p.name }} ({{ p.host }}:{{ p.port }})</option>
          </select>
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
import { ref, reactive, onMounted } from 'vue';
import { connectionsApi } from '@/lib/api';
import type { SshKey, Proxy } from '@/lib/api';

const props = defineProps<{ connection?: { id: number; name: string; host: string; port: number; username: string; auth_method: string; ssh_key_id?: number; proxy_id?: number } }>();
const emit = defineEmits<{ close: []; saved: [] }>();

const editing = !!props.connection;
const error = ref('');
const busy = ref(false);
const sshKeys = ref<SshKey[]>([]);
const proxies = ref<Proxy[]>([]);

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
  background: #313244; border-radius: 12px; padding: 1.5rem;
  min-width: 400px; max-width: 480px; box-shadow: 0 8px 32px rgba(0,0,0,0.4);
}
h3 { margin: 0 0 1rem; color: #cdd6f4; font-weight: 500; }
.form { display: flex; flex-direction: column; gap: 0.6rem; }
label { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: #a6adc8; }
input, select {
  padding: 0.5rem 0.6rem; border-radius: 6px; border: 1px solid #45475a;
  background: #1e1e2e; color: #cdd6f4; font-size: 0.85rem; outline: none;
}
input:focus, select:focus { border-color: #89b4fa; }
.row { display: flex; gap: 0.5rem; }
.flex { flex: 1; }
.port { width: 80px; }
.actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem; }
.btn-cancel {
  padding: 0.5rem 1rem; border-radius: 6px; border: 1px solid #45475a;
  background: transparent; color: #a6adc8; cursor: pointer; font-size: 0.85rem;
}
.btn-save {
  padding: 0.5rem 1rem; border-radius: 6px; border: none;
  background: #89b4fa; color: #1e1e2e; cursor: pointer; font-weight: 600; font-size: 0.85rem;
}
.btn-save:hover { background: #74c7ec; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-cancel:hover { background: #45475a; }
.error { color: #f38ba8; font-size: 0.8rem; margin-bottom: 0.5rem; }
</style>
