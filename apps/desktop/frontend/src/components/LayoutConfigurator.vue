<template>
  <div class="layout-configurator">
    <div class="config-header">
      <span>布局配置</span>
      <button class="btn-reset" @click="resetLayout">重置默认</button>
    </div>
    <div class="config-info">
      <p>当前布局: 三栏模式</p>
      <label class="toggle-row">
        <input type="checkbox" v-model="layoutStore.leftSidebarVisible" /> 左侧边栏
      </label>
      <label class="toggle-row">
        <input type="checkbox" v-model="layoutStore.rightSidebarVisible" /> 右侧边栏
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useLayoutStore } from '@/stores/layout';

const layoutStore = useLayoutStore();

function resetLayout() {
  layoutStore.resetLayout();
}
</script>

<style scoped>
.layout-configurator { padding: 12px; }
.config-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; font-size: 14px; font-weight: 600; }
.btn-reset { padding: 4px 10px; border-radius: 4px; border: 1px solid var(--border); background: transparent; color: var(--text-sub); cursor: pointer; font-size: 12px; }
.btn-reset:hover { background: var(--bg-surface1); }
.config-info { font-size: 13px; color: var(--text); }
.config-info p { margin: 0 0 12px; }
.toggle-row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; font-size: 13px; color: var(--text); cursor: pointer; }
</style>
