<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import type { ITheme } from '@xterm/xterm';
import type { TerminalTheme } from '@/stores/appearance';
import { defaultXtermTheme, useAppearanceStore } from '@/stores/appearance';
import { useUiNotificationsStore } from '@/stores/uiNotifications';

const appearanceStore = useAppearanceStore();
const notificationsStore = useUiNotificationsStore();

const props = defineProps<{
  isEditingTheme: boolean;
  editingTheme: TerminalTheme | null;
  modalRootRef?: HTMLDivElement | null;
}>();

const emit = defineEmits<{
  (e: 'update:isEditingTheme', value: boolean): void;
  (e: 'update:editingTheme', value: TerminalTheme | null): void;
}>();

const {
  allTerminalThemes,
  activeTerminalThemeId,
  currentTerminalFontFamily,
  currentTerminalFontSize,
  terminalTextStrokeEnabled,
  terminalTextStrokeWidth,
  terminalTextStrokeColor,
  terminalTextShadowEnabled,
  terminalTextShadowOffsetX,
  terminalTextShadowOffsetY,
  terminalTextShadowBlur,
  terminalTextShadowColor,
} = storeToRefs(appearanceStore);

const editableTerminalFontFamily = ref('');
const editableTerminalFontSize = ref(14);

const editableTerminalTextStrokeEnabled = ref(false);
const editableTerminalTextStrokeWidth = ref(1);
const editableTerminalTextStrokeColor = ref('#000000');

const editableTerminalTextShadowEnabled = ref(false);
const editableTerminalTextShadowOffsetX = ref(0);
const editableTerminalTextShadowOffsetY = ref(0);
const editableTerminalTextShadowBlur = ref(0);
const editableTerminalTextShadowColor = ref('rgba(0, 0, 0, 0.5)');

const themeSearchTerm = ref('');
const saveThemeError = ref<string | null>(null);
const editableTerminalThemeString = ref('');
const terminalThemeParseError = ref<string | null>(null);

const terminalThemePlaceholder = `background: #000000
foreground: #ffffff
cursor: #ffffff
selectionBackground: #555555
black: #000000
red: #ff0000
green: #00ff00
yellow: #ffff00
blue: #0000ff
magenta: #ff00ff
cyan: #00ffff
white: #ffffff
brightBlack: #555555
brightRed: #ff5555
brightGreen: #55ff55
brightYellow: #ffff55
brightBlue: #5555ff
brightMagenta: #ff55ff
brightCyan: #55ffff
brightWhite: #ffffff`;

const initializeEditableState = () => {
  editableTerminalFontFamily.value = currentTerminalFontFamily.value;
  editableTerminalFontSize.value = currentTerminalFontSize.value;

  editableTerminalTextStrokeEnabled.value = terminalTextStrokeEnabled.value;
  editableTerminalTextStrokeWidth.value = terminalTextStrokeWidth.value;
  editableTerminalTextStrokeColor.value = terminalTextStrokeColor.value;

  editableTerminalTextShadowEnabled.value = terminalTextShadowEnabled.value;
  editableTerminalTextShadowOffsetX.value = terminalTextShadowOffsetX.value;
  editableTerminalTextShadowOffsetY.value = terminalTextShadowOffsetY.value;
  editableTerminalTextShadowBlur.value = terminalTextShadowBlur.value;
  editableTerminalTextShadowColor.value = terminalTextShadowColor.value;

  saveThemeError.value = null;
  terminalThemeParseError.value = null;
};

watch(
  () => [
    currentTerminalFontFamily.value,
    currentTerminalFontSize.value,
    terminalTextStrokeEnabled.value,
    terminalTextStrokeWidth.value,
    terminalTextStrokeColor.value,
    terminalTextShadowEnabled.value,
    terminalTextShadowOffsetX.value,
    terminalTextShadowOffsetY.value,
    terminalTextShadowBlur.value,
    terminalTextShadowColor.value,
  ],
  () => {
    if (!props.isEditingTheme) {
      initializeEditableState();
    }
  },
  { immediate: true, deep: true },
);

const notifyError = (fallback: string, error?: any) => {
  notificationsStore.addNotification({
    type: 'error',
    message: error?.message ?? fallback,
  });
};

const handleSaveTerminalFont = async () => {
  try {
    await appearanceStore.setTerminalFontFamily(editableTerminalFontFamily.value);
    notificationsStore.addNotification({ type: 'success', message: '终端字体已保存' });
  } catch (error: any) {
    notifyError('保存终端字体失败', error);
  }
};

const handleSaveTerminalFontSize = async () => {
  try {
    const size = Number(editableTerminalFontSize.value);
    if (Number.isNaN(size) || size <= 0) {
      notificationsStore.addNotification({ type: 'error', message: '终端字体大小必须大于 0' });
      return;
    }
    await appearanceStore.setTerminalFontSize(size);
    notificationsStore.addNotification({ type: 'success', message: '终端字体大小已保存' });
  } catch (error: any) {
    notifyError('保存终端字体大小失败', error);
  }
};

const handleSaveTerminalTextStroke = async () => {
  try {
    await appearanceStore.setTerminalTextStrokeEnabled(editableTerminalTextStrokeEnabled.value);
    await appearanceStore.setTerminalTextStrokeWidth(Number(editableTerminalTextStrokeWidth.value));
    await appearanceStore.setTerminalTextStrokeColor(editableTerminalTextStrokeColor.value);
    notificationsStore.addNotification({ type: 'success', message: '文字描边设置已保存' });
  } catch (error: any) {
    notifyError('保存文字描边设置失败', error);
  }
};

const handleSaveTerminalTextShadow = async () => {
  try {
    await appearanceStore.setTerminalTextShadowEnabled(editableTerminalTextShadowEnabled.value);
    await appearanceStore.setTerminalTextShadowOffsetX(Number(editableTerminalTextShadowOffsetX.value));
    await appearanceStore.setTerminalTextShadowOffsetY(Number(editableTerminalTextShadowOffsetY.value));
    await appearanceStore.setTerminalTextShadowBlur(Number(editableTerminalTextShadowBlur.value));
    await appearanceStore.setTerminalTextShadowColor(editableTerminalTextShadowColor.value);
    notificationsStore.addNotification({ type: 'success', message: '文字阴影设置已保存' });
  } catch (error: any) {
    notifyError('保存文字阴影设置失败', error);
  }
};

const toggleTextStrokeAndSave = async () => {
  editableTerminalTextStrokeEnabled.value = !editableTerminalTextStrokeEnabled.value;
  await handleSaveTerminalTextStroke();
};

const toggleTextShadowAndSave = async () => {
  editableTerminalTextShadowEnabled.value = !editableTerminalTextShadowEnabled.value;
  await handleSaveTerminalTextShadow();
};

const handleApplyTheme = async (theme: TerminalTheme) => {
  if (!theme._id) {
    return;
  }
  if (String(activeTerminalThemeId.value) === theme._id) {
    return;
  }
  try {
    await appearanceStore.setActiveTerminalTheme(theme._id);
    notificationsStore.addNotification({ type: 'success', message: `已应用主题：${theme.name}` });
  } catch (error: any) {
    notifyError('应用终端主题失败', error);
  }
};

const handleAddNewTheme = () => {
  saveThemeError.value = null;
  terminalThemeParseError.value = null;

  const newTheme: TerminalTheme = {
    _id: undefined,
    name: '新主题',
    themeData: JSON.parse(JSON.stringify(defaultXtermTheme)),
    isPreset: false,
  };

  emit('update:editingTheme', newTheme);
  editableTerminalThemeString.value = Object.entries(newTheme.themeData)
    .map(([key, value]) => `${key}: ${value}`)
    .join('\n');
  emit('update:isEditingTheme', true);
};

const handleEditTheme = async (theme: TerminalTheme) => {
  if (!theme._id) {
    notifyError('无法编辑无效主题');
    return;
  }

  try {
    const themeDataToEdit = await appearanceStore.loadTerminalThemeData(theme._id);
    if (!themeDataToEdit) {
      throw new Error('加载主题详情失败');
    }

    const themeToEdit: TerminalTheme = {
      _id: theme.isPreset ? undefined : theme._id,
      name: theme.isPreset ? `${theme.name} (Copy)` : theme.name,
      themeData: JSON.parse(JSON.stringify(themeDataToEdit)),
      isPreset: false,
    };

    emit('update:editingTheme', themeToEdit);
    editableTerminalThemeString.value = Object.entries(themeToEdit.themeData)
      .map(([key, value]) => `${key}: ${value}`)
      .join('\n');
    emit('update:isEditingTheme', true);
  } catch (error: any) {
    saveThemeError.value = error?.message ?? '进入编辑状态失败';
    emit('update:isEditingTheme', false);
    emit('update:editingTheme', null);
  }
};

const handleSaveEditingTheme = async () => {
  if (!props.editingTheme || !props.editingTheme.name.trim()) {
    saveThemeError.value = '主题名称不能为空';
    return;
  }

  handleTerminalThemeStringChange();
  if (terminalThemeParseError.value) {
    saveThemeError.value = '请先修复 JSON 编辑器中的格式错误';
    return;
  }

  saveThemeError.value = null;
  try {
    if (!props.editingTheme) {
      return;
    }

    if (props.editingTheme._id) {
      await appearanceStore.updateTerminalTheme(
        props.editingTheme._id,
        props.editingTheme.name,
        props.editingTheme.themeData,
      );
      notificationsStore.addNotification({ type: 'success', message: '主题已更新' });
    } else {
      await appearanceStore.createTerminalTheme(props.editingTheme.name, props.editingTheme.themeData);
      notificationsStore.addNotification({ type: 'success', message: '主题已创建' });
    }

    emit('update:isEditingTheme', false);
    emit('update:editingTheme', null);
    editableTerminalThemeString.value = '';
    terminalThemeParseError.value = null;
  } catch (error: any) {
    saveThemeError.value = error?.message ?? '保存主题失败';
  }
};

const handleCancelEditingTheme = () => {
  emit('update:isEditingTheme', false);
  emit('update:editingTheme', null);
  saveThemeError.value = null;
  terminalThemeParseError.value = null;
  editableTerminalThemeString.value = '';
};

const handleTerminalThemeStringChange = () => {
  terminalThemeParseError.value = null;
  if (!props.editingTheme) {
    return;
  }

  const inputText = editableTerminalThemeString.value.trim();
  if (!inputText) {
    emit('update:editingTheme', {
      ...props.editingTheme,
      themeData: {},
    });
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
    const parsedThemeData = JSON.parse(fullJsonString) as ITheme;
    if (typeof parsedThemeData !== 'object' || parsedThemeData === null || Array.isArray(parsedThemeData)) {
      throw new Error('配置必须是对象格式');
    }

    emit('update:editingTheme', {
      ...props.editingTheme,
      themeData: parsedThemeData,
    });
  } catch (error: any) {
    terminalThemeParseError.value = error?.message ?? '终端主题 JSON 格式错误';
  }
};

const handleDeleteTheme = async (theme: TerminalTheme) => {
  if (theme.isPreset || !theme._id) {
    return;
  }
  try {
    await appearanceStore.deleteTerminalTheme(theme._id);
    notificationsStore.addNotification({ type: 'success', message: '主题已删除' });
  } catch (error: any) {
    notifyError('删除主题失败', error);
  }
};

const activeThemeName = computed(() => {
  const activeId = activeTerminalThemeId.value;
  if (activeId === null || activeId === undefined) {
    return '未选择主题';
  }
  const theme = allTerminalThemes.value.find(item => item._id === String(activeId));
  return theme ? theme.name : '未知主题';
});

const filteredAndSortedThemes = computed(() => {
  const searchTerm = themeSearchTerm.value.toLowerCase().trim();
  const result = [...allTerminalThemes.value]
    .filter(theme => !searchTerm || theme.name.toLowerCase().includes(searchTerm))
    .sort((a, b) => a.name.localeCompare(b.name));
  return result;
});

const formatXtermLabel = (key: keyof ITheme): string => {
  return key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase());
};

const handleFocusAndSelect = (event: FocusEvent) => {
  const target = event.target;
  if (target instanceof HTMLInputElement) {
    target.select();
  }
};

const patchEditingTheme = (updater: (theme: TerminalTheme) => TerminalTheme) => {
  if (!props.editingTheme) {
    return;
  }
  emit('update:editingTheme', updater(props.editingTheme));
};

const updateEditingThemeName = (name: string) => {
  patchEditingTheme(theme => ({ ...theme, name }));
};

const updateEditingThemeField = (key: keyof ITheme, value: string) => {
  patchEditingTheme(theme => ({
    ...theme,
    themeData: {
      ...theme.themeData,
      [key]: value,
    },
  }));
};

watch(
  () => props.editingTheme?.themeData,
  (newThemeData) => {
    if (!newThemeData) {
      return;
    }
    if (document.activeElement?.id === 'terminalThemeTextarea' && !terminalThemeParseError.value) {
      return;
    }
    const newStringValue = Object.entries(newThemeData)
      .map(([key, value]) => `${key}: ${value}`)
      .join('\n');
    editableTerminalThemeString.value = newStringValue;
    if (document.activeElement?.id !== 'terminalThemeTextarea') {
      terminalThemeParseError.value = null;
    }
  },
  { deep: true },
);

watch(
  () => props.isEditingTheme,
  isEditing => {
    if (isEditing && props.editingTheme) {
      editableTerminalThemeString.value = Object.entries(props.editingTheme.themeData)
        .map(([key, value]) => `${key}: ${value}`)
        .join('\n');
      terminalThemeParseError.value = null;
      return;
    }

    editableTerminalThemeString.value = '';
    terminalThemeParseError.value = null;
    saveThemeError.value = null;
    initializeEditableState();
  },
  { immediate: true },
);
</script>

<template>
  <section v-if="!isEditingTheme" class="terminal-tab">
    <h3 class="section-title">终端样式</h3>

    <div class="field-grid three-col">
      <label for="terminalFontFamily" class="field-label">终端字体:</label>
      <input
        id="terminalFontFamily"
        v-model="editableTerminalFontFamily"
        type="text"
        class="field-input"
        placeholder="Consolas, 'Courier New', monospace"
      />
      <button class="save-btn" @click="handleSaveTerminalFont">保存</button>
    </div>

    <p class="hint-text">输入字体名称，使用英文逗号分隔。如果字体名称包含空格，请用引号括起来。</p>

    <div class="field-grid three-col">
      <label for="terminalFontSize" class="field-label">终端字体大小:</label>
      <input
        id="terminalFontSize"
        v-model.number="editableTerminalFontSize"
        type="number"
        min="1"
        class="field-input number-input"
      />
      <button class="save-btn" @click="handleSaveTerminalFontSize">保存</button>
    </div>

    <hr class="divider" />

    <h4 class="group-title">文字描边设置</h4>
    <div class="section-body">
      <div class="toggle-row">
        <label for="terminalTextStrokeEnabledSwitch" class="field-label">启用文字描边</label>
        <button
          id="terminalTextStrokeEnabledSwitch"
          type="button"
          class="switch-btn"
          :class="{ active: editableTerminalTextStrokeEnabled }"
          :aria-checked="editableTerminalTextStrokeEnabled"
          role="switch"
          @click="toggleTextStrokeAndSave"
        >
          <span class="switch-knob" :class="{ active: editableTerminalTextStrokeEnabled }" />
        </button>
      </div>

      <div class="field-grid two-col">
        <label for="terminalTextStrokeWidth" class="field-label">描边粗细 (px):</label>
        <input
          id="terminalTextStrokeWidth"
          v-model.number="editableTerminalTextStrokeWidth"
          type="number"
          min="0"
          step="0.1"
          class="field-input number-input"
        />
      </div>

      <div class="field-grid two-col">
        <label for="terminalTextStrokeColor" class="field-label">描边颜色:</label>
        <div class="color-wrap">
          <input
            id="terminalTextStrokeColor"
            v-model.lazy="editableTerminalTextStrokeColor"
            type="color"
            class="color-input"
          />
          <input
            :value="editableTerminalTextStrokeColor"
            class="field-input"
            @input="editableTerminalTextStrokeColor = ($event.target as HTMLInputElement).value"
          />
        </div>
      </div>

      <button class="save-btn inline" @click="handleSaveTerminalTextStroke">保存</button>
    </div>

    <hr class="divider" />

    <h4 class="group-title">文字阴影设置</h4>
    <div class="section-body">
      <div class="toggle-row">
        <label for="terminalTextShadowEnabledSwitch" class="field-label">启用文字阴影</label>
        <button
          id="terminalTextShadowEnabledSwitch"
          type="button"
          class="switch-btn"
          :class="{ active: editableTerminalTextShadowEnabled }"
          :aria-checked="editableTerminalTextShadowEnabled"
          role="switch"
          @click="toggleTextShadowAndSave"
        >
          <span class="switch-knob" :class="{ active: editableTerminalTextShadowEnabled }" />
        </button>
      </div>

      <div class="field-grid two-col">
        <label for="terminalTextShadowOffsetX" class="field-label">阴影 X 偏移 (px):</label>
        <input
          id="terminalTextShadowOffsetX"
          v-model.number="editableTerminalTextShadowOffsetX"
          type="number"
          step="0.1"
          class="field-input number-input"
        />
      </div>

      <div class="field-grid two-col">
        <label for="terminalTextShadowOffsetY" class="field-label">阴影 Y 偏移 (px):</label>
        <input
          id="terminalTextShadowOffsetY"
          v-model.number="editableTerminalTextShadowOffsetY"
          type="number"
          step="0.1"
          class="field-input number-input"
        />
      </div>

      <div class="field-grid two-col">
        <label for="terminalTextShadowBlur" class="field-label">阴影模糊 (px):</label>
        <input
          id="terminalTextShadowBlur"
          v-model.number="editableTerminalTextShadowBlur"
          type="number"
          min="0"
          step="0.1"
          class="field-input number-input"
        />
      </div>

      <div class="field-grid two-col">
        <label for="terminalTextShadowColor" class="field-label">阴影颜色:</label>
        <div class="color-wrap">
          <input
            id="terminalTextShadowColor"
            v-model.lazy="editableTerminalTextShadowColor"
            type="color"
            class="color-input"
          />
          <input
            :value="editableTerminalTextShadowColor"
            class="field-input"
            @input="editableTerminalTextShadowColor = ($event.target as HTMLInputElement).value"
          />
        </div>
      </div>

      <button class="save-btn inline" @click="handleSaveTerminalTextShadow">保存</button>
    </div>

    <hr class="divider" />

    <h4 class="group-title">终端主题选择</h4>
    <p class="active-theme-row">当前主题: <strong>{{ activeThemeName }}</strong></p>

    <div class="theme-actions">
      <button class="save-btn" @click="handleAddNewTheme">新增主题</button>
    </div>

    <input
      v-model="themeSearchTerm"
      type="text"
      class="field-input search-input"
      placeholder="搜索主题名称..."
    />

    <ul class="theme-list">
      <li v-if="filteredAndSortedThemes.length === 0" class="theme-empty">未找到匹配主题</li>
      <li
        v-for="theme in filteredAndSortedThemes"
        v-else
        :key="theme._id"
        class="theme-item"
        :class="{ active: theme._id === String(activeTerminalThemeId) }"
      >
        <span class="theme-name" :title="theme.name">{{ theme.name }}</span>
        <div class="theme-item-actions">
          <button class="mini-btn" :disabled="theme._id === String(activeTerminalThemeId)" @click="handleApplyTheme(theme)">应用</button>
          <button class="mini-btn" @click="handleEditTheme(theme)">编辑</button>
          <button class="mini-btn danger" :disabled="theme.isPreset" @click="handleDeleteTheme(theme)">删除</button>
        </div>
      </li>
    </ul>
  </section>

  <section v-else-if="editingTheme" class="terminal-tab">
    <h3 class="section-title">{{ editingTheme._id ? '编辑终端主题' : '新建终端主题' }}</h3>

    <p v-if="saveThemeError" class="error-message">{{ saveThemeError }}</p>

    <div class="field-grid two-col">
      <label for="editingThemeName" class="field-label">主题名称:</label>
      <input
        id="editingThemeName"
        :value="editingTheme.name"
        required
        class="field-input"
        @input="updateEditingThemeName(($event.target as HTMLInputElement).value)"
      />
    </div>

    <hr class="divider" />

    <h4 class="group-title">终端主题颜色编辑器</h4>
    <div v-for="(value, key) in editingTheme.themeData" :key="key" class="field-grid two-col">
      <label :for="`xterm-${key}`" class="field-label">{{ formatXtermLabel(key as keyof ITheme) }}:</label>
      <div class="color-wrap">
        <input
          v-if="typeof value === 'string' && value.startsWith('#')"
          :id="`xterm-${key}`"
          :value="value"
          type="color"
          class="color-input"
          @input="updateEditingThemeField(key as keyof ITheme, ($event.target as HTMLInputElement).value)"
        />
        <input
          v-if="typeof value === 'string' && value.startsWith('#')"
          :value="value"
          readonly
          class="field-input"
          @focus="handleFocusAndSelect"
        />
        <input
          v-else
          :id="`xterm-${key}`"
          :value="value as string"
          class="field-input"
          @input="updateEditingThemeField(key as keyof ITheme, ($event.target as HTMLInputElement).value)"
        />
      </div>
    </div>

    <hr class="divider" />

    <h4 class="group-title">终端主题 JSON 编辑器</h4>
    <p class="hint-text">支持按 `变量: 值` 的形式批量编辑终端主题。</p>

    <textarea
      id="terminalThemeTextarea"
      v-model="editableTerminalThemeString"
      rows="10"
      spellcheck="false"
      class="json-textarea"
      :placeholder="terminalThemePlaceholder"
      @blur="handleTerminalThemeStringChange"
    />

    <p v-if="terminalThemeParseError" class="error-message">{{ terminalThemeParseError }}</p>

    <div class="footer-actions">
      <button class="save-btn" @click="handleCancelEditingTheme">取消</button>
      <button class="save-btn primary" @click="handleSaveEditingTheme">保存</button>
    </div>
  </section>
</template>

<style scoped>
.terminal-tab {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-right: 6px;
}

.section-title {
  margin: 0;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border);
  font-size: 19px;
  line-height: 1.2;
  color: var(--text);
  font-weight: 700;
}

.group-title {
  margin: 0;
  font-size: 16px;
  line-height: 1;
  color: var(--text);
  font-weight: 600;
}

.section-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field-grid {
  display: grid;
  gap: 10px;
  align-items: center;
}

.field-grid.three-col {
  grid-template-columns: 130px minmax(0, 1fr) auto;
}

.field-grid.two-col {
  grid-template-columns: 130px minmax(0, 1fr);
}

.field-label {
  font-size: 13px;
  color: var(--text);
  font-weight: 600;
}

.field-input {
  height: 34px;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
  background: color-mix(in srgb, var(--app-bg-color) 76%, var(--header-bg-color) 24%);
  color: var(--text);
  padding: 0 10px;
  font-size: 13px;
  min-width: 0;
}

.field-input:focus {
  outline: none;
  border-color: var(--input-focus-border-color);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--input-focus-border-color) 35%, transparent);
}

.number-input {
  max-width: 170px;
}

.hint-text {
  margin: -4px 0 0;
  font-size: 12px;
  color: var(--text-sub);
  line-height: 1.5;
}

.save-btn {
  height: 34px;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
  background: color-mix(in srgb, var(--header-bg-color) 86%, transparent);
  color: var(--text);
  padding: 0 14px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.save-btn:hover {
  background: color-mix(in srgb, var(--header-bg-color) 70%, var(--bg-surface1) 30%);
}

.save-btn.inline {
  width: fit-content;
}

.save-btn.primary {
  background: var(--button-bg-color);
  border-color: var(--button-bg-color);
  color: var(--button-text-color);
}

.save-btn.primary:hover {
  background: var(--button-hover-bg-color);
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 34px;
}

.switch-btn {
  width: 44px;
  height: 24px;
  border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  border-radius: 999px;
  background: #666d79;
  position: relative;
  cursor: pointer;
  padding: 0;
}

.switch-btn.active {
  background: var(--link-active-color);
}

.switch-knob {
  position: absolute;
  left: 2px;
  top: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #ffffff;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.35);
  transition: transform 0.18s ease;
}

.switch-knob.active {
  transform: translateX(20px);
}

.color-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.color-wrap .field-input {
  flex: 1;
}

.color-input {
  width: 44px;
  height: 32px;
  border-radius: 4px;
  border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
  padding: 2px;
  background: transparent;
}

.divider {
  border: none;
  border-top: 1px solid color-mix(in srgb, var(--border) 75%, transparent);
  margin: 8px 0 4px;
}

.active-theme-row {
  margin: 0;
  font-size: 13px;
  color: var(--text-sub);
}

.active-theme-row strong {
  color: var(--text);
}

.theme-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.search-input {
  margin-top: 2px;
}

.theme-list {
  list-style: none;
  margin: 0;
  padding: 0;
  border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
  border-radius: 8px;
  overflow: auto;
  max-height: 280px;
  background: color-mix(in srgb, var(--app-bg-color) 80%, var(--header-bg-color) 20%);
}

.theme-empty {
  padding: 12px;
  text-align: center;
  color: var(--text-sub);
  font-size: 12px;
}

.theme-item {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 10px;
  align-items: center;
  padding: 10px 12px;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 75%, transparent);
}

.theme-item:last-child {
  border-bottom: none;
}

.theme-item:hover {
  background: color-mix(in srgb, var(--header-bg-color) 65%, transparent);
}

.theme-item.active {
  background: var(--button-bg-color);
}

.theme-name {
  font-size: 13px;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.theme-item.active .theme-name {
  color: var(--button-text-color);
  font-weight: 600;
}

.theme-item-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.mini-btn {
  height: 28px;
  border-radius: 5px;
  border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
  background: color-mix(in srgb, var(--header-bg-color) 85%, transparent);
  color: var(--text);
  padding: 0 10px;
  font-size: 12px;
  cursor: pointer;
}

.mini-btn:hover {
  background: color-mix(in srgb, var(--header-bg-color) 70%, var(--bg-surface1) 30%);
}

.mini-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.mini-btn.danger {
  border-color: color-mix(in srgb, var(--color-error) 70%, transparent);
  color: var(--color-error);
  background: color-mix(in srgb, var(--color-error) 14%, transparent);
}

.theme-item.active .mini-btn {
  border-color: rgba(255, 255, 255, 0.35);
  color: var(--button-text-color);
  background: rgba(255, 255, 255, 0.12);
}

.json-textarea {
  width: 100%;
  min-height: 180px;
  resize: vertical;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--border) 85%, transparent);
  background: color-mix(in srgb, var(--app-bg-color) 76%, var(--header-bg-color) 24%);
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

.footer-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

@media (max-width: 860px) {
  .field-grid.three-col,
  .field-grid.two-col,
  .theme-item {
    grid-template-columns: 1fr;
  }

  .number-input {
    max-width: 100%;
  }

  .theme-item-actions {
    justify-content: flex-start;
  }
}
</style>