<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入 JWT</div>
        <textarea v-model="state.input" class="textarea" placeholder="xxxxx.yyyyy.zzzzz"></textarea>
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="decodeJwt">解析</button>
          <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制输出</button>
        </div>
        <div v-if="state.error" class="error">{{ state.error }}</div>
        <div class="hint">仅解析 Header/Payload，不验签。</div>
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
import { decodeBase64Utf8 } from './shared/codecs';
import { useCopyText } from './shared/useCopyText';

const props = defineProps<{ resetSeq: number }>();
const copyText = useCopyText();

const state = reactive({ input: '', output: '', error: '' });

function reset() {
  state.input = '';
  state.output = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

function base64UrlToBase64(input: string) {
  const normalized = input.replaceAll('-', '+').replaceAll('_', '/');
  const pad = normalized.length % 4;
  return pad === 0 ? normalized : normalized + '='.repeat(4 - pad);
}

function decodeJwtPart(part: string) {
  const json = decodeBase64Utf8(base64UrlToBase64(part));
  return JSON.parse(json) as unknown;
}

function decodeJwt() {
  state.error = '';
  state.output = '';
  const raw = state.input.trim();
  const parts = raw.split('.');
  if (parts.length < 2) {
    state.error = 'JWT 格式不正确（至少应包含 header.payload）';
    return;
  }

  try {
    const header = decodeJwtPart(parts[0]);
    const payload = decodeJwtPart(parts[1]);
    state.output = JSON.stringify({ header, payload }, null, 2);
  } catch (err) {
    state.error = err instanceof Error ? err.message : 'JWT 解析失败';
  }
}
</script>
