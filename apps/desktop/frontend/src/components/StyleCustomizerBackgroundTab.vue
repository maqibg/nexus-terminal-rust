<template>
  <section class="tab-section">
    <h3 class="section-title">背景设置</h3>

    <div class="field-row">
      <label>背景类型</label>
      <select class="input" v-model="bgType" @change="save('background_type', bgType)">
        <option value="none">无</option>
        <option value="color">纯色</option>
        <option value="gradient">渐变</option>
        <option value="image">图片</option>
      </select>
    </div>

    <template v-if="bgType === 'color'">
      <div class="field-row">
        <label>背景颜色</label>
        <div class="color-field">
          <input type="color" :value="bgColor" @change="save('background_color', ($event.target as HTMLInputElement).value)" />
          <input class="input" :value="bgColor" @change="save('background_color', ($event.target as HTMLInputElement).value)" />
        </div>
      </div>
    </template>

    <template v-if="bgType === 'gradient'">
      <div class="field-row">
        <label>渐变 CSS</label>
        <input class="input" v-model="bgGradient" placeholder="linear-gradient(135deg, #1e1e2e, #313244)" />
      </div>
      <button class="btn-save" @click="save('background_gradient', bgGradient)">保存</button>
    </template>

    <template v-if="bgType === 'image'">
      <div class="field-row">
        <label>图片 URL</label>
        <input class="input" v-model="bgImage" placeholder="https://..." />
      </div>
      <div class="field-row">
        <label>蒙版透明度</label>
        <div class="range-row">
          <input type="range" v-model.number="bgOverlay" min="0" max="1" step="0.01" />
          <span class="range-val">{{ bgOverlay.toFixed(2) }}</span>
        </div>
      </div>
      <button class="btn-save" @click="saveImageSettings">保存</button>
    </template>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAppearanceStore } from '@/stores/appearance';
import { useUINotificationStore } from '@/stores/uiNotifications';

const appearance = useAppearanceStore();
const notify = useUINotificationStore();

const bgType = ref('none');
const bgColor = ref('#1e1e2e');
const bgGradient = ref('');
const bgImage = ref('');
const bgOverlay = ref(0.5);

onMounted(() => {
  bgType.value = appearance.get('background_type', 'none');
  bgColor.value = appearance.get('background_color', '#1e1e2e');
  bgGradient.value = appearance.get('background_gradient', '');
  bgImage.value = appearance.get('background_image', '');
  bgOverlay.value = parseFloat(appearance.get('background_overlay', '0.5'));
});

async function save(key: string, value: string) {
  try {
    await appearance.set(key, value);
    notify.addNotification('success', '已保存');
  } catch (e: any) { notify.addNotification('error', e.message); }
}

async function saveImageSettings() {
  await save('background_image', bgImage.value);
  await save('background_overlay', bgOverlay.value.toString());
}
</script>

<style scoped>
.tab-section { display: flex; flex-direction: column; gap: 12px; }
.section-title { font-size: 15px; font-weight: 600; margin: 0 0 4px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.field-row { display: flex; flex-direction: column; gap: 4px; }
.field-row label { font-size: 12px; color: var(--text-sub); }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 6px 8px; color: var(--text); font-size: 13px; outline: none; }
.input:focus { border-color: var(--blue); }
.color-field { display: flex; gap: 8px; align-items: center; }
.color-field input[type="color"] { width: 32px; height: 32px; border: 1px solid var(--border); border-radius: 4px; padding: 2px; cursor: pointer; }
.range-row { display: flex; align-items: center; gap: 8px; }
.range-row input[type="range"] { flex: 1; accent-color: var(--blue); }
.range-val { font-size: 12px; color: var(--text-sub); min-width: 36px; text-align: right; }
.btn-save { align-self: flex-start; padding: 4px 14px; border-radius: 4px; border: none; background: var(--blue); color: var(--bg-base); cursor: pointer; font-size: 13px; font-weight: 600; }
.btn-save:hover { opacity: 0.9; }
</style>
