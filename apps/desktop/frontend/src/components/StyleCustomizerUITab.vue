<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { defaultUiTheme, safeJsonParse, useAppearanceStore } from '@/stores/appearance';
import { useUiNotificationsStore } from '@/stores/uiNotifications';

const appearanceStore = useAppearanceStore();
const notificationsStore = useUiNotificationsStore();
const { appearanceSettings } = storeToRefs(appearanceStore);

const editableUiTheme = ref<Record<string, string>>({});
const editableUiThemeString = ref('');
const themeParseError = ref<string | null>(null);

const darkModeTheme: Record<string, string> = {
  '--app-bg-color': '#212529',
  '--text-color': '#e9ecef',
  '--text-color-secondary': '#adb5bd',
  '--border-color': '#495057',
  '--link-color': '#BB86FC',
  '--link-hover-color': '#D1A9FF',
  '--link-active-color': '#A06CD5',
  '--link-active-bg-color': 'rgba(160, 108, 213, 0.2)',
  '--nav-item-active-bg-color': 'var(--link-active-bg-color)',
  '--header-bg-color': '#343a40',
  '--footer-bg-color': '#343a40',
  '--button-bg-color': 'var(--link-active-color)',
  '--button-text-color': '#ffffff',
  '--button-hover-bg-color': '#8E44AD',
  '--icon-color': 'var(--text-color-secondary)',
  '--icon-hover-color': 'var(--link-hover-color)',
  '--split-line-color': 'var(--border-color)',
  '--split-line-hover-color': 'var(--border-color)',
  '--input-focus-border-color': 'var(--link-active-color)',
  '--input-focus-glow': 'var(--link-active-color)',
  '--overlay-bg-color': 'rgba(0, 0, 0, 0.8)',
  '--color-success': '#5cb85c',
  '--color-error': '#d9534f',
  '--color-warning': '#f0ad4e',
  '--font-family-sans-serif': 'sans-serif',
  '--base-padding': '1rem',
  '--base-margin': '0.5rem',
};

const initializeEditableState = () => {
  const userTheme = safeJsonParse<Record<string, string>>(appearanceSettings.value.customUiTheme, {});
  const mergedTheme = { ...defaultUiTheme, ...userTheme };
  editableUiTheme.value = JSON.parse(JSON.stringify(mergedTheme));
  themeParseError.value = null;

  const lines = Object.entries(editableUiTheme.value).map(([key, value]) => `${key}: ${value}`);
  editableUiThemeString.value = lines.join('\n');
};

onMounted(initializeEditableState);

watch(
  () => appearanceSettings.value.customUiTheme,
  () => {
    initializeEditableState();
  },
  { deep: true },
);

const formattedEditableUiThemeJson = computed(() => {
  const themeObject = editableUiTheme.value;
  if (!themeObject || typeof themeObject !== 'object' || Object.keys(themeObject).length === 0) {
    return '';
  }
  return Object.entries(themeObject)
    .map(([key, value]) => `${key}: ${value}`)
    .join('\n');
});

watch(formattedEditableUiThemeJson, (newValue) => {
  if (document.activeElement?.id !== 'uiThemeTextarea' || themeParseError.value) {
    editableUiThemeString.value = newValue;
    if (themeParseError.value && document.activeElement?.id !== 'uiThemeTextarea') {
      themeParseError.value = null;
    }
  }
});

const handleSaveUiTheme = async () => {
  try {
    await appearanceStore.saveCustomUiTheme(editableUiTheme.value);
    notificationsStore.addNotification({ type: 'success', message: '界面主题已保存' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '界面主题保存失败' });
  }
};

const handleResetUiTheme = async () => {
  try {
    await appearanceStore.resetCustomUiTheme();
    notificationsStore.addNotification({ type: 'info', message: '已恢复默认模式' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '恢复默认模式失败' });
  }
};

const applyDarkMode = async () => {
  try {
    editableUiTheme.value = JSON.parse(JSON.stringify(darkModeTheme));
    await appearanceStore.saveCustomUiTheme(editableUiTheme.value);
    notificationsStore.addNotification({ type: 'success', message: '黑暗模式已应用' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '应用黑暗模式失败' });
  }
};

const handleUiThemeStringChange = () => {
  themeParseError.value = null;
  const inputText = editableUiThemeString.value.trim();

  if (!inputText) {
    editableUiTheme.value = {};
    return;
  }

  const jsonStringToParse = inputText
    .split('\n')
    .map(line => line.trim())
    .filter(line => line && line.includes(':'))
    .map(line => {
      const parts = line.split(/:(.*)/s);
      if (parts.length < 2) {
        return null;
      }

      let key = parts[0].trim();
      let value = parts[1].trim();

      if (key.startsWith('"') && key.endsWith('"')) {
        key = key.slice(1, -1);
      }
      if (key.startsWith("'") && key.endsWith("'")) {
        key = key.slice(1, -1);
      }
      key = JSON.stringify(key);

      if (value.endsWith(',')) {
        value = value.slice(0, -1).trim();
      }
      let originalValue = value;
      if (value.startsWith('"') && value.endsWith('"')) {
        originalValue = value.slice(1, -1);
      } else if (value.startsWith("'") && value.endsWith("'")) {
        originalValue = value.slice(1, -1);
      }

      if (
        Number.isNaN(Number(originalValue))
        && originalValue !== 'true'
        && originalValue !== 'false'
        && originalValue !== 'null'
      ) {
        value = JSON.stringify(originalValue);
      } else {
        value = originalValue;
      }

      return `  ${key}: ${value}`;
    })
    .filter((line): line is string => line !== null)
    .join(',\n');

  const fullJsonString = `{\n${jsonStringToParse}\n}`;

  try {
    const parsedTheme = JSON.parse(fullJsonString);
    if (typeof parsedTheme !== 'object' || parsedTheme === null || Array.isArray(parsedTheme)) {
      throw new Error('配置必须是对象格式');
    }
    editableUiTheme.value = parsedTheme;
  } catch (error: any) {
    themeParseError.value = error?.message ?? 'JSON 格式错误，请检查后重试';
  }
};

const formatLabel = (key: string): string => {
  return key
    .replace(/^--/, '')
    .replace(/-/g, ' ')
    .replace(/([A-Z])/g, ' $1')
    .replace(/^./, str => str.toUpperCase());
};

const handleFocusAndSelect = (event: FocusEvent) => {
  const target = event.target;
  if (target instanceof HTMLInputElement) {
    target.select();
  }
};

defineExpose({
  handleSaveUiTheme,
  handleResetUiTheme,
});
</script>

<template>
  <section class="ui-tab">
    <h3 class="section-title">界面样式</h3>

    <div class="mode-row">
      <label class="mode-label">主题模式:</label>
      <div class="mode-actions">
        <button type="button" class="mode-btn" @click="handleResetUiTheme">默认模式</button>
        <button type="button" class="mode-btn" @click="applyDarkMode">黑暗模式</button>
      </div>
    </div>

    <p class="section-desc">调整程序界面的颜色、边框和交互视觉风格。</p>

    <div class="theme-rows">
      <div v-for="(value, key) in editableUiTheme" :key="key" class="theme-row">
        <label :for="`ui-${key}`" class="theme-label">{{ formatLabel(key) }}:</label>
        <div class="theme-input-wrap">
          <input
            v-if="typeof value === 'string' && (value.startsWith('#') || value.startsWith('rgb') || value.startsWith('hsl'))"
            type="color"
            :id="`ui-${key}`"
            v-model="editableUiTheme[key]"
            class="color-input"
          />
          <input
            v-if="typeof value === 'string' && (value.startsWith('#') || value.startsWith('rgb') || value.startsWith('hsl'))"
            type="text"
            :value="editableUiTheme[key]"
            class="text-input"
            @focus="handleFocusAndSelect"
            @input="editableUiTheme[key] = ($event.target as HTMLInputElement).value"
          />
          <input
            v-else
            type="text"
            :id="`ui-${key}`"
            v-model="editableUiTheme[key]"
            class="text-input"
          />
        </div>
      </div>
    </div>

    <hr class="divider" />

    <h4 class="json-title">界面主题 JSON 编辑器</h4>
    <p class="section-desc">支持按 `变量: 值` 的形式批量编辑主题配置。</p>

    <div class="json-editor-wrap">
      <label for="uiThemeTextarea" class="sr-only">界面主题 JSON 编辑器</label>
      <textarea
        id="uiThemeTextarea"
        v-model="editableUiThemeString"
        @blur="handleUiThemeStringChange"
        rows="15"
        placeholder="--app-bg-color: #ffffff&#10;--text-color: #333333"
        spellcheck="false"
        class="json-textarea"
      />
    </div>

    <p v-if="themeParseError" class="error-message">{{ themeParseError }}</p>
  </section>
</template>

<style scoped>
.ui-tab {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  margin: 0;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border);
  font-size: 19px;
  line-height: 1.2;
  color: var(--text);
  font-weight: 600;
}

.mode-row {
  display: grid;
  grid-template-columns: 110px 1fr;
  gap: 8px;
  align-items: center;
  margin-top: 2px;
}

.mode-label {
  color: var(--text);
  font-size: 14px;
  font-weight: 500;
}

.mode-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.mode-btn {
  height: 32px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--header-bg-color);
  color: var(--text);
  padding: 0 12px;
  font-size: 13px;
  cursor: pointer;
}

.mode-btn:hover {
  background: var(--bg-surface1);
}

.section-desc {
  margin: 0;
  color: var(--text-sub);
  font-size: 13px;
  line-height: 1.6;
}

.theme-rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.theme-row {
  display: grid;
  grid-template-columns: 220px 1fr;
  gap: 10px;
  align-items: center;
}

.theme-label {
  color: var(--text);
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.theme-input-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.color-input {
  width: 44px;
  height: 32px;
  border-radius: 4px;
  border: 1px solid var(--border);
  background: transparent;
  padding: 2px;
}

.text-input {
  flex: 1;
  min-width: 0;
  height: 32px;
  border-radius: 4px;
  border: 1px solid var(--border);
  background: var(--app-bg-color);
  color: var(--text);
  padding: 0 10px;
  font-size: 13px;
}

.text-input:focus {
  outline: none;
  border-color: var(--input-focus-border-color);
}

.divider {
  border: none;
  border-top: 1px solid var(--border);
  margin: 14px 0 0;
}

.json-title {
  margin: 0;
  font-size: 16px;
  color: var(--text);
  font-weight: 600;
}

.json-editor-wrap {
  margin-top: 2px;
}

.json-textarea {
  width: 100%;
  min-height: 220px;
  resize: vertical;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--app-bg-color);
  color: var(--text);
  padding: 10px 12px;
  font-size: 13px;
  font-family: 'Cascadia Mono', Consolas, 'Courier New', monospace;
  line-height: 1.45;
}

.json-textarea:focus {
  outline: none;
  border-color: var(--input-focus-border-color);
}

.error-message {
  margin: 0;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--color-error) 45%, transparent);
  background: color-mix(in srgb, var(--color-error) 14%, transparent);
  color: var(--color-error);
  padding: 8px 10px;
  font-size: 12px;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  border: 0;
}

@media (max-width: 860px) {
  .mode-row,
  .theme-row {
    grid-template-columns: 1fr;
    gap: 6px;
  }
}
</style>
