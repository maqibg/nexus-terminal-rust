<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入</div>
        <textarea v-model="state.input" class="textarea" placeholder="Hello 你好 👋"></textarea>
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="unicodeEscape">Unicode 转义</button>
          <button type="button" class="btn btn-secondary" @click="unicodeUnescape">Unicode 反转义</button>
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
import { escapeUnicode, unescapeUnicode } from './shared/codecs';
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

function unicodeEscape() {
  state.error = '';
  state.output = '';
  try {
    state.output = escapeUnicode(state.input);
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'Unicode 转义失败';
  }
}

function unicodeUnescape() {
  state.error = '';
  state.output = '';
  try {
    state.output = unescapeUnicode(state.input);
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'Unicode 反转义失败';
  }
}
</script>
