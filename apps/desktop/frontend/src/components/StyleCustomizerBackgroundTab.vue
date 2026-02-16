<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { useAppearanceStore } from '@/stores/appearance';
import { useUiNotificationsStore } from '@/stores/uiNotifications';

const { confirm } = useConfirmDialog();
const appearanceStore = useAppearanceStore();
const notificationsStore = useUiNotificationsStore();

const {
  terminalBackgroundImage,
  isTerminalBackgroundEnabled,
  currentTerminalBackgroundOverlayOpacity,
  localHtmlPresets,
  remoteHtmlPresets,
  remoteHtmlPresetsRepositoryUrl,
  activeHtmlPresetTab,
  isLoadingHtmlPresets,
  htmlPresetError,
} = storeToRefs(appearanceStore);

const {
  fetchLocalHtmlPresets,
  getLocalHtmlPresetContent,
  createLocalHtmlPreset,
  updateLocalHtmlPreset,
  deleteLocalHtmlPreset,
  fetchRemoteHtmlPresetsRepositoryUrl,
  updateRemoteHtmlPresetsRepositoryUrl,
  fetchRemoteHtmlPresets,
  getRemoteHtmlPresetContent,
  applyHtmlPreset,
} = appearanceStore;

const localTerminalBackgroundEnabled = ref(true);
const editableTerminalBackgroundOverlayOpacity = ref(0.5);
const terminalBgFileInput = ref<HTMLInputElement | null>(null);
const uploadError = ref<string | null>(null);

const currentActiveTab = ref<'local' | 'remote'>('local');

const showPresetEditor = ref(false);
const editingPreset = ref<{ name: string; content: string } | null>(null);
const newPresetName = ref('');
const newPresetContent = ref('');

const localRemoteHtmlPresetsRepositoryUrl = ref('');
const localHtmlSearchTerm = ref('');
const remoteHtmlSearchTerm = ref('');

const localSpecificLoading = ref(false);
const remoteSpecificLoading = ref(false);

const initializeEditableState = () => {
  localTerminalBackgroundEnabled.value = isTerminalBackgroundEnabled.value;
  editableTerminalBackgroundOverlayOpacity.value = currentTerminalBackgroundOverlayOpacity.value;
  uploadError.value = null;
  currentActiveTab.value = activeHtmlPresetTab.value;
  localRemoteHtmlPresetsRepositoryUrl.value = remoteHtmlPresetsRepositoryUrl.value || 'https://github.com/Heavrnl/nexus-terminal/tree/main/doc/custom_html_theme';
};

onMounted(async () => {
  initializeEditableState();
  localSpecificLoading.value = true;
  try {
    await fetchLocalHtmlPresets();
  } finally {
    localSpecificLoading.value = false;
  }

  if (!remoteHtmlPresetsRepositoryUrl.value) {
    await fetchRemoteHtmlPresetsRepositoryUrl();
  }

  if (remoteHtmlPresetsRepositoryUrl.value) {
    remoteSpecificLoading.value = true;
    try {
      await fetchRemoteHtmlPresets();
    } finally {
      remoteSpecificLoading.value = false;
    }
  }
});

watch(isTerminalBackgroundEnabled, value => {
  if (localTerminalBackgroundEnabled.value !== value) {
    localTerminalBackgroundEnabled.value = value;
  }
});

watch(currentTerminalBackgroundOverlayOpacity, value => {
  if (editableTerminalBackgroundOverlayOpacity.value !== value) {
    editableTerminalBackgroundOverlayOpacity.value = value;
  }
});

watch(activeHtmlPresetTab, value => {
  currentActiveTab.value = value;
});

watch(remoteHtmlPresetsRepositoryUrl, value => {
  localRemoteHtmlPresetsRepositoryUrl.value = value || 'https://github.com/Heavrnl/nexus-terminal/tree/main/doc/custom_html_theme';
});

const switchTab = (tab: 'local' | 'remote') => {
  appearanceStore.activeHtmlPresetTab = tab;
};

const handleTriggerTerminalBgUpload = () => {
  uploadError.value = null;
  terminalBgFileInput.value?.click();
};

const handleTerminalBgUpload = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  if (input.files && input.files[0]) {
    const file = input.files[0];
    try {
      await appearanceStore.uploadTerminalBackground(file);
      notificationsStore.addNotification({ type: 'success', message: '终端背景上传成功' });
      input.value = '';
    } catch (error: any) {
      const message = error?.message ?? '上传失败';
      uploadError.value = message;
      notificationsStore.addNotification({ type: 'error', message });
      input.value = '';
    }
  }
};

const handleRemoveTerminalBg = async () => {
  try {
    await appearanceStore.removeTerminalBackground();
    notificationsStore.addNotification({ type: 'success', message: '终端背景已移除' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '移除终端背景失败' });
  }
};

const handleToggleTerminalBackground = async () => {
  const newValue = !localTerminalBackgroundEnabled.value;
  localTerminalBackgroundEnabled.value = newValue;
  try {
    await appearanceStore.setTerminalBackgroundEnabled(newValue);
  } catch (error: any) {
    localTerminalBackgroundEnabled.value = !newValue;
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '更新终端背景状态失败' });
  }
};

const handleSaveTerminalBackgroundOverlayOpacity = async () => {
  try {
    const opacity = Number(editableTerminalBackgroundOverlayOpacity.value);
    if (Number.isNaN(opacity) || opacity < 0 || opacity > 1) {
      notificationsStore.addNotification({ type: 'error', message: '透明度范围必须在 0 到 1 之间' });
      return;
    }
    await appearanceStore.setTerminalBackgroundOverlayOpacity(opacity);
    notificationsStore.addNotification({ type: 'success', message: '终端背景蒙版透明度已保存' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '保存透明度失败' });
  }
};

const handleApplyPreset = async (htmlContent: string) => {
  try {
    await applyHtmlPreset(htmlContent);
    notificationsStore.addNotification({ type: 'success', message: 'HTML 背景主题已应用' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '应用 HTML 背景主题失败' });
  }
};

const handleResetCustomHtml = async () => {
  try {
    await applyHtmlPreset('');
    notificationsStore.addNotification({ type: 'success', message: '自定义 HTML 已重置' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '重置自定义 HTML 失败' });
  }
};

const openNewPresetEditor = () => {
  editingPreset.value = null;
  newPresetName.value = '';
  newPresetContent.value = '';
  showPresetEditor.value = true;
};

const openEditPresetEditor = (preset: { name: string; content: string }) => {
  editingPreset.value = { ...preset };
  newPresetName.value = preset.name.replace(/\.html$/, '');
  newPresetContent.value = preset.content;
  showPresetEditor.value = true;
};

const handleEditCustomPreset = async (name: string) => {
  try {
    const content = await getLocalHtmlPresetContent(name);
    openEditPresetEditor({ name, content });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '加载主题内容失败' });
  }
};

const handleEditPresetAsNew = async (preset: { name: string; type: 'preset' | 'custom' }) => {
  try {
    const content = await getLocalHtmlPresetContent(preset.name);
    editingPreset.value = null;
    newPresetName.value = `${preset.name.replace(/\.html$/, '')}(1)`;
    newPresetContent.value = content;
    showPresetEditor.value = true;
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '获取预设内容失败' });
  }
};

const handleSaveLocalPreset = async () => {
  const desiredBaseName = newPresetName.value.trim();
  const content = newPresetContent.value.trim();

  if (!desiredBaseName) {
    notificationsStore.addNotification({ type: 'error', message: '预设名称不能为空' });
    return;
  }
  if (!content) {
    notificationsStore.addNotification({ type: 'error', message: '预设内容不能为空' });
    return;
  }

  const finalName = desiredBaseName.endsWith('.html') ? desiredBaseName : `${desiredBaseName}.html`;

  if (editingPreset.value) {
    const originalName = editingPreset.value.name;
    if (finalName === originalName) {
      try {
        await updateLocalHtmlPreset(originalName, content);
        notificationsStore.addNotification({ type: 'success', message: '本地主题已更新' });
        showPresetEditor.value = false;
      } catch (error: any) {
        notificationsStore.addNotification({ type: 'error', message: error?.message ?? '更新本地主题失败' });
      }
    } else {
      try {
        await createLocalHtmlPreset(finalName, content);
        await deleteLocalHtmlPreset(originalName);
        notificationsStore.addNotification({
          type: 'success',
          message: `主题已重命名为：${desiredBaseName}`,
        });
        showPresetEditor.value = false;
      } catch (error: any) {
        notificationsStore.addNotification({ type: 'error', message: error?.message ?? '重命名主题失败' });
      }
    }
  } else {
    try {
      await createLocalHtmlPreset(finalName, content);
      notificationsStore.addNotification({ type: 'success', message: '本地主题已创建' });
      showPresetEditor.value = false;
      newPresetName.value = '';
      newPresetContent.value = '';
    } catch (error: any) {
      notificationsStore.addNotification({ type: 'error', message: error?.message ?? '创建本地主题失败' });
    }
  }
};

const handleDeleteLocalPreset = async (name: string) => {
  const displayName = name.replace(/\.html$/, '');
  const confirmed = await confirm('删除主题', `确认删除本地主题“${displayName}”吗？`);
  if (!confirmed) {
    return;
  }

  try {
    await deleteLocalHtmlPreset(name);
    notificationsStore.addNotification({ type: 'success', message: '本地主题已删除' });
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '删除本地主题失败' });
  }
};

const handleSaveRemoteRepositoryUrl = async () => {
  try {
    await updateRemoteHtmlPresetsRepositoryUrl(localRemoteHtmlPresetsRepositoryUrl.value);
    notificationsStore.addNotification({ type: 'success', message: '远程仓库地址已保存' });
    if (localRemoteHtmlPresetsRepositoryUrl.value) {
      remoteSpecificLoading.value = true;
      try {
        await fetchRemoteHtmlPresets(localRemoteHtmlPresetsRepositoryUrl.value);
      } finally {
        remoteSpecificLoading.value = false;
      }
    }
  } catch (error: any) {
    remoteSpecificLoading.value = false;
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '保存远程仓库地址失败' });
  }
};

const handleLoadRemotePresets = async () => {
  if (!remoteHtmlPresetsRepositoryUrl.value) {
    notificationsStore.addNotification({ type: 'error', message: '请先设置远程仓库地址' });
    return;
  }
  remoteSpecificLoading.value = true;
  try {
    await fetchRemoteHtmlPresets();
    if (!htmlPresetError.value) {
      notificationsStore.addNotification({ type: 'success', message: '远程主题已加载' });
    }
  } finally {
    remoteSpecificLoading.value = false;
  }
};

const applyLocalPreset = async (presetName: string) => {
  try {
    const content = await getLocalHtmlPresetContent(presetName);
    await handleApplyPreset(content);
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '应用本地主题失败' });
  }
};

const applyRemotePreset = async (downloadUrl?: string) => {
  if (!downloadUrl) {
    notificationsStore.addNotification({ type: 'error', message: '远程主题缺少下载地址' });
    return;
  }
  try {
    const content = await getRemoteHtmlPresetContent(downloadUrl);
    await handleApplyPreset(content);
  } catch (error: any) {
    notificationsStore.addNotification({ type: 'error', message: error?.message ?? '应用远程主题失败' });
  }
};

const filteredLocalHtmlPresets = computed(() => {
  const searchTerm = localHtmlSearchTerm.value.toLowerCase().trim();
  return [...localHtmlPresets.value]
    .filter(preset => !searchTerm || preset.name.replace(/\.html$/, '').toLowerCase().includes(searchTerm))
    .sort((a, b) => a.name.localeCompare(b.name));
});

const filteredRemoteHtmlPresets = computed(() => {
  const searchTerm = remoteHtmlSearchTerm.value.toLowerCase().trim();
  return [...remoteHtmlPresets.value]
    .filter(preset => !searchTerm || preset.name.replace(/\.html$/, '').toLowerCase().includes(searchTerm))
    .sort((a, b) => a.name.localeCompare(b.name));
});
</script>

<template>
  <section class="background-tab">
    <h3 class="section-title">背景设置</h3>

    <hr class="divider" />

    <div class="toggle-row">
      <h4 class="group-title">终端背景</h4>
      <button
        type="button"
        class="switch-btn"
        :class="{ active: localTerminalBackgroundEnabled }"
        :aria-checked="localTerminalBackgroundEnabled"
        @click="handleToggleTerminalBackground"
      >
        <span class="switch-knob" :class="{ active: localTerminalBackgroundEnabled }" />
      </button>
    </div>

    <div v-if="localTerminalBackgroundEnabled" class="background-config-panel">
      <div
        class="preview-box"
        :style="{ backgroundImage: terminalBackgroundImage ? `url(${terminalBackgroundImage})` : 'none' }"
      >
        <div
          v-if="terminalBackgroundImage"
          class="preview-overlay"
          :style="{ backgroundColor: `rgba(0, 0, 0, ${editableTerminalBackgroundOverlayOpacity})` }"
        />
        <span v-if="!terminalBackgroundImage" class="preview-empty">暂无背景</span>
      </div>

      <div class="button-row">
        <button class="save-btn" @click="handleTriggerTerminalBgUpload">上传终端背景</button>
        <button class="save-btn danger" :disabled="!terminalBackgroundImage" @click="handleRemoveTerminalBg">移除终端背景</button>
        <input ref="terminalBgFileInput" type="file" accept="image/*" class="hidden-input" @change="handleTerminalBgUpload" />
      </div>

      <p v-if="uploadError" class="error-message">{{ uploadError }}</p>

      <div class="opacity-row">
        <label for="terminalBgOverlayOpacity" class="field-label">终端背景蒙版透明度:</label>
        <div class="opacity-control">
          <input
            id="terminalBgOverlayOpacity"
            v-model.number="editableTerminalBackgroundOverlayOpacity"
            type="range"
            min="0"
            max="1"
            step="0.01"
            class="opacity-slider"
          />
          <span class="opacity-value">{{ editableTerminalBackgroundOverlayOpacity.toFixed(2) }}</span>
          <button class="save-btn" @click="handleSaveTerminalBackgroundOverlayOpacity">保存</button>
        </div>
      </div>
    </div>
    <div v-else class="disabled-tip">终端背景功能已禁用。</div>

    <hr class="divider" />

    <div v-if="localTerminalBackgroundEnabled" class="html-theme-panel">
      <div class="title-row">
        <h4 class="group-title">HTML 背景主题</h4>
        <button class="icon-btn" title="重置 HTML 主题" @click="handleResetCustomHtml">
          <i class="fa-solid fa-rotate-left" />
        </button>
      </div>

      <div class="tab-row">
        <button class="tab-btn" :class="{ active: currentActiveTab === 'local' }" @click="switchTab('local')">本地主题</button>
        <button class="tab-btn" :class="{ active: currentActiveTab === 'remote' }" @click="switchTab('remote')">远程主题</button>
      </div>

      <div v-if="currentActiveTab === 'local'" class="tab-panel">
        <div class="toolbar-row">
          <input v-model="localHtmlSearchTerm" class="field-input" placeholder="搜索本地主题..." />
          <button class="save-btn" @click="openNewPresetEditor">新增主题</button>
        </div>

        <div v-if="localSpecificLoading" class="loading-tip">加载中...</div>

        <ul v-else-if="filteredLocalHtmlPresets.length > 0" class="preset-list">
          <li v-for="preset in filteredLocalHtmlPresets" :key="preset.name" class="preset-item">
            <div class="preset-name-wrap">
              <span class="preset-name" :title="preset.name.replace(/\.html$/, '')">{{ preset.name.replace(/\.html$/, '') }}</span>
              <span v-if="preset.type === 'preset'" class="preset-tag preset">预设</span>
              <span v-else class="preset-tag custom">自定义</span>
            </div>

            <div class="preset-actions">
              <button class="mini-btn" @click="applyLocalPreset(preset.name)">应用</button>
              <button v-if="preset.type === 'custom'" class="mini-btn" @click="handleEditCustomPreset(preset.name)">编辑</button>
              <button v-else class="mini-btn" @click="handleEditPresetAsNew(preset)">编辑</button>
              <button v-if="preset.type === 'custom'" class="mini-btn danger" @click="handleDeleteLocalPreset(preset.name)">删除</button>
            </div>
          </li>
        </ul>

        <div v-else-if="htmlPresetError" class="error-message">{{ htmlPresetError }}</div>
        <div v-else class="empty-tip">{{ localHtmlSearchTerm ? '未找到匹配的本地主题' : '暂无本地主题' }}</div>
      </div>

      <div v-if="currentActiveTab === 'remote'" class="tab-panel">
        <div class="remote-url-row">
          <label for="remoteRepoUrl" class="field-label">远程主题仓库地址:</label>
          <div class="remote-url-input-row">
            <input
              id="remoteRepoUrl"
              v-model="localRemoteHtmlPresetsRepositoryUrl"
              class="field-input"
              placeholder="https://github.com/Heavrnl/nexus-terminal/tree/main/doc/custom_html_theme"
            />
            <button class="save-btn" @click="handleSaveRemoteRepositoryUrl">保存</button>
            <button class="save-btn" :disabled="!remoteHtmlPresetsRepositoryUrl || remoteSpecificLoading" @click="handleLoadRemotePresets">
              {{ remoteSpecificLoading ? '加载中...' : '加载主题' }}
            </button>
          </div>
        </div>

        <input v-model="remoteHtmlSearchTerm" class="field-input" placeholder="搜索远程主题..." />

        <div v-if="(remoteSpecificLoading || isLoadingHtmlPresets) && currentActiveTab === 'remote'" class="loading-tip">加载中...</div>
        <div v-else-if="htmlPresetError && currentActiveTab === 'remote'" class="error-message">{{ htmlPresetError }}</div>
        <div v-else-if="!remoteHtmlPresetsRepositoryUrl" class="empty-tip">请先设置远程仓库地址</div>

        <ul v-else-if="filteredRemoteHtmlPresets.length > 0" class="preset-list">
          <li v-for="preset in filteredRemoteHtmlPresets" :key="preset.name" class="preset-item">
            <span class="preset-name" :title="preset.name.replace(/\.html$/, '')">{{ preset.name.replace(/\.html$/, '') }}</span>
            <div class="preset-actions">
              <button class="mini-btn" :disabled="!preset.downloadUrl" @click="applyRemotePreset(preset.downloadUrl)">应用</button>
            </div>
          </li>
        </ul>
        <div v-else class="empty-tip">{{ remoteHtmlSearchTerm ? '未找到匹配的远程主题' : '暂无远程主题' }}</div>
      </div>
    </div>

    <div v-if="showPresetEditor" class="editor-mask" @click.self="showPresetEditor = false">
      <div class="editor-dialog">
        <div class="editor-field">
          <label for="presetName" class="field-label">主题名称</label>
          <input id="presetName" v-model="newPresetName" class="field-input" placeholder="my-theme" />
        </div>
        <div class="editor-field">
          <label for="presetContent" class="field-label">主题内容</label>
          <textarea id="presetContent" v-model="newPresetContent" rows="10" class="editor-textarea" placeholder="例如：<h1>Hello</h1>" />
        </div>
        <div class="editor-actions">
          <button class="save-btn" @click="showPresetEditor = false">取消</button>
          <button class="save-btn primary" @click="handleSaveLocalPreset">保存</button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.background-tab {
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

.group-title {
  margin: 0;
  font-size: 15px;
  color: var(--text);
  font-weight: 600;
}

.field-label {
  font-size: 13px;
  color: var(--text);
  font-weight: 500;
}

.field-input {
  height: 34px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--app-bg-color);
  color: var(--text);
  padding: 0 10px;
  font-size: 13px;
  min-width: 0;
}

.field-input:focus,
.editor-textarea:focus {
  outline: none;
  border-color: var(--input-focus-border-color);
}

.divider {
  border: none;
  border-top: 1px solid var(--border);
  margin: 6px 0;
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.switch-btn {
  width: 44px;
  height: 24px;
  border: none;
  border-radius: 999px;
  background: #7c8794;
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
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #ffffff;
  transition: transform 0.18s ease;
}

.switch-knob.active {
  transform: translateX(20px);
}

.background-config-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.preview-box {
  height: 145px;
  border: 1px dashed var(--border);
  border-radius: 8px;
  position: relative;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  background-color: var(--header-bg-color);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-overlay {
  position: absolute;
  inset: 0;
}

.preview-empty {
  position: relative;
  z-index: 1;
  background: color-mix(in srgb, var(--app-bg-color) 85%, transparent);
  border-radius: 999px;
  padding: 4px 12px;
  font-size: 12px;
  color: var(--text);
}

.button-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.save-btn {
  height: 34px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--header-bg-color);
  color: var(--text);
  padding: 0 14px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.save-btn:hover {
  background: var(--bg-surface1);
}

.save-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.save-btn.primary {
  background: var(--button-bg-color);
  border-color: var(--button-bg-color);
  color: var(--button-text-color);
}

.save-btn.primary:hover {
  background: var(--button-hover-bg-color);
}

.save-btn.danger {
  border-color: color-mix(in srgb, var(--color-error) 70%, transparent);
  color: var(--color-error);
  background: color-mix(in srgb, var(--color-error) 14%, transparent);
}

.hidden-input {
  display: none;
}

.opacity-row {
  border-top: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  padding-top: 10px;
}

.opacity-control {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 4px;
}

.opacity-slider {
  flex: 1;
  accent-color: var(--link-active-color);
}

.opacity-value {
  min-width: 40px;
  text-align: right;
  color: var(--text);
  font-size: 12px;
}

.disabled-tip,
.loading-tip,
.empty-tip {
  border: 1px dashed var(--border);
  border-radius: 8px;
  padding: 12px;
  text-align: center;
  color: var(--text-sub);
  font-size: 12px;
}

.html-theme-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-btn {
  width: 26px;
  height: 26px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  cursor: pointer;
}

.icon-btn:hover {
  background: var(--header-bg-color);
  border-color: var(--border);
}

.tab-row {
  display: flex;
  border-bottom: 1px solid var(--border);
}

.tab-btn {
  height: 34px;
  padding: 0 14px;
  border: none;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--text-sub);
  cursor: pointer;
  font-size: 13px;
}

.tab-btn.active {
  border-bottom-color: var(--link-active-color);
  color: var(--link-active-color);
  font-weight: 600;
}

.tab-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.toolbar-row {
  display: flex;
  gap: 10px;
  align-items: center;
}

.preset-list {
  list-style: none;
  margin: 0;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: 8px;
  max-height: 280px;
  overflow: auto;
}

.preset-item {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 10px;
  align-items: center;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
}

.preset-item:last-child {
  border-bottom: none;
}

.preset-item:hover {
  background: var(--header-bg-color);
}

.preset-name-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.preset-name {
  font-size: 13px;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preset-tag {
  padding: 1px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}

.preset-tag.preset {
  background: rgba(59, 130, 246, 0.18);
  color: #60a5fa;
}

.preset-tag.custom {
  background: rgba(34, 197, 94, 0.18);
  color: #22c55e;
}

.preset-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.mini-btn {
  height: 28px;
  border-radius: 5px;
  border: 1px solid var(--border);
  background: var(--header-bg-color);
  color: var(--text);
  padding: 0 10px;
  font-size: 12px;
  cursor: pointer;
}

.mini-btn:hover {
  background: var(--bg-surface1);
}

.mini-btn.danger {
  border-color: color-mix(in srgb, var(--color-error) 70%, transparent);
  color: var(--color-error);
  background: color-mix(in srgb, var(--color-error) 14%, transparent);
}

.mini-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.remote-url-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.remote-url-input-row {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: 8px;
}

.editor-mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  z-index: 12000;
}

.editor-dialog {
  width: min(720px, 100%);
  background: var(--bg-surface0);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.editor-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.editor-textarea {
  min-height: 220px;
  resize: vertical;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--app-bg-color);
  color: var(--text);
  padding: 10px 12px;
  font-size: 13px;
  line-height: 1.45;
  font-family: 'Cascadia Mono', Consolas, 'Courier New', monospace;
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
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

@media (max-width: 860px) {
  .toolbar-row,
  .remote-url-input-row,
  .preset-item {
    grid-template-columns: 1fr;
  }

  .preset-actions {
    justify-content: flex-start;
  }

  .button-row,
  .opacity-control {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
