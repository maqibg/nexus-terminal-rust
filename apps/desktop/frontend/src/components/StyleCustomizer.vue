<script setup lang="ts">
import { ref } from 'vue';
import type { TerminalTheme } from '@/stores/appearance';
import StyleCustomizerUITab from './StyleCustomizerUITab.vue';
import StyleCustomizerTerminalTab from './StyleCustomizerTerminalTab.vue';
import StyleCustomizerBackgroundTab from './StyleCustomizerBackgroundTab.vue';
import StyleCustomizerOtherTab from './StyleCustomizerOtherTab.vue';

defineProps<{ visible: boolean }>();

const emit = defineEmits<{ close: [] }>();

type TabKey = 'ui' | 'terminal' | 'background' | 'other';
type UITabExpose = {
  handleSaveUiTheme?: () => Promise<void>;
  handleResetUiTheme?: () => Promise<void>;
};

const currentTab = ref<TabKey>('ui');
const uiTabRef = ref<UITabExpose | null>(null);

const isEditingTheme = ref(false);
const editingTheme = ref<TerminalTheme | null>(null);

const tabs: Array<{ key: TabKey; label: string }> = [
  { key: 'ui', label: '界面样式' },
  { key: 'terminal', label: '终端样式' },
  { key: 'background', label: '背景设置' },
  { key: 'other', label: '其他设置' },
];

const closeCustomizer = () => {
  isEditingTheme.value = false;
  editingTheme.value = null;
  emit('close');
};

const saveUiTheme = async () => {
  await uiTabRef.value?.handleSaveUiTheme?.();
};

const resetUiTheme = async () => {
  await uiTabRef.value?.handleResetUiTheme?.();
};
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="style-customizer-mask" @click.self="closeCustomizer">
      <div class="style-customizer-dialog">
        <header class="style-customizer-header">
          <h2>外观自定义</h2>
          <button class="close-btn" title="关闭" @click="closeCustomizer">×</button>
        </header>

        <div class="style-customizer-main">
          <aside class="tab-sidebar">
            <button
              v-for="tab in tabs"
              :key="tab.key"
              class="tab-btn"
              :class="{ active: currentTab === tab.key }"
              :disabled="isEditingTheme && tab.key !== 'terminal'"
              @click="currentTab = tab.key"
            >
              {{ tab.label }}
            </button>
          </aside>

          <main class="tab-content">
            <StyleCustomizerUITab v-if="currentTab === 'ui'" ref="uiTabRef" />
            <StyleCustomizerTerminalTab
              v-if="currentTab === 'terminal'"
              :is-editing-theme="isEditingTheme"
              :editing-theme="editingTheme"
              @update:is-editing-theme="value => isEditingTheme = value"
              @update:editing-theme="value => editingTheme = value"
            />
            <StyleCustomizerBackgroundTab v-if="currentTab === 'background'" />
            <StyleCustomizerOtherTab v-if="currentTab === 'other'" />
          </main>
        </div>

        <footer class="style-customizer-footer">
          <button v-if="currentTab === 'ui'" class="footer-btn" @click="resetUiTheme">重置界面主题</button>
          <button v-if="currentTab === 'ui'" class="footer-btn primary" @click="saveUiTheme">保存界面主题</button>
          <button class="footer-btn" @click="closeCustomizer">关闭</button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.style-customizer-mask {
  position: fixed;
  inset: 0;
  z-index: 9800;
  background: rgba(0, 0, 0, 0.65);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.style-customizer-dialog {
  width: min(800px, 90vw);
  height: min(85vh, 700px);
  min-height: 620px;
  background: var(--bg-surface0);
  border: 1px solid var(--border);
  border-radius: 10px;
  box-shadow: 0 20px 48px rgba(0, 0, 0, 0.48);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.style-customizer-header {
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
  background: var(--header-bg-color);
}

.style-customizer-header h2 {
  margin: 0;
  font-size: 26px;
  line-height: 1;
  color: var(--text);
  font-weight: 600;
}

.close-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub);
  font-size: 24px;
  line-height: 1;
  cursor: pointer;
}

.close-btn:hover {
  background: var(--link-active-bg-color);
  color: var(--text);
}

.style-customizer-main {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 190px 1fr;
}

.tab-sidebar {
  border-right: 1px solid var(--border);
  background: var(--header-bg-color);
  padding: 12px 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow: auto;
}

.tab-btn {
  min-height: 38px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  text-align: left;
  padding: 0 12px;
  font-size: 14px;
  cursor: pointer;
  transition: 0.15s ease;
}

.tab-btn:hover {
  background: var(--link-active-bg-color);
}

.tab-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.tab-btn.active {
  background: var(--button-bg-color);
  color: var(--button-text-color);
  font-weight: 600;
}

.tab-content {
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: 14px 16px;
}

.style-customizer-footer {
  min-height: 56px;
  padding: 10px 14px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  border-top: 1px solid var(--border);
  background: var(--footer-bg-color);
}

.footer-btn {
  height: 34px;
  border-radius: 6px;
  border: 1px solid var(--border);
  padding: 0 16px;
  font-size: 13px;
  color: var(--text);
  background: var(--header-bg-color);
  cursor: pointer;
}

.footer-btn:hover {
  background: var(--bg-surface1);
}

.footer-btn.primary {
  border-color: var(--button-bg-color);
  background: var(--button-bg-color);
  color: var(--button-text-color);
}

.footer-btn.primary:hover {
  background: var(--button-hover-bg-color);
}

@media (max-width: 860px) {
  .style-customizer-dialog {
    min-height: 480px;
  }

  .style-customizer-main {
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr;
  }

  .tab-sidebar {
    border-right: none;
    border-bottom: 1px solid var(--border);
    flex-direction: row;
    flex-wrap: wrap;
  }

  .tab-btn {
    min-width: 120px;
  }
}
</style>
