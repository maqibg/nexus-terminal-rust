import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useSettingsStore } from './settings';

export type PaneName =
  | 'terminal'
  | 'fileManager'
  | 'editor'
  | 'commandBar'
  | 'statusMonitor'
  | 'commandHistory'
  | 'quickCommands';

export interface LayoutNode {
  type: 'split' | 'pane';
  direction?: 'horizontal' | 'vertical';
  size?: number;
  children?: LayoutNode[];
  pane?: PaneName;
}

export interface LayoutConfig {
  root: LayoutNode;
  leftSidebar?: LayoutNode;
  rightSidebar?: LayoutNode;
}

const DEFAULT_LAYOUT: LayoutConfig = {
  leftSidebar: {
    type: 'split',
    direction: 'vertical',
    children: [
      { type: 'pane', pane: 'statusMonitor', size: 44.6 },
      { type: 'pane', pane: 'commandHistory', size: 26.2 },
      { type: 'pane', pane: 'quickCommands', size: 29.2 },
    ],
  },
  root: {
    type: 'split',
    direction: 'vertical',
    children: [
      { type: 'pane', pane: 'terminal', size: 59.9 },
      { type: 'pane', pane: 'commandBar', size: 5 },
      { type: 'pane', pane: 'fileManager', size: 35.1 },
    ],
  },
  rightSidebar: {
    type: 'pane',
    pane: 'editor',
  },
};

export const useLayoutStore = defineStore('layout', () => {
  const layoutConfig = ref<LayoutConfig>(structuredClone(DEFAULT_LAYOUT));
  const leftSidebarVisible = ref(true);
  const rightSidebarVisible = ref(true);
  const leftSidebarSize = ref(14.6);
  const rightSidebarSize = ref(27.4);

  async function loadLayout() {
    const settings = useSettingsStore();
    const json = settings.get('layout_config');
    if (json) {
      try { layoutConfig.value = JSON.parse(json); } catch { /* keep default */ }
    }
  }

  async function saveLayout() {
    const settings = useSettingsStore();
    await settings.set('layout_config', JSON.stringify(layoutConfig.value));
  }

  function resetLayout() {
    layoutConfig.value = structuredClone(DEFAULT_LAYOUT);
    leftSidebarVisible.value = true;
    rightSidebarVisible.value = true;
    leftSidebarSize.value = 14.6;
    rightSidebarSize.value = 27.4;
    saveLayout();
  }

  function toggleLeftSidebar() {
    leftSidebarVisible.value = !leftSidebarVisible.value;
  }

  function toggleRightSidebar() {
    rightSidebarVisible.value = !rightSidebarVisible.value;
  }

  function setLeftSidebarSize(size: number) {
    leftSidebarSize.value = size;
  }

  function setRightSidebarSize(size: number) {
    rightSidebarSize.value = size;
  }

  return {
    layoutConfig,
    leftSidebarVisible,
    rightSidebarVisible,
    leftSidebarSize,
    rightSidebarSize,
    loadLayout,
    saveLayout,
    resetLayout,
    toggleLeftSidebar,
    toggleRightSidebar,
    setLeftSidebarSize,
    setRightSidebarSize,
  };
});
