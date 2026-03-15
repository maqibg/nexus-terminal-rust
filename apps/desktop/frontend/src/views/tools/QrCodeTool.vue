<template>
  <div class="tool-body">
    <div class="inline-row">
      <label class="inline-label">内容</label>
      <input v-model="state.text" class="input input-wide" placeholder="输入要生成二维码的内容" />
      <button type="button" class="btn btn-primary" @click="generateQr">生成</button>
    </div>
    <div v-if="state.error" class="error">{{ state.error }}</div>
    <div v-if="state.dataUrl" class="qr-preview">
      <img :src="state.dataUrl" class="qr-img" alt="QR Code" />
      <div class="btn-row">
        <button type="button" class="btn btn-ghost" @click="downloadQr">下载 PNG</button>
        <button type="button" class="btn btn-ghost" @click="copyText(state.text)">复制内容</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import QRCode from 'qrcode';
import { useCopyText } from './shared/useCopyText';

const QR_IMAGE_WIDTH = 256;
const QR_IMAGE_MARGIN = 1;

const props = defineProps<{ resetSeq: number }>();
const copyText = useCopyText();

const state = reactive({ text: '', dataUrl: '', error: '' });

function reset() {
  state.text = '';
  state.dataUrl = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

async function generateQr() {
  state.error = '';
  state.dataUrl = '';
  const text = state.text.trim();
  if (!text) {
    state.error = '请输入二维码内容';
    return;
  }

  try {
    state.dataUrl = await QRCode.toDataURL(text, { margin: QR_IMAGE_MARGIN, width: QR_IMAGE_WIDTH });
  } catch (err) {
    state.error = err instanceof Error ? err.message : '二维码生成失败';
  }
}

function downloadQr() {
  if (!state.dataUrl) {
    return;
  }
  const a = document.createElement('a');
  a.href = state.dataUrl;
  a.download = 'qrcode.png';
  a.click();
}
</script>
