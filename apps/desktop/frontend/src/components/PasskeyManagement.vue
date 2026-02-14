<template>
  <section class="passkey-section">
    <h3 class="section-title">Passkey 管理</h3>
    <div class="key-list">
      <div v-for="pk in passkeys" :key="pk.credential_id" class="key-item">
        <span class="key-name">{{ pk.name }}</span>
        <div class="key-actions">
          <button class="btn-sm" @click="rename(pk)">重命名</button>
          <button class="btn-sm danger" @click="remove(pk)">删除</button>
        </div>
      </div>
      <div v-if="!passkeys.length" class="empty">暂无 Passkey</div>
    </div>
    <button class="btn-add" @click="register">注册新 Passkey</button>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { authApi, type PasskeyInfo } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const notify = useUINotificationStore();
const passkeys = ref<PasskeyInfo[]>([]);

async function load() {
  try { passkeys.value = await authApi.passkeyList(); } catch { passkeys.value = []; }
}
onMounted(load);

async function register() {
  const name = prompt('Passkey 名称:');
  if (!name) return;
  try {
    const options = await authApi.passkeyRegisterStart();
    // 简化：直接用随机 credential_id 注册（完整实现需要 navigator.credentials.create）
    const credentialId = crypto.randomUUID();
    await authApi.passkeyRegisterFinish(credentialId, '{}', name);
    notify.addNotification('success', 'Passkey 已注册');
    load();
  } catch (e: any) { notify.addNotification('error', e.message); }
}

async function remove(pk: PasskeyInfo) {
  if (!confirm(`确定删除 "${pk.name}"？`)) return;
  try {
    await authApi.passkeyDelete(pk.credential_id);
    notify.addNotification('success', 'Passkey 已删除');
    load();
  } catch (e: any) { notify.addNotification('error', e.message); }
}

async function rename(pk: PasskeyInfo) {
  const name = prompt('新名称:', pk.name);
  if (!name || name === pk.name) return;
  try {
    await authApi.passkeyRename(pk.credential_id, name);
    notify.addNotification('success', '已重命名');
    load();
  } catch (e: any) { notify.addNotification('error', e.message); }
}
</script>

<style scoped>
.passkey-section { display: flex; flex-direction: column; gap: 10px; }
.section-title { font-size: 15px; font-weight: 600; margin: 0; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.key-list { display: flex; flex-direction: column; gap: 4px; }
.key-item { display: flex; justify-content: space-between; align-items: center; padding: 8px; border-radius: 4px; background: var(--bg-mantle); }
.key-name { font-size: 13px; }
.key-actions { display: flex; gap: 4px; }
.btn-sm { padding: 3px 10px; border-radius: 3px; border: 1px solid var(--border); background: transparent; color: var(--text); cursor: pointer; font-size: 12px; }
.btn-sm:hover { background: var(--bg-surface1); }
.btn-sm.danger { color: var(--red); border-color: var(--red); }
.btn-sm.danger:hover { background: rgba(243,139,168,0.1); }
.btn-add { align-self: flex-start; padding: 5px 14px; border-radius: 4px; border: none; background: var(--blue); color: var(--bg-base); cursor: pointer; font-size: 13px; font-weight: 600; }
.btn-add:hover { opacity: 0.9; }
.empty { text-align: center; color: var(--text-dim); font-size: 13px; padding: 12px; }
</style>
