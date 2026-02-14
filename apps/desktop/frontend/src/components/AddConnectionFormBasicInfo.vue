<template>
  <div class="form-section">
    <label class="field">
      <span class="label">名称</span>
      <input v-model="form.name" class="input" placeholder="My Server" />
    </label>

    <div class="row">
      <label class="field protocol-field">
        <span class="label">协议</span>
        <select v-model="form.type" class="input">
          <option value="SSH">SSH</option>
          <option value="RDP">RDP</option>
        </select>
      </label>
      <label class="field flex-1">
        <span class="label">主机</span>
        <input v-model="form.host" class="input" placeholder="192.168.1.1" />
      </label>
    </div>

    <div class="row">
      <label class="field flex-1">
        <span class="label">端口</span>
        <input v-model.number="form.port" type="number" class="input" />
      </label>
      <label class="field flex-2">
        <span class="label">用户名</span>
        <input
          v-model="form.username"
          class="input"
          :placeholder="form.type === 'RDP' ? 'Administrator' : 'root'"
        />
      </label>
    </div>

    <label class="field">
      <span class="label">标签</span>
      <TagInput v-model="form.tags" :available-tags="availableTags.map(t => t.name)" />
    </label>

    <label class="field">
      <span class="label">代理</span>
      <select v-model="form.proxy_id" class="input">
        <option :value="undefined">无</option>
        <option v-for="p in proxies" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </label>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useProxiesStore } from '@/stores/proxies';
import { useTagsStore } from '@/stores/tags';
import { storeToRefs } from 'pinia';
import TagInput from './TagInput.vue';
import type { ConnectionFormData } from '@/composables/useAddConnectionForm';

defineProps<{ form: ConnectionFormData }>();

const proxiesStore = useProxiesStore();
const { items: proxies } = storeToRefs(proxiesStore);
const tagsStore = useTagsStore();
const { items: availableTags } = storeToRefs(tagsStore);
onMounted(() => {
  proxiesStore.fetchAll();
  tagsStore.fetchAll();
});
</script>

<style scoped>
.form-section { display: flex; flex-direction: column; gap: 12px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.label { font-size: 12px; color: var(--text-sub); }
.input { padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: 13px; }
.input:focus { outline: none; border-color: var(--blue); }
.row { display: flex; gap: 12px; }
.protocol-field { width: 120px; }
.flex-1 { flex: 1; }
.flex-2 { flex: 2; }
</style>
