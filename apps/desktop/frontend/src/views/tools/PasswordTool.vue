<template>
  <div class="tool-body">
    <div class="inline-row">
      <label class="inline-label">长度</label>
      <input v-model.number="state.length" type="number" :min="PASSWORD_MIN_LENGTH" :max="PASSWORD_MAX_LENGTH" class="input input-count" />
      <label class="checkbox"><input v-model="state.digits" type="checkbox" />数字</label>
      <label class="checkbox"><input v-model="state.lower" type="checkbox" />小写</label>
      <label class="checkbox"><input v-model="state.upper" type="checkbox" />大写</label>
      <label class="checkbox"><input v-model="state.symbols" type="checkbox" />符号</label>
      <button type="button" class="btn btn-primary" @click="generatePassword">生成</button>
      <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制</button>
    </div>
    <div v-if="state.error" class="error">{{ state.error }}</div>
    <textarea v-model="state.output" class="textarea" readonly></textarea>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import { useCopyText } from './shared/useCopyText';

const PASSWORD_MIN_LENGTH = 4;
const PASSWORD_MAX_LENGTH = 128;
const DEFAULT_PASSWORD_LENGTH = 16;
const DIGITS_ALPHABET = '0123456789';
const LOWER_ALPHABET = 'abcdefghijklmnopqrstuvwxyz';
const UPPER_ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
const SYMBOL_ALPHABET = '!@#$%^&*()-_=+[]{};:,.?';

const props = defineProps<{ resetSeq: number }>();
const copyText = useCopyText();

const state = reactive({
  length: DEFAULT_PASSWORD_LENGTH,
  digits: true,
  lower: true,
  upper: true,
  symbols: false,
  output: '',
  error: '',
});

function reset() {
  state.length = DEFAULT_PASSWORD_LENGTH;
  state.digits = true;
  state.lower = true;
  state.upper = true;
  state.symbols = false;
  state.output = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

function getAlphabet() {
  let alphabet = '';
  if (state.digits) alphabet += DIGITS_ALPHABET;
  if (state.lower) alphabet += LOWER_ALPHABET;
  if (state.upper) alphabet += UPPER_ALPHABET;
  if (state.symbols) alphabet += SYMBOL_ALPHABET;
  return alphabet;
}

function generatePassword() {
  state.error = '';
  state.output = '';

  const length = Number.isFinite(state.length) ? Math.floor(state.length) : 0;
  if (length < PASSWORD_MIN_LENGTH || length > PASSWORD_MAX_LENGTH) {
    state.error = `长度需在 ${PASSWORD_MIN_LENGTH}~${PASSWORD_MAX_LENGTH} 之间`;
    return;
  }
  if (!crypto?.getRandomValues) {
    state.error = '当前环境不支持安全随机数（crypto.getRandomValues）';
    return;
  }

  const alphabet = getAlphabet();
  if (!alphabet) {
    state.error = '至少选择一种字符集';
    return;
  }

  const bytes = new Uint32Array(length);
  crypto.getRandomValues(bytes);

  const chars: string[] = [];
  for (let i = 0; i < length; i++) {
    chars.push(alphabet[bytes[i] % alphabet.length]);
  }
  state.output = chars.join('');
}
</script>
