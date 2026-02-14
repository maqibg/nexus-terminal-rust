<template>
  <section class="tab-section">
    <h3 class="section-title">UI 颜色变量</h3>
    <p class="desc">编辑 CSS 变量以自定义界面颜色，修改后点击保存生效。</p>

    <div v-for="(value, key) in vars" :key="key" class="var-row">
      <label class="var-label">{{ key }}</label>
      <div class="var-input">
        <input v-if="isColor(value)" type="color" :value="value" @change="vars[key] = ($event.target as HTMLInputElement).value" class="color-picker" />
        <input class="input" v-model="vars[key]" />
      </div>
    </div>

    <div class="actions">
      <button class="btn-reset" @click="reset">重置默认</button>
      <button class="btn-save" @click="saveAll">保存</button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue';
import { useAppearanceStore } from '@/stores/appearance';
import { useUINotificationStore } from '@/stores/uiNotifications';

const appearance = useAppearanceStore();
const notify = useUINotificationStore();

const DEFAULT_VARS: Record<string, string> = {
  '--bg-base': '#1e1e2e', '--bg-mantle': '#181825', '--bg-surface0': '#313244',
  '--bg-surface1': '#45475a', '--text': '#cdd6f4', '--text-sub': '#a6adc8',
  '--text-dim': '#6c7086', '--border': '#45475a', '--blue': '#89b4fa',
  '--green': '#a6e3a1', '--red': '#f38ba8', '--yellow': '#f9e2af',
  '--peach': '#fab387', '--mauve': '#cba6f7',
};

const vars = reactive<Record<string, string>>({});

onMounted(() => {
  for (const [key, def] of Object.entries(DEFAULT_VARS)) {
    vars[key] = appearance.get(key, def);
  }
});

function isColor(v: string): boolean {
  return /^#[0-9a-fA-F]{3,8}$/.test(v);
}

async function saveAll() {
  try {
    for (const [key, value] of Object.entries(vars)) {
      await appearance.set(key, value);
    }
    notify.addNotification('success', 'UI 主题已保存');
  } catch (e: any) { notify.addNotification('error', e.message); }
}

async function reset() {
  for (const [key, def] of Object.entries(DEFAULT_VARS)) vars[key] = def;
  await saveAll();
}
</script>

<style scoped>
.tab-section { display: flex; flex-direction: column; gap: 10px; }
.section-title { font-size: 15px; font-weight: 600; margin: 0 0 4px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.desc { font-size: 12px; color: var(--text-sub); margin: 0; }
.var-row { display: flex; align-items: center; gap: 8px; }
.var-label { font-size: 12px; color: var(--text-sub); min-width: 120px; font-family: monospace; }
.var-input { display: flex; align-items: center; gap: 6px; flex: 1; }
.color-picker { width: 28px; height: 28px; border: 1px solid var(--border); border-radius: 4px; padding: 1px; cursor: pointer; flex-shrink: 0; }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 4px 8px; color: var(--text); font-size: 12px; outline: none; flex: 1; font-family: monospace; }
.input:focus { border-color: var(--blue); }
.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
.btn-save { padding: 5px 16px; border-radius: 4px; border: none; background: var(--blue); color: var(--bg-base); cursor: pointer; font-size: 13px; font-weight: 600; }
.btn-reset { padding: 5px 16px; border-radius: 4px; border: 1px solid var(--border); background: transparent; color: var(--text-sub); cursor: pointer; font-size: 13px; }
.btn-reset:hover { background: var(--bg-surface1); }
</style>
