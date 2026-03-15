<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入</div>
        <textarea v-model="state.input" class="textarea"></textarea>
        <div class="inline-row">
          <label class="inline-label">算法</label>
          <select v-model="state.algorithm" class="select">
            <option value="SHA-1">SHA-1</option>
            <option value="SHA-256">SHA-256</option>
            <option value="SHA-384">SHA-384</option>
            <option value="SHA-512">SHA-512</option>
          </select>
        </div>
        <div class="btn-row">
          <button type="button" class="btn btn-primary" :disabled="state.busy" @click="computeHash">
            {{ state.busy ? '计算中...' : '计算' }}
          </button>
          <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制输出</button>
        </div>
        <div v-if="state.error" class="error">{{ state.error }}</div>
        <div class="hint">基于 WebCrypto（crypto.subtle.digest），不提供 MD5。</div>
      </div>
      <div class="tool-col">
        <div class="field-label">输出（Hex）</div>
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

type HashAlgorithm = 'SHA-1' | 'SHA-256' | 'SHA-384' | 'SHA-512';

const state = reactive({
  input: '',
  algorithm: 'SHA-256' as HashAlgorithm,
  output: '',
  busy: false,
  error: '',
});

function reset() {
  state.input = '';
  state.algorithm = 'SHA-256';
  state.output = '';
  state.busy = false;
  state.error = '';
}

watch(() => props.resetSeq, reset);

const HEX_PAD = 2;

function bytesToHex(bytes: Uint8Array) {
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(HEX_PAD, '0'))
    .join('');
}

async function computeHash() {
  state.error = '';
  state.output = '';
  if (!state.input) {
    state.error = '请输入要计算的文本';
    return;
  }
  if (!crypto?.subtle?.digest) {
    state.error = '当前环境不支持 WebCrypto（crypto.subtle.digest）';
    return;
  }

  state.busy = true;
  try {
    const data = new TextEncoder().encode(state.input);
    const digest = await crypto.subtle.digest(state.algorithm, data);
    state.output = bytesToHex(new Uint8Array(digest));
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'Hash 计算失败';
  } finally {
    state.busy = false;
  }
}
</script>
