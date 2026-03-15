<template>
  <div class="tool-body">
    <div class="inline-row">
      <label class="inline-label">Pattern</label>
      <input v-model="state.pattern" class="input input-wide" placeholder="\\b\\w+\\b" />
      <label class="inline-label">Flags</label>
      <input v-model="state.flags" class="input input-flags" placeholder="gim" />
    </div>

    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入文本</div>
        <textarea v-model="state.input" class="textarea"></textarea>
        <div class="inline-row">
          <label class="inline-label">Replace</label>
          <input v-model="state.replaceWith" class="input input-wide" placeholder="$1" />
        </div>
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="runRegexMatch">匹配</button>
          <button type="button" class="btn btn-secondary" @click="runRegexReplace">替换</button>
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
const copyText = useCopyText();

const state = reactive({
  pattern: '',
  flags: 'g',
  input: '',
  replaceWith: '',
  output: '',
  error: '',
});

function reset() {
  state.pattern = '';
  state.flags = 'g';
  state.input = '';
  state.replaceWith = '';
  state.output = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

function runRegexMatch() {
  state.error = '';
  state.output = '';
  try {
    const re = new RegExp(state.pattern, state.flags);
    const matches: Array<{ index: number; match: string; groups: string[] }> = [];

    if (state.flags.includes('g')) {
      let m: RegExpExecArray | null;
      while ((m = re.exec(state.input)) !== null) {
        matches.push({ index: m.index, match: m[0], groups: m.slice(1) });
        if (m[0] === '') {
          re.lastIndex++;
        }
      }
    } else {
      const m = re.exec(state.input);
      if (m) {
        matches.push({ index: m.index, match: m[0], groups: m.slice(1) });
      }
    }

    state.output = JSON.stringify({ count: matches.length, matches }, null, 2);
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'Regex 执行失败';
  }
}

function runRegexReplace() {
  state.error = '';
  state.output = '';
  try {
    const re = new RegExp(state.pattern, state.flags);
    state.output = state.flags.includes('g')
      ? state.input.replaceAll(re, state.replaceWith)
      : state.input.replace(re, state.replaceWith);
  } catch (err) {
    state.error = err instanceof Error ? err.message : '替换失败';
  }
}
</script>
