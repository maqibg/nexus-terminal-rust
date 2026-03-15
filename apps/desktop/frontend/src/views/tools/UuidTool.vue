<template>
  <div class="tool-body">
    <div class="inline-row">
      <label class="inline-label">数量</label>
      <input v-model.number="state.count" type="number" :min="UUID_MIN_COUNT" :max="UUID_MAX_COUNT" class="input input-count" />
      <button type="button" class="btn btn-primary" @click="generateUuids">生成</button>
      <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制输出</button>
    </div>
    <div v-if="state.error" class="error">{{ state.error }}</div>
    <textarea v-model="state.output" class="textarea" readonly></textarea>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import { v4 as uuidv4 } from 'uuid';
import { useCopyText } from './shared/useCopyText';

const UUID_MIN_COUNT = 1;
const UUID_MAX_COUNT = 100;
const DEFAULT_UUID_COUNT = 5;

const props = defineProps<{ resetSeq: number }>();
const copyText = useCopyText();

const state = reactive({ count: DEFAULT_UUID_COUNT, output: '', error: '' });

function reset() {
  state.count = DEFAULT_UUID_COUNT;
  state.output = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

function generateUuid() {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return uuidv4();
}

function generateUuids() {
  state.error = '';
  state.output = '';

  const count = Number.isFinite(state.count) ? Math.floor(state.count) : 0;
  if (count < UUID_MIN_COUNT || count > UUID_MAX_COUNT) {
    state.error = `数量需在 ${UUID_MIN_COUNT}~${UUID_MAX_COUNT} 之间`;
    return;
  }

  const items: string[] = [];
  for (let i = 0; i < count; i++) {
    items.push(generateUuid());
  }
  state.output = items.join('\n');
}
</script>
