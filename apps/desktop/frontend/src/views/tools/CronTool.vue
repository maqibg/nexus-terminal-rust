<template>
  <div class="tool-body">
    <div class="inline-row">
      <label class="inline-label">表达式</label>
      <input v-model="state.expression" class="input input-wide" placeholder="*/5 * * * *" />
      <label class="inline-label">条数</label>
      <input v-model.number="state.count" type="number" :min="CRON_MIN_COUNT" :max="CRON_MAX_COUNT" class="input input-count" />
      <button type="button" class="btn btn-primary" @click="previewCron">预览</button>
      <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制输出</button>
    </div>
    <div class="hint">支持 5 段 cron（分 时 日 月 周），以本机时区计算。</div>
    <div v-if="state.error" class="error">{{ state.error }}</div>
    <textarea v-model="state.output" class="textarea" readonly></textarea>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import { previewCronDates } from './shared/cron';
import { useCopyText } from './shared/useCopyText';

const CRON_MIN_COUNT = 1;
const CRON_MAX_COUNT = 50;
const DEFAULT_CRON_COUNT = 10;

const props = defineProps<{ resetSeq: number }>();
const copyText = useCopyText();

const state = reactive({ expression: '', count: DEFAULT_CRON_COUNT, output: '', error: '' });

function reset() {
  state.expression = '';
  state.count = DEFAULT_CRON_COUNT;
  state.output = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

function previewCron() {
  state.error = '';
  state.output = '';

  const expression = state.expression.trim();
  if (!expression) {
    state.error = '请输入 cron 表达式';
    return;
  }

  const count = Number.isFinite(state.count) ? Math.floor(state.count) : 0;
  if (count < CRON_MIN_COUNT || count > CRON_MAX_COUNT) {
    state.error = `条数需在 ${CRON_MIN_COUNT}~${CRON_MAX_COUNT} 之间`;
    return;
  }

  try {
    const dates = previewCronDates({ expression, count });
    if (dates.length === 0) {
      state.error = '未在合理范围内找到下一次触发时间，请检查表达式';
      return;
    }
    state.output = dates.map(d => d.toLocaleString()).join('\n');
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'Cron 解析失败';
  }
}
</script>
