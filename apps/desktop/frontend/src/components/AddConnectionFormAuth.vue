<script setup lang="ts">
import SshKeySelector from './SshKeySelector.vue';

defineProps<{
  formData: {
    type: 'SSH' | 'RDP' | 'VNC';
    username: string;
    auth_method: 'password' | 'key';
    password: string;
    selected_ssh_key_id: number | null;
    vncPassword: string;
  };
  isEditMode: boolean;
}>();
</script>

<template>
  <div class="section-card">
    <h4 class="section-title">认证信息</h4>

    <div class="field-block">
      <label for="conn-username" class="field-label">用户名</label>
      <input id="conn-username" v-model="formData.username" type="text" class="field-input" :required="formData.type === 'SSH'" />
    </div>

    <template v-if="formData.type === 'SSH'">
      <div class="field-block">
        <label class="field-label">认证方式</label>
        <div class="segment-group">
          <button
            type="button"
            class="segment-btn"
            :class="{ active: formData.auth_method === 'password' }"
            @click="formData.auth_method = 'password'"
          >
            密码
          </button>
          <button
            type="button"
            class="segment-btn"
            :class="{ active: formData.auth_method === 'key' }"
            @click="formData.auth_method = 'key'"
          >
            SSH 密钥
          </button>
        </div>
      </div>

      <div v-if="formData.auth_method === 'password'" class="field-block">
        <label for="conn-password" class="field-label">密码</label>
        <input
          id="conn-password"
          v-model="formData.password"
          type="password"
          class="field-input"
          :required="!isEditMode"
          autocomplete="new-password"
        />
      </div>

      <div v-if="formData.auth_method === 'key'" class="field-block">
        <label class="field-label">SSH 密钥</label>
        <SshKeySelector v-model="formData.selected_ssh_key_id" />
      </div>
    </template>

    <template v-if="formData.type === 'RDP'">
      <div class="field-block">
        <label for="conn-password-rdp" class="field-label">密码</label>
        <input
          id="conn-password-rdp"
          v-model="formData.password"
          type="password"
          class="field-input"
          :required="false"
          autocomplete="new-password"
        />
      </div>
    </template>

    <template v-if="formData.type === 'VNC'">
      <div class="field-block">
        <label for="conn-password-vnc" class="field-label">VNC 密码</label>
        <input
          id="conn-password-vnc"
          v-model="formData.vncPassword"
          type="password"
          class="field-input"
          :required="!isEditMode"
          autocomplete="new-password"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.section-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-surface1) 45%, transparent);
}

.section-title {
  margin: 0;
  padding-bottom: 8px;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-sub);
}

.field-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  font-size: 13px;
}

.field-input:focus {
  outline: none;
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}

.segment-group {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}

.segment-btn {
  height: 34px;
  border: none;
  border-right: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  font-size: 13px;
  cursor: pointer;
}

.segment-btn:last-child {
  border-right: none;
}

.segment-btn.active {
  background: var(--blue);
  color: #ffffff;
}
</style>
