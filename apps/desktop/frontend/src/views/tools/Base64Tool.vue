<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入</div>
        <textarea v-model="state.input" class="textarea"></textarea>
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="base64Encode">Base64 编码</button>
          <button type="button" class="btn btn-secondary" @click="base64Decode">Base64 解码</button>
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
import { decodeBase64Utf8, encodeBase64Utf8 } from './shared/codecs';
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

function base64Encode() {
  state.error = '';
  state.output = '';
  try {
    state.output = encodeBase64Utf8(state.input);
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'Base64 编码失败';
  }
}

function base64Decode() {
  state.error = '';
  state.output = '';
  try {
    state.output = decodeBase64Utf8(state.input.trim());
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'Base64 解码失败';
  }
}
</script>
