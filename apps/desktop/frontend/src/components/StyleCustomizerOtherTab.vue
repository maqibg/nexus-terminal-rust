<template>
  <section class="tab-section">
    <h3 class="section-title">其他设置</h3>

    <div class="field-row">
      <label>编辑器字体大小</label>
      <div class="inline-save">
        <input class="input num" type="number" v-model.number="editorFontSize" min="8" max="36" />
        <button class="btn-save" @click="save('editor_font_size', String(editorFontSize))">保存</button>
      </div>
    </div>

    <div class="field-row">
      <label>编辑器字体族</label>
      <div class="inline-save">
        <input class="input" v-model="editorFontFamily" />
        <button class="btn-save" @click="save('editor_font_family', editorFontFamily)">保存</button>
      </div>
    </div>

    <div class="field-row">
      <label>侧边栏宽度 (px)</label>
      <div class="inline-save">
        <input class="input num" type="number" v-model.number="sidebarWidth" min="150" max="500" />
        <button class="btn-save" @click="save('sidebar_width', String(sidebarWidth))">保存</button>
      </div>
    </div>

    <div class="field-row">
      <label class="checkbox-row">
        <input type="checkbox" v-model="animationEnabled" @change="save('animation_enabled', String(animationEnabled))" />
        启用动画
      </label>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAppearanceStore } from '@/stores/appearance';
import { useUINotificationStore } from '@/stores/uiNotifications';

const appearance = useAppearanceStore();
const notify = useUINotificationStore();

const editorFontSize = ref(14);
const editorFontFamily = ref('monospace');
const sidebarWidth = ref(240);
const animationEnabled = ref(true);

onMounted(() => {
  editorFontSize.value = parseInt(appearance.get('editor_font_size', '14'));
  editorFontFamily.value = appearance.get('editor_font_family', 'monospace');
  sidebarWidth.value = parseInt(appearance.get('sidebar_width', '240'));
  animationEnabled.value = appearance.get('animation_enabled', 'true') === 'true';
});

async function save(key: string, value: string) {
  try {
    await appearance.set(key, value);
    notify.addNotification('success', '已保存');
  } catch (e: any) { notify.addNotification('error', e.message); }
}
</script>

<style scoped>
.tab-section { display: flex; flex-direction: column; gap: 12px; }
.section-title { font-size: 15px; font-weight: 600; margin: 0 0 4px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.field-row { display: flex; flex-direction: column; gap: 4px; }
.field-row label { font-size: 12px; color: var(--text-sub); }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 6px 8px; color: var(--text); font-size: 13px; outline: none; }
.input:focus { border-color: var(--blue); }
.input.num { width: 80px; }
.inline-save { display: flex; gap: 6px; align-items: center; }
.inline-save .input { flex: 1; }
.checkbox-row { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--text-sub); cursor: pointer; }
.btn-save { padding: 4px 14px; border-radius: 4px; border: none; background: var(--blue); color: var(--bg-base); cursor: pointer; font-size: 13px; font-weight: 600; flex-shrink: 0; }
.btn-save:hover { opacity: 0.9; }
</style>
