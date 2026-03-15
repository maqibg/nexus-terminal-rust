<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入</div>
        <textarea v-model="state.input" class="textarea" placeholder="https://example.com?q=中文"></textarea>
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="urlEncode">URL 编码</button>
          <button type="button" class="btn btn-secondary" @click="urlDecode">URL 解码</button>
          <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制输出</button>
        </div>
        <div v-if="state.error" class="error">{{ state.error }}</div>
      </div>
      <div class="tool-col">
        <div class="field-label">输出</div>
        <textarea v-model="state.output" class="textarea" readonly></textarea>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import { useCopyText } from './shared/useCopyText';

const props = defineProps<{ resetSeq: number }>();

const state = reactive({ input: '', output: '', error: '' });
const copyText = useCopyText();

function reset() {
  state.input = '';
  state.output = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

function urlEncode() {
  state.error = '';
  state.output = '';
  try {
    state.output = encodeURIComponent(state.input);
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'URL 编码失败';
  }
}

function urlDecode() {
  state.error = '';
  state.output = '';
  try {
    state.output = decodeURIComponent(state.input);
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'URL 解码失败';
  }
}
</script>
