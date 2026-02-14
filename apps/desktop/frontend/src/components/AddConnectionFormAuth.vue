<template>
  <div class="form-section">
    <template v-if="form.type === 'SSH'">
      <label class="field">
        <span class="label">认证方式</span>
        <select v-model="form.auth_method" class="input">
          <option value="password">密码</option>
          <option value="key">SSH 密钥</option>
          <option value="none">无</option>
        </select>
      </label>
      <label v-if="form.auth_method === 'password'" class="field">
        <span class="label">密码</span>
        <input v-model="form.password" type="password" class="input" />
      </label>
      <label v-if="form.auth_method === 'key'" class="field">
        <span class="label">SSH 密钥</span>
        <select v-model="form.ssh_key_id" class="input">
          <option :value="undefined">选择密钥...</option>
          <option v-for="k in sshKeys" :key="k.id" :value="k.id">{{ k.name }}</option>
        </select>
      </label>
    </template>
    <div v-else class="hint">RDP 协议不使用 SSH 认证参数。</div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useSshKeysStore } from '@/stores/sshKeys';
import { storeToRefs } from 'pinia';
import type { ConnectionFormData } from '@/composables/useAddConnectionForm';

defineProps<{ form: ConnectionFormData }>();

const sshKeysStore = useSshKeysStore();
const { items: sshKeys } = storeToRefs(sshKeysStore);
onMounted(() => sshKeysStore.fetchAll());
</script>

<style scoped>
.form-section { display: flex; flex-direction: column; gap: 12px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.label { font-size: 12px; color: var(--text-sub); }
.input { padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: 13px; }
.input:focus { outline: none; border-color: var(--blue); }
.hint {
  padding: 10px 12px;
  border-radius: 6px;
  border: 1px dashed var(--border);
  color: var(--text-sub);
  font-size: 12px;
}
</style>
