<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">Unix 时间戳</div>
        <input v-model="state.timestamp" class="input" placeholder="1710000000 或 1710000000000" />
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="timestampToDate">转日期</button>
          <button type="button" class="btn btn-secondary" @click="nowToTimestamp">当前时间</button>
        </div>
        <div v-if="state.error" class="error">{{ state.error }}</div>
      </div>

      <div class="tool-col">
        <div class="field-label">日期时间（本地）</div>
        <input v-model="state.dateText" class="input" placeholder="2026-03-13 20:00:00" />
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="dateToTimestamp">转时间戳</button>
          <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制输出</button>
        </div>
      </div>
    </div>

    <div class="field-label">输出</div>
    <textarea v-model="state.output" class="textarea" readonly></textarea>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import { useCopyText } from './shared/useCopyText';

const TIMESTAMP_DIGITS_THRESHOLD_SECONDS = 10;

const props = defineProps<{ resetSeq: number }>();
const copyText = useCopyText();

const state = reactive({ timestamp: '', dateText: '', output: '', error: '' });

function reset() {
  state.timestamp = '';
  state.dateText = '';
  state.output = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

function timestampToDate() {
  state.error = '';
  state.output = '';

  const raw = state.timestamp.trim();
  if (!raw) {
    state.error = '请输入时间戳';
    return;
  }
  const n = Number(raw);
  if (!Number.isFinite(n)) {
    state.error = '时间戳不是有效数字';
    return;
  }

  const ms = raw.length <= TIMESTAMP_DIGITS_THRESHOLD_SECONDS ? n * 1000 : n;
  const d = new Date(ms);
  if (Number.isNaN(d.getTime())) {
    state.error = '无法解析该时间戳';
    return;
  }

  state.output = d.toLocaleString();
}

function parseLocalDateTime(text: string) {
  const t = text.trim();
  if (!t) {
    return null;
  }

  const normalized = t.replace('T', ' ').replaceAll('/', '-');
  const m = normalized.match(/^(\d{4})-(\d{1,2})-(\d{1,2})(?:\s+(\d{1,2}):(\d{1,2})(?::(\d{1,2}))?)?$/);
  if (!m) {
    return null;
  }

  const year = Number(m[1]);
  const month = Number(m[2]) - 1;
  const day = Number(m[3]);
  const hour = Number(m[4] ?? '0');
  const minute = Number(m[5] ?? '0');
  const second = Number(m[6] ?? '0');
  const d = new Date(year, month, day, hour, minute, second);
  return Number.isNaN(d.getTime()) ? null : d;
}

function dateToTimestamp() {
  state.error = '';
  state.output = '';

  const d = parseLocalDateTime(state.dateText);
  if (!d) {
    state.error = '日期格式不正确（示例：2026-03-13 20:00:00）';
    return;
  }

  state.output = `${Math.floor(d.getTime() / 1000)} (s)\n${d.getTime()} (ms)`;
}

function nowToTimestamp() {
  const now = new Date();
  state.timestamp = String(now.getTime());
  state.dateText = now.toLocaleString();
  state.output = `${Math.floor(now.getTime() / 1000)} (s)\n${now.getTime()} (ms)`;
  state.error = '';
}
</script>
