<template>
  <div class="tool-body">
    <div class="diff-grid">
      <div class="tool-col">
        <div class="field-label">左侧</div>
        <textarea v-model="state.left" class="textarea" placeholder="左侧文本"></textarea>
      </div>
      <div class="tool-col">
        <div class="field-label">右侧</div>
        <textarea v-model="state.right" class="textarea" placeholder="右侧文本"></textarea>
      </div>
    </div>

    <div class="btn-row">
      <button type="button" class="btn btn-primary" @click="computeDiff">生成 Diff</button>
      <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制输出</button>
    </div>

    <div v-if="state.error" class="error">{{ state.error }}</div>
    <div class="field-label">输出（简易按行 Diff）</div>
    <textarea v-model="state.output" class="textarea textarea-output" readonly></textarea>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import { useCopyText } from './shared/useCopyText';

const props = defineProps<{ resetSeq: number }>();
const copyText = useCopyText();

const state = reactive({ left: '', right: '', output: '', error: '' });

function reset() {
  state.left = '';
  state.right = '';
  state.output = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

function normalizeLines(text: string) {
  return text.replaceAll('\r\n', '\n').split('\n');
}

function computeDiff() {
  state.error = '';
  state.output = '';
  try {
    const leftLines = normalizeLines(state.left);
    const rightLines = normalizeLines(state.right);
    const maxLen = Math.max(leftLines.length, rightLines.length);
    const out: string[] = [];

    for (let i = 0; i < maxLen; i++) {
      const l = leftLines[i];
      const r = rightLines[i];
      if (l === r) {
        out.push(` ${l ?? ''}`);
        continue;
      }
      if (l !== undefined) {
        out.push(`-${l}`);
      }
      if (r !== undefined) {
        out.push(`+${r}`);
      }
    }

    state.output = out.join('\n');
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'Diff 生成失败';
  }
}
</script>
