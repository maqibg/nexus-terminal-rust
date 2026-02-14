<template>
  <div class="ssh-key-selector">
    <select v-model="selected" class="input" @change="emit('update:modelValue', selected)">
      <option :value="undefined">选择密钥...</option>
      <option v-for="k in items" :key="k.id" :value="k.id">{{ k.name }}</option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useSshKeysStore } from '@/stores/sshKeys';
import { storeToRefs } from 'pinia';

const props = defineProps<{ modelValue?: number }>();
const emit = defineEmits<{ 'update:modelValue': [id: number | undefined] }>();

const store = useSshKeysStore();
const { items } = storeToRefs(store);
const selected = ref(props.modelValue);

watch(() => props.modelValue, (v) => { selected.value = v; });
onMounted(() => { if (!items.value.length) store.fetchAll(); });
</script>

<style scoped>
.input { width: 100%; padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: 13px; }
.input:focus { outline: none; border-color: var(--blue); }
</style>
