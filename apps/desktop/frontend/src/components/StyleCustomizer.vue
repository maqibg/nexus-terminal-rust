<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="$emit('close')">
      <div class="customizer-card">
        <div class="customizer-header">
          <span>样式定制</span>
          <span class="close-btn" @click="$emit('close')">&times;</span>
        </div>
        <div class="customizer-body">
          <nav class="tab-nav">
            <button v-for="tab in tabs" :key="tab.key" class="tab-btn" :class="{ active: currentTab === tab.key }" @click="currentTab = tab.key">{{ tab.label }}</button>
          </nav>
          <div class="tab-content">
            <StyleCustomizerUITab v-if="currentTab === 'ui'" />
            <StyleCustomizerTerminalTab v-if="currentTab === 'terminal'" />
            <StyleCustomizerBackgroundTab v-if="currentTab === 'background'" />
            <StyleCustomizerOtherTab v-if="currentTab === 'other'" />
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import StyleCustomizerUITab from './StyleCustomizerUITab.vue';
import StyleCustomizerTerminalTab from './StyleCustomizerTerminalTab.vue';
import StyleCustomizerBackgroundTab from './StyleCustomizerBackgroundTab.vue';
import StyleCustomizerOtherTab from './StyleCustomizerOtherTab.vue';

defineProps<{ visible: boolean }>();
defineEmits<{ close: [] }>();

const tabs = [
  { key: 'ui', label: 'UI 样式' },
  { key: 'terminal', label: '终端' },
  { key: 'background', label: '背景' },
  { key: 'other', label: '其他' },
] as const;

const currentTab = ref<'ui' | 'terminal' | 'background' | 'other'>('ui');
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.customizer-card { background: var(--bg-surface0); border-radius: 8px; width: 600px; max-height: 80vh; display: flex; flex-direction: column; border: 1px solid var(--border); overflow: hidden; }
.customizer-header { display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; font-size: 16px; font-weight: 600; border-bottom: 1px solid var(--border); }
.close-btn { cursor: pointer; font-size: 20px; color: var(--text-dim); }
.close-btn:hover { color: var(--red); }
.customizer-body { display: flex; flex: 1; overflow: hidden; }
.tab-nav { display: flex; flex-direction: column; gap: 2px; padding: 8px; border-right: 1px solid var(--border); min-width: 100px; flex-shrink: 0; }
.tab-btn { padding: 6px 12px; border: none; border-radius: 4px; background: transparent; color: var(--text-sub); cursor: pointer; font-size: 13px; text-align: left; }
.tab-btn:hover { background: var(--bg-mantle); }
.tab-btn.active { background: var(--blue); color: var(--bg-base); font-weight: 600; }
.tab-content { flex: 1; padding: 16px; overflow-y: auto; }
</style>
