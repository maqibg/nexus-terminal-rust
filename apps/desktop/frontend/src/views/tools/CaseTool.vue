<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入</div>
        <textarea v-model="state.input" class="textarea" placeholder="hello_world example-text"></textarea>
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="toCamelCase">camelCase</button>
          <button type="button" class="btn btn-secondary" @click="toPascalCase">PascalCase</button>
          <button type="button" class="btn btn-secondary" @click="toSnakeCase">snake_case</button>
          <button type="button" class="btn btn-secondary" @click="toKebabCase">kebab-case</button>
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

function splitWords(text: string) {
  return text
    .trim()
    .replaceAll(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replaceAll(/[_\\-]+/g, ' ')
    .split(/\s+/)
    .filter(Boolean)
    .map(w => w.toLowerCase());
}

function toCamelCase() {
  const words = splitWords(state.input);
  if (words.length === 0) {
    state.output = '';
    return;
  }

  state.output =
    words[0] +
    words
      .slice(1)
      .map(w => w.slice(0, 1).toUpperCase() + w.slice(1))
      .join('');
}

function toPascalCase() {
  const words = splitWords(state.input);
  state.output = words.map(w => w.slice(0, 1).toUpperCase() + w.slice(1)).join('');
}

function toSnakeCase() {
  const words = splitWords(state.input);
  state.output = words.join('_');
}

function toKebabCase() {
  const words = splitWords(state.input);
  state.output = words.join('-');
}
</script>
