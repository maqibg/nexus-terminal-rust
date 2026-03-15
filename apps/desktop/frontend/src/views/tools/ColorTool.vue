<template>
  <div class="tool-body">
    <div class="tool-grid">
      <div class="tool-col">
        <div class="field-label">输入（HEX 或 RGB）</div>
        <input v-model="state.input" class="input" placeholder="#ff00aa 或 rgb(255,0,170)" />
        <div class="btn-row">
          <button type="button" class="btn btn-primary" @click="convertColor">转换</button>
          <button type="button" class="btn btn-ghost" @click="copyText(state.output)">复制输出</button>
        </div>
        <div v-if="state.error" class="error">{{ state.error }}</div>
      </div>
      <div class="tool-col">
        <div class="field-label">输出</div>
        <textarea v-model="state.output" class="textarea" readonly></textarea>
      </div>
    </div>

    <div v-if="state.preview" class="color-preview">
      <div class="color-swatch" :style="{ background: state.preview }"></div>
      <div class="hint">{{ state.preview }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import { useCopyText } from './shared/useCopyText';

const props = defineProps<{ resetSeq: number }>();
const copyText = useCopyText();

const state = reactive({ input: '', output: '', preview: '', error: '' });

function reset() {
  state.input = '';
  state.output = '';
  state.preview = '';
  state.error = '';
}

watch(() => props.resetSeq, reset);

const RGB_MAX = 255;

function clampByte(n: number) {
  return Math.max(0, Math.min(RGB_MAX, Math.round(n)));
}

function rgbToHex(r: number, g: number, b: number) {
  return `#${[r, g, b]
    .map(v => clampByte(v).toString(16).padStart(2, '0'))
    .join('')}`;
}

function parseHexColor(input: string) {
  const raw = input.trim();
  const m = raw.match(/^#?([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/);
  if (!m) {
    return null;
  }

  let hex = m[1];
  if (hex.length === 3) {
    hex = hex
      .split('')
      .map(ch => ch + ch)
      .join('');
  }

  const r = Number.parseInt(hex.slice(0, 2), 16);
  const g = Number.parseInt(hex.slice(2, 4), 16);
  const b = Number.parseInt(hex.slice(4, 6), 16);
  return { r, g, b };
}

function parseRgbColor(input: string) {
  const raw = input.trim();
  const m = raw.match(/^rgb\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)$/i);
  if (!m) {
    return null;
  }
  return { r: clampByte(Number(m[1])), g: clampByte(Number(m[2])), b: clampByte(Number(m[3])) };
}

function convertColor() {
  state.error = '';
  state.output = '';
  state.preview = '';

  const hex = parseHexColor(state.input);
  if (hex) {
    state.preview = rgbToHex(hex.r, hex.g, hex.b);
    state.output = `rgb(${hex.r}, ${hex.g}, ${hex.b})\n${state.preview}`;
    return;
  }

  const rgb = parseRgbColor(state.input);
  if (rgb) {
    const outHex = rgbToHex(rgb.r, rgb.g, rgb.b);
    state.preview = outHex;
    state.output = `${outHex}\nrgb(${rgb.r}, ${rgb.g}, ${rgb.b})`;
    return;
  }

  state.error = '无法识别颜色格式（支持 #rgb/#rrggbb 或 rgb(r,g,b)）';
}
</script>
