<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入（按行）</div>
        <textarea v-model="state.input" class="textarea"></textarea>
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="linesUnique">去重</button>
          <button type="button" class="btn btn-secondary" @click="linesSort">排序</button>
          <button type="button" class="btn btn-secondary" @click="linesTrim">Trim</button>
          <button type="button" class="btn btn-secondary" @click="linesRemoveEmpty">去空行</button>
          <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制输出</button>
        </div>
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
const copyText = useCopyText();

const state = reactive({ input: '', output: '' });

function reset() {
  state.input = '';
  state.output = '';
}

watch(() => props.resetSeq, reset);

function getLines(text: string) {
  return text.replaceAll('\r\n', '\n').split('\n');
}

function linesUnique() {
  state.output = Array.from(new Set(getLines(state.input))).join('\n');
}

function linesSort() {
  const sorted = getLines(state.input)
    .slice()
    .sort((a, b) => a.localeCompare(b));
  state.output = sorted.join('\n');
}

function linesTrim() {
  state.output = getLines(state.input)
    .map(l => l.trim())
    .join('\n');
}

function linesRemoveEmpty() {
  state.output = getLines(state.input)
    .filter(l => l.trim().length > 0)
    .join('\n');
}
</script>
