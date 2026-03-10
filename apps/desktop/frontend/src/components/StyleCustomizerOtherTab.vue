<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useAppearanceStore } from '@/stores/appearance';
import { useUiNotificationsStore } from '@/stores/uiNotifications';

const appearanceStore = useAppearanceStore();
const notificationsStore = useUiNotificationsStore();

const {
  appearanceSettings,
  currentEditorFontSize,
  currentEditorFontFamily,
} = storeToRefs(appearanceStore);

const editableEditorFontSize = ref(16);
const editableEditorFontFamily = ref('');

const initializeEditableState = () => {
  editableEditorFontSize.value = currentEditorFontSize.value;
  editableEditorFontFamily.value = currentEditorFontFamily.value;
};

onMounted(initializeEditableState);

watch(
  () => appearanceSettings.value,
  (newSettings, oldSettings) => {
    const fontSizeChanged = newSettings?.editorFontSize !== oldSettings?.editorFontSize;
    const fontFamilyChanged = newSettings?.editorFontFamily !== oldSettings?.editorFontFamily;

    if (fontSizeChanged) {
      editableEditorFontSize.value = newSettings?.editorFontSize || 16;
    }
    if (fontFamilyChanged) {
      editableEditorFontFamily.value = newSettings?.editorFontFamily || `Consolas, 'Noto Sans SC', 'Microsoft YaHei'`;
    }
  },
  { deep: true },
);

const handleSaveEditorFontSize = async () => {
  try {
    const size = Number(editableEditorFontSize.value);
    if (Number.isNaN(size) || size <= 0) {
      notificationsStore.addNotification({ type: 'error', message: '编辑器字号必须大于 0' });
      return;
    }
    await appearanceStore.setEditorFontSize(size);
    notificationsStore.addNotification({ type: 'success', message: '编辑器字体大小已保存' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '保存编辑器字体大小失败' });
  }
};

const handleSaveEditorFontFamily = async () => {
  try {
    await appearanceStore.setEditorFontFamily(editableEditorFontFamily.value);
    notificationsStore.addNotification({ type: 'success', message: '编辑器字体已保存' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '保存编辑器字体失败' });
  }
};
</script>

<template>
  <section class="other-tab">
    <h3 class="section-title">其他设置</h3>

    <div class="field-grid">
      <label for="editorFontSizeOther" class="field-label">编辑器字体大小:</label>
      <input
        id="editorFontSizeOther"
        v-model.number="editableEditorFontSize"
        type="number"
        min="1"
        class="field-input number-input"
      />
      <button class="save-btn" @click="handleSaveEditorFontSize">保存</button>
    </div>

    <div class="field-grid">
      <label for="editorFontFamilyOther" class="field-label">编辑器字体:</label>
      <input
        id="editorFontFamilyOther"
        v-model="editableEditorFontFamily"
        type="text"
        class="field-input"
      />
      <button class="save-btn" @click="handleSaveEditorFontFamily">保存</button>
    </div>
  </section>
</template>

<style scoped>
.other-tab {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  margin: 0;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border);
  font-size: calc(19px + var(--ui-font-size-offset));
  line-height: 1.2;
  color: var(--text);
  font-weight: 600;
}

.field-grid {
  display: grid;
  grid-template-columns: 160px 1fr auto;
  align-items: center;
  gap: 10px;
}

.field-label {
  color: var(--text);
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 500;
}

.field-input {
  height: 34px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--app-bg-color);
  color: var(--text);
  padding: 0 10px;
  font-size: calc(13px + var(--ui-font-size-offset));
}

.field-input:focus {
  outline: none;
  border-color: var(--input-focus-border-color);
}

.number-input {
  max-width: 120px;
}

.save-btn {
  height: 34px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--header-bg-color);
  color: var(--text);
  padding: 0 14px;
  font-size: calc(13px + var(--ui-font-size-offset));
  cursor: pointer;
}

.save-btn:hover {
  background: var(--bg-surface1);
}

@media (max-width: 860px) {
  .field-grid {
    grid-template-columns: 1fr;
  }

  .number-input {
    max-width: 100%;
  }
}
</style>
