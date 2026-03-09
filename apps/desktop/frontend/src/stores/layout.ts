import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useSettingsStore } from './settings';

export type PaneName =
  | 'connections'
  | 'dockerManager'
  | 'terminal'
  | 'fileManager'
  | 'editor'
  | 'commandBar'
  | 'statusMonitor'
  | 'commandHistory'
  | 'quickCommands';

export interface LayoutNode {
  id?: string;
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

const ALL_POSSIBLE_PANES: PaneName[] = [
  'connections',
  'dockerManager',
  'terminal',
  'commandBar',
  'fileManager',
  'editor',
  'statusMonitor',
  'commandHistory',
  'quickCommands',
];

const SETTINGS_KEYS = {
  layoutConfig: 'layout_config',
  leftVisible: 'layout_left_visible',
  rightVisible: 'layout_right_visible',
  leftSize: 'layout_left_size',
  rightSize: 'layout_right_size',
  headerVisible: 'layout_header_visible',
  layoutLocked: 'layoutLocked',
} as const;

const LEGACY_STATUS_COLUMN_SIZE = 14.6;
const LEGACY_MAIN_COLUMN_SIZE = 58;
const DEFAULT_STATUS_COLUMN_SIZE = 15.55;
const DEFAULT_MAIN_COLUMN_SIZE = 57.05;

const DEFAULT_LEFT_SIZE = 14.6;
const DEFAULT_RIGHT_SIZE = 27.4;

const DEFAULT_LAYOUT: LayoutConfig = {
  root: {
    id: 'root',
    type: 'split',
    direction: 'horizontal',
    children: [
      {
        id: 'status-column',
        type: 'split',
        direction: 'vertical',
        size: DEFAULT_STATUS_COLUMN_SIZE,
        children: [
          { id: 'status-monitor', type: 'pane', pane: 'statusMonitor', size: 44.6 },
          { id: 'command-history', type: 'pane', pane: 'commandHistory', size: 26.2 },
          { id: 'quick-commands', type: 'pane', pane: 'quickCommands', size: 29.2 },
        ],
      },
      {
        id: 'main-column',
        type: 'split',
        direction: 'vertical',
        size: DEFAULT_MAIN_COLUMN_SIZE,
        children: [
          { id: 'terminal-pane', type: 'pane', pane: 'terminal', size: 59.9 },
          { id: 'command-bar', type: 'pane', pane: 'commandBar', size: 5 },
          { id: 'file-manager', type: 'pane', pane: 'fileManager', size: 35.1 },
        ],
      },
      {
        id: 'editor-column',
        type: 'split',
        direction: 'vertical',
        size: 27.4,
        children: [{ id: 'editor-pane', type: 'pane', pane: 'editor', size: 100 }],
      },
    ],
  },
  leftSidebar: {
    id: 'left-sidebar',
    children: [
      { id: 'connections-pane', type: 'pane', pane: 'connections', size: 50 },
      { id: 'docker-pane', type: 'pane', pane: 'dockerManager', size: 50 },
    ],
    direction: 'vertical',
    type: 'split',
  },
  rightSidebar: undefined,
};

let globalLayoutResizeRaf = 0;

function createLayoutNodeId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return Math.random().toString(36).slice(2, 11);
}

function ensureLayoutNodeIds(node: LayoutNode): LayoutNode {
  const nextNode: LayoutNode = {
    ...node,
    id: node.id ?? createLayoutNodeId(),
  };

  if (Array.isArray(node.children)) {
    nextNode.children = node.children.map((child) => ensureLayoutNodeIds(child));
  }

  return nextNode;
}

function ensureLayoutConfigNodeIds(config: LayoutConfig): LayoutConfig {
  return {
    root: ensureLayoutNodeIds(config.root),
    leftSidebar: config.leftSidebar ? ensureLayoutNodeIds(config.leftSidebar) : undefined,
    rightSidebar: config.rightSidebar ? ensureLayoutNodeIds(config.rightSidebar) : undefined,
  };
}

export function notifyGlobalLayoutResized(): void {
  if (typeof window === 'undefined') {
    return;
  }

  if (globalLayoutResizeRaf) {
    window.cancelAnimationFrame(globalLayoutResizeRaf);
  }

  globalLayoutResizeRaf = window.requestAnimationFrame(() => {
    window.dispatchEvent(new Event('resize'));
    window.dispatchEvent(new CustomEvent('nexus:layout-resized'));
    globalLayoutResizeRaf = 0;
  });
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function parseBoolean(raw: string, fallback: boolean): boolean {
  if (raw === 'true') return true;
  if (raw === 'false') return false;
  return fallback;
}

function parseNumber(raw: string, fallback: number, min: number, max: number): number {
  const value = Number.parseFloat(raw);
  if (!Number.isFinite(value)) {
    return fallback;
  }
  return clamp(value, min, max);
}

function deepCloneLayoutConfig(config: LayoutConfig): LayoutConfig {
  return ensureLayoutConfigNodeIds(JSON.parse(JSON.stringify(config)) as LayoutConfig);
}

function approximatelyEqual(value: number | undefined, target: number, tolerance = 0.05): boolean {
  if (!Number.isFinite(value)) {
    return false;
  }
  return Math.abs((value as number) - target) <= tolerance;
}

function patchLegacyStatusColumnWidth(config: LayoutConfig): void {
  const root = config.root;
  if (root.type !== 'split' || !Array.isArray(root.children) || root.children.length < 2) {
    return;
  }

  const statusColumn = root.children[0];
  const mainColumn = root.children[1];

  if (statusColumn.type !== 'split' || mainColumn.type !== 'split') {
    return;
  }

  const hasStatusMonitor = Array.isArray(statusColumn.children)
    && statusColumn.children.some((child) => child.type === 'pane' && child.pane === 'statusMonitor');
  if (!hasStatusMonitor) {
    return;
  }

  if (
    approximatelyEqual(statusColumn.size, LEGACY_STATUS_COLUMN_SIZE)
    && approximatelyEqual(mainColumn.size, LEGACY_MAIN_COLUMN_SIZE)
  ) {
    statusColumn.size = DEFAULT_STATUS_COLUMN_SIZE;
    mainColumn.size = DEFAULT_MAIN_COLUMN_SIZE;
  }
}

export const useLayoutStore = defineStore('layout', () => {
  const layoutConfig = ref<LayoutConfig>(deepCloneLayoutConfig(DEFAULT_LAYOUT));
  const leftSidebarVisible = ref(false);
  const rightSidebarVisible = ref(false);
  const leftSidebarSize = ref(DEFAULT_LEFT_SIZE);
  const rightSidebarSize = ref(DEFAULT_RIGHT_SIZE);
  const headerVisible = ref(true);
  const layoutLocked = ref(false);
  let persistSizesTimer: ReturnType<typeof setTimeout> | null = null;

  async function ensureSettingsLoaded() {
    const settings = useSettingsStore();
    if (!settings.loaded) {
      await settings.loadAll();
    }
    return settings;
  }

  async function persistHeaderVisibility() {
    const settings = await ensureSettingsLoaded();
    await settings.set(SETTINGS_KEYS.headerVisible, String(headerVisible.value));
  }

  async function persistLayoutLocked() {
    const settings = await ensureSettingsLoaded();
    await settings.set(SETTINGS_KEYS.layoutLocked, String(layoutLocked.value));
  }

  function scheduleSizePersistence() {
    if (persistSizesTimer) {
      clearTimeout(persistSizesTimer);
    }
    persistSizesTimer = setTimeout(() => {
      persistSizesTimer = null;
      void saveLayout();
    }, 200);
  }

  async function loadLayout() {
    const settings = await ensureSettingsLoaded();

    const json = settings.get(SETTINGS_KEYS.layoutConfig);
    if (json) {
      try {
        layoutConfig.value = JSON.parse(json);
      } catch {
        layoutConfig.value = deepCloneLayoutConfig(DEFAULT_LAYOUT);
      }
    }
    layoutConfig.value = ensureLayoutConfigNodeIds(layoutConfig.value);
    patchLegacyStatusColumnWidth(layoutConfig.value);

    leftSidebarVisible.value = parseBoolean(settings.get(SETTINGS_KEYS.leftVisible, 'false'), false);
    rightSidebarVisible.value = parseBoolean(settings.get(SETTINGS_KEYS.rightVisible, 'false'), false);
    leftSidebarSize.value = parseNumber(
      settings.get(SETTINGS_KEYS.leftSize, String(DEFAULT_LEFT_SIZE)),
      DEFAULT_LEFT_SIZE,
      10,
      30,
    );
    rightSidebarSize.value = parseNumber(
      settings.get(SETTINGS_KEYS.rightSize, String(DEFAULT_RIGHT_SIZE)),
      DEFAULT_RIGHT_SIZE,
      15,
      40,
    );
    headerVisible.value = parseBoolean(settings.get(SETTINGS_KEYS.headerVisible, 'true'), true);
    layoutLocked.value = parseBoolean(settings.get(SETTINGS_KEYS.layoutLocked, 'false'), false);
  }

  async function saveLayout() {
    const settings = await ensureSettingsLoaded();
    await settings.set(SETTINGS_KEYS.layoutConfig, JSON.stringify(layoutConfig.value));
    await settings.set(SETTINGS_KEYS.leftVisible, String(leftSidebarVisible.value));
    await settings.set(SETTINGS_KEYS.rightVisible, String(rightSidebarVisible.value));
    await settings.set(SETTINGS_KEYS.leftSize, String(leftSidebarSize.value));
    await settings.set(SETTINGS_KEYS.rightSize, String(rightSidebarSize.value));
    await settings.set(SETTINGS_KEYS.headerVisible, String(headerVisible.value));
    await settings.set(SETTINGS_KEYS.layoutLocked, String(layoutLocked.value));
  }

  function resetLayout() {
    layoutConfig.value = deepCloneLayoutConfig(DEFAULT_LAYOUT);
    leftSidebarVisible.value = false;
    rightSidebarVisible.value = false;
    leftSidebarSize.value = DEFAULT_LEFT_SIZE;
    rightSidebarSize.value = DEFAULT_RIGHT_SIZE;
    headerVisible.value = true;
    layoutLocked.value = false;
    void saveLayout();
  }

  function toggleLeftSidebar() {
    leftSidebarVisible.value = !leftSidebarVisible.value;
  }

  function toggleRightSidebar() {
    rightSidebarVisible.value = !rightSidebarVisible.value;
  }

  function setLeftSidebarSize(size: number) {
    leftSidebarSize.value = clamp(size, 10, 30);
    scheduleSizePersistence();
  }

  function setRightSidebarSize(size: number) {
    rightSidebarSize.value = clamp(size, 15, 40);
    scheduleSizePersistence();
  }

  function setHeaderVisibility(visible: boolean) {
    if (headerVisible.value === visible) {
      return;
    }

    headerVisible.value = visible;
    void persistHeaderVisibility();
  }

  function toggleHeaderVisibility() {
    setHeaderVisibility(!headerVisible.value);
  }

  function setLayoutLocked(locked: boolean) {
    if (layoutLocked.value === locked) {
      return;
    }

    layoutLocked.value = locked;
    void persistLayoutLocked();
  }

  function toggleLayoutLocked() {
    setLayoutLocked(!layoutLocked.value);
  }

  function generateId(): string {
    return createLayoutNodeId();
  }

  function getSystemDefaultLayoutConfig(): LayoutConfig {
    return deepCloneLayoutConfig(DEFAULT_LAYOUT);
  }

  return {
    allPossiblePanes: ALL_POSSIBLE_PANES,
    layoutConfig,
    leftSidebarVisible,
    rightSidebarVisible,
    leftSidebarSize,
    rightSidebarSize,
    headerVisible,
    layoutLocked,
    loadLayout,
    saveLayout,
    resetLayout,
    toggleLeftSidebar,
    toggleRightSidebar,
    setLeftSidebarSize,
    setRightSidebarSize,
    setHeaderVisibility,
    toggleHeaderVisibility,
    setLayoutLocked,
    toggleLayoutLocked,
    generateId,
    getSystemDefaultLayoutConfig,
  };
});
