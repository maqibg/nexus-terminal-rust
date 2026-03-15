<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入</div>
        <textarea v-model="state.input" class="textarea" placeholder="<div>&quot; &amp;</div>"></textarea>
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="htmlEntitiesEncode">HTML 转义</button>
          <button type="button" class="btn btn-secondary" @click="htmlEntitiesDecode">HTML 反转义</button>
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
import { decodeHtmlEntities, encodeHtmlEntities } from './shared/codecs';
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

function htmlEntitiesEncode() {
  state.error = '';
  state.output = encodeHtmlEntities(state.input);
}

function htmlEntitiesDecode() {
  state.error = '';
  state.output = '';
  try {
    state.output = decodeHtmlEntities(state.input);
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'HTML 反转义失败';
  }
}
</script>
