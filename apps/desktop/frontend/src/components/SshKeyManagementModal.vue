<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="$emit('close')">
      <div class="modal-card">
        <div class="modal-header">
          <span>SSH 密钥管理</span>
          <span class="close-btn" @click="$emit('close')">×</span>
        </div>
        <div class="key-list">
          <div v-for="k in keys" :key="k.id" class="key-item">
            <span class="key-name">{{ k.name }}</span>
            <button class="btn-del" @click="remove(k.id)">删除</button>
          </div>
          <div v-if="!keys.length" class="empty">暂无密钥</div>
        </div>
        <div class="divider"></div>
        <div class="add-form">
          <input v-model="form.name" class="input" placeholder="密钥名称" />
          <textarea v-model="form.privateKey" class="input textarea" placeholder="粘贴私钥内容..." rows="4"></textarea>
          <input v-model="form.passphrase" class="input" type="password" placeholder="密码短语（可选）" />
          <div class="form-row">
            <label class="upload-label">
              <input type="file" hidden @change="onFileSelect" />
              或上传私钥文件
            </label>
            <button class="btn btn-primary" @click="create" :disabled="!form.name || !form.privateKey">添加</button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import { connectionsApi } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

defineProps<{ visible: boolean }>();
defineEmits<{ close: [] }>();

const notify = useUINotificationStore();
const keys = ref<{ id: number; name: string }[]>([]);
const form = reactive({ name: '', privateKey: '', passphrase: '' });

async function load() {
  keys.value = await connectionsApi.sshKeyList();
}
onMounted(load);

async function create() {
  try {
    await connectionsApi.sshKeyCreate(form.name, form.privateKey, form.passphrase || undefined);
    form.name = ''; form.privateKey = ''; form.passphrase = '';
    await load();
    notify.addNotification('success', '密钥已添加');
  } catch (e: any) { notify.addNotification('error', e.message); }
}

async function remove(id: number) {
  await connectionsApi.sshKeyDelete(id);
  await load();
}

function onFileSelect(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => { form.privateKey = reader.result as string; };
  reader.readAsText(file);
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.modal-card { background: var(--bg-surface0); border-radius: 8px; padding: 20px; width: 480px; max-height: 80vh; overflow-y: auto; border: 1px solid var(--border); }
.modal-header { display: flex; justify-content: space-between; align-items: center; font-size: calc(16px + var(--ui-font-size-offset)); font-weight: 600; margin-bottom: 16px; }
.close-btn { cursor: pointer; font-size: calc(20px + var(--ui-font-size-offset)); color: var(--text-dim); }
.close-btn:hover { color: var(--red); }
.key-list { margin-bottom: 12px; }
.key-item { display: flex; justify-content: space-between; align-items: center; padding: 8px; border-radius: 4px; background: var(--bg-mantle); margin-bottom: 4px; }
.key-name { font-size: calc(13px + var(--ui-font-size-offset)); }
.btn-del { background: none; border: none; color: var(--red); cursor: pointer; font-size: calc(12px + var(--ui-font-size-offset)); }
.empty { color: var(--text-dim); font-size: calc(13px + var(--ui-font-size-offset)); text-align: center; padding: 12px; }
.divider { height: 1px; background: var(--border); margin: 12px 0; }
.add-form { display: flex; flex-direction: column; gap: 8px; }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 8px; color: var(--text); font-size: calc(13px + var(--ui-font-size-offset)); outline: none; }
.input:focus { border-color: var(--blue); }
.textarea { resize: vertical; font-family: monospace; font-size: calc(12px + var(--ui-font-size-offset)); }
.form-row { display: flex; justify-content: space-between; align-items: center; }
.upload-label { font-size: calc(12px + var(--ui-font-size-offset)); color: var(--blue); cursor: pointer; }
.upload-label:hover { text-decoration: underline; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: calc(13px + var(--ui-font-size-offset)); }
.btn-primary { background: var(--blue); color: var(--bg-base); }
.btn:disabled { opacity: 0.4; cursor: default; }
</style>
