<template>
  <div class="form-section">
    <label class="field">
      <span class="label">编码</span>
      <select v-model="form.encoding" class="input">
        <option value="utf-8">UTF-8</option>
        <option value="gbk">GBK</option>
        <option value="gb2312">GB2312</option>
      </select>
    </label>
    <label class="field">
      <span class="label">备注</span>
      <textarea v-model="form.notes" class="input textarea" rows="3" />
    </label>
  </div>
</template>

<script setup lang="ts">
import type { ConnectionFormData } from '@/composables/useAddConnectionForm';
defineProps<{ form: ConnectionFormData }>();
</script>

<style scoped>
.form-section { display: flex; flex-direction: column; gap: 12px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.label { font-size: 12px; color: var(--text-sub); }
.input { padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: 13px; }
.input:focus { outline: none; border-color: var(--blue); }
.textarea { resize: vertical; font-family: inherit; }
</style>
