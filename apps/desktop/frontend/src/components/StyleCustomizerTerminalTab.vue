<template>
  <section class="tab-section">
    <h3 class="section-title">终端样式</h3>

    <div class="field-row">
      <label>字体族</label>
      <div class="inline-save">
        <input class="input" v-model="fontFamily" />
        <button class="btn-save" @click="save('terminal_font_family', fontFamily)">保存</button>
      </div>
    </div>

    <div class="field-row">
      <label>字体大小</label>
      <div class="inline-save">
        <input class="input num" type="number" v-model.number="fontSize" min="8" max="36" />
        <button class="btn-save" @click="save('terminal_font_size', String(fontSize))">保存</button>
      </div>
    </div>

    <div class="field-row">
      <label>光标样式</label>
      <select class="input" v-model="cursorStyle" @change="save('terminal_cursor_style', cursorStyle)">
        <option value="block">Block</option>
        <option value="underline">Underline</option>
        <option value="bar">Bar</option>
      </select>
    </div>

    <div class="field-row">
      <label class="checkbox-row">
        <input type="checkbox" v-model="cursorBlink" @change="save('terminal_cursor_blink', String(cursorBlink))" />
        光标闪烁
      </label>
    </div>

    <hr class="divider" />
    <h4 class="sub-title">终端主题</h4>

    <div class="theme-list">
      <div v-for="theme in themes" :key="theme.id" class="theme-item" :class="{ active: theme.id === activeThemeId }">
        <span class="theme-name">{{ theme.name }}</span>
        <div class="theme-actions">
          <button class="btn-sm" @click="applyTheme(theme.id)" :disabled="theme.id === activeThemeId">应用</button>
          <button class="btn-sm danger" @click="deleteTheme(theme.id)" v-if="theme.theme_type !== 'preset'">删除</button>
        </div>
      </div>
      <div v-if="!themes.length" class="empty">暂无主题</div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { settingsApi, type TerminalTheme } from '@/lib/api';
import { useAppearanceStore } from '@/stores/appearance';
import { useUINotificationStore } from '@/stores/uiNotifications';

const appearance = useAppearanceStore();
const notify = useUINotificationStore();

const fontFamily = ref('monospace');
const fontSize = ref(14);
const cursorStyle = ref('block');
const cursorBlink = ref(true);
const themes = ref<TerminalTheme[]>([]);
const activeThemeId = ref(0);

onMounted(async () => {
  fontFamily.value = appearance.get('terminal_font_family', 'monospace');
  fontSize.value = parseInt(appearance.get('terminal_font_size', '14'));
  cursorStyle.value = appearance.get('terminal_cursor_style', 'block');
  cursorBlink.value = appearance.get('terminal_cursor_blink', 'true') === 'true';
  activeThemeId.value = parseInt(appearance.get('terminal_theme_id', '0'));
  try { themes.value = await settingsApi.themeList(); } catch { /* ignore */ }
});

async function save(key: string, value: string) {
  try {
    await appearance.set(key, value);
    notify.addNotification('success', '已保存');
  } catch (e: any) { notify.addNotification('error', e.message); }
}

async function applyTheme(id: number) {
  await save('terminal_theme_id', String(id));
  activeThemeId.value = id;
}

async function deleteTheme(id: number) {
  if (!confirm('确定删除此主题？')) return;
  try {
    await settingsApi.themeDelete(id);
    themes.value = themes.value.filter(t => t.id !== id);
    notify.addNotification('success', '主题已删除');
  } catch (e: any) { notify.addNotification('error', e.message); }
}
</script>

<style scoped>
.tab-section { display: flex; flex-direction: column; gap: 12px; }
.section-title { font-size: 15px; font-weight: 600; margin: 0 0 4px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.sub-title { font-size: 14px; font-weight: 600; margin: 0; }
.field-row { display: flex; flex-direction: column; gap: 4px; }
.field-row label { font-size: 12px; color: var(--text-sub); }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 6px 8px; color: var(--text); font-size: 13px; outline: none; }
.input:focus { border-color: var(--blue); }
.input.num { width: 80px; }
.inline-save { display: flex; gap: 6px; align-items: center; }
.inline-save .input { flex: 1; }
.checkbox-row { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--text-sub); cursor: pointer; }
.divider { border: none; border-top: 1px solid var(--border); margin: 4px 0; }
.btn-save { padding: 4px 14px; border-radius: 4px; border: none; background: var(--blue); color: var(--bg-base); cursor: pointer; font-size: 13px; font-weight: 600; flex-shrink: 0; }
.btn-save:hover { opacity: 0.9; }
.theme-list { display: flex; flex-direction: column; gap: 2px; max-height: 200px; overflow-y: auto; }
.theme-item { display: flex; justify-content: space-between; align-items: center; padding: 6px 8px; border-radius: 4px; background: var(--bg-mantle); font-size: 13px; }
.theme-item.active { border: 1px solid var(--blue); }
.theme-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.theme-actions { display: flex; gap: 4px; flex-shrink: 0; }
.btn-sm { padding: 2px 8px; border-radius: 3px; border: 1px solid var(--border); background: transparent; color: var(--text); cursor: pointer; font-size: 12px; }
.btn-sm:hover { background: var(--bg-surface1); }
.btn-sm:disabled { opacity: 0.4; cursor: default; }
.btn-sm.danger { color: var(--red); border-color: var(--red); }
.btn-sm.danger:hover { background: rgba(243,139,168,0.1); }
.empty { text-align: center; color: var(--text-dim); font-size: 13px; padding: 12px; }
</style>
