<template>
  <DbConnectionDialog
    :visible="visible"
    :title="title"
    :save-label="saveLabel"
    :busy="busy"
    :error="error"
    @close="emit('close')"
    @save="emit('save')"
  >
    <div class="field-block">
      <label class="field-label">名称（可选）</label>
      <input :value="name" class="field-input" type="text" placeholder="例如：prod-redis" @input="emit('update:name', ($event.target as HTMLInputElement).value)" />
    </div>

    <div class="field-grid">
      <div class="field-block">
        <label class="field-label">Host</label>
        <input :value="host" class="field-input" type="text" placeholder="127.0.0.1" @input="emit('update:host', ($event.target as HTMLInputElement).value)" />
      </div>
      <div class="field-block">
        <label class="field-label">Port</label>
        <input :value="port" class="field-input" type="number" min="1" max="65535" @input="emit('update:port', Number(($event.target as HTMLInputElement).value) || 6379)" />
      </div>
    </div>

    <div class="field-grid">
      <div class="field-block">
        <label class="field-label">用户名（可选）</label>
        <input :value="username" class="field-input" type="text" placeholder="可留空" @input="emit('update:username', ($event.target as HTMLInputElement).value)" />
      </div>
      <div class="field-block">
        <label class="field-label">密码（保存到本地）</label>
        <input
          :value="password"
          class="field-input"
          type="password"
          placeholder="新建可留空；编辑留空保持"
          @input="emit('update:password', ($event.target as HTMLInputElement).value)"
        />
      </div>
    </div>

    <div v-if="mode === 'edit'" class="field-row">
      <label class="checkbox">
        <input :checked="clearSavedPassword" type="checkbox" @change="emit('update:clearSavedPassword', ($event.target as HTMLInputElement).checked)" />
        <span>清除已保存密码</span>
      </label>
    </div>

    <div class="field-block">
      <label class="field-label">默认 DB（可选）</label>
      <input :value="db" class="field-input" type="number" min="0" placeholder="默认 0" @input="emit('update:db', Number(($event.target as HTMLInputElement).value) || 0)" />
    </div>
  </DbConnectionDialog>
</template>

<script setup lang="ts">
import DbConnectionDialog from './DbConnectionDialog.vue';

defineProps<{
  visible: boolean;
  title: string;
  saveLabel: string;
  mode: 'create' | 'edit';
  busy: boolean;
  error: string;
  name: string;
  host: string;
  port: number;
  username: string;
  password: string;
  db: number | null;
  clearSavedPassword: boolean;
}>();

const emit = defineEmits<{
  close: [];
  save: [];
  'update:name': [value: string];
  'update:host': [value: string];
  'update:port': [value: number];
  'update:username': [value: string];
  'update:password': [value: string];
  'update:db': [value: number];
  'update:clearSavedPassword': [value: boolean];
}>();
</script>
