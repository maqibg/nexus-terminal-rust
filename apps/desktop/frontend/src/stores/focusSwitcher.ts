import { defineStore } from 'pinia';
import { computed, nextTick, ref } from 'vue';
import { settingsApi } from '@/lib/api';

export interface FocusableInput {
  id: string;
  label: string;
}

export interface FocusItemConfig {
  shortcut?: string;
}

export interface FocusSwitcherFullConfig {
  sequence: string[];
  shortcuts: Record<string, FocusItemConfig>;
}

type FocusAction = () => boolean | undefined | Promise<boolean | undefined>;

const SETTING_KEY = 'focus_switcher_config';
const LEGACY_SHORTCUTS_KEY = 'focus_shortcuts';
const LEGACY_SEQUENCE_KEYS = ['focusSwitcherSequence', 'focus_switcher_sequence'];

const DEFAULT_AVAILABLE_INPUTS: FocusableInput[] = [
  { id: 'commandHistorySearch', label: '命令历史搜索' },
  { id: 'quickCommandsSearch', label: '快捷指令搜索' },
  { id: 'fileManagerSearch', label: '文件管理器搜索' },
  { id: 'commandInput', label: '命令输入' },
  { id: 'terminalSearch', label: '终端内搜索' },
  { id: 'connectionListSearch', label: '连接列表搜索' },
  { id: 'fileEditorActive', label: '文件编辑器' },
  { id: 'fileManagerPathInput', label: '文件管理器路径编辑' },
];

function parseJson(value: string | undefined): unknown {
  if (!value) {
    return null;
  }
  try {
    return JSON.parse(value);
  } catch {
    return null;
  }
}

function isValidShortcut(value: unknown): value is string {
  return typeof value === 'string' && /^Alt\+[A-Z0-9]$/.test(value.trim());
}

function normalizeConfig(raw: unknown, availableInputs: FocusableInput[]): FocusSwitcherFullConfig {
  const availableIds = new Set(availableInputs.map((input) => input.id));
  const normalized: FocusSwitcherFullConfig = {
    sequence: [],
    shortcuts: {},
  };

  if (!raw || typeof raw !== 'object') {
    return normalized;
  }

  const source = raw as {
    sequence?: unknown;
    shortcuts?: Record<string, { shortcut?: unknown }>;
  };

  if (Array.isArray(source.sequence)) {
    normalized.sequence = source.sequence.filter(
      (id): id is string => typeof id === 'string' && availableIds.has(id),
    );
  }

  if (source.shortcuts && typeof source.shortcuts === 'object') {
    for (const id of Object.keys(source.shortcuts)) {
      if (!availableIds.has(id)) {
        continue;
      }
      const shortcut = source.shortcuts[id]?.shortcut;
      if (isValidShortcut(shortcut)) {
        normalized.shortcuts[id] = { shortcut: shortcut.trim().toUpperCase() };
      }
    }
  }

  return normalized;
}

export const useFocusSwitcherStore = defineStore('focusSwitcher', () => {
  const availableInputs = ref<FocusableInput[]>([...DEFAULT_AVAILABLE_INPUTS]);
  const sequenceOrder = ref<string[]>([]);
  const itemConfigs = ref<Record<string, FocusItemConfig>>({});
  const isConfiguratorVisible = ref(false);
  const registeredActions = ref<Map<string, FocusAction[]>>(new Map());

  function ensureConfigSlots(): void {
    for (const input of availableInputs.value) {
      if (!itemConfigs.value[input.id]) {
        itemConfigs.value[input.id] = {};
      }
    }
  }

  async function loadConfigurationFromBackend(): Promise<void> {
    try {
      const allSettings = await settingsApi.getAll();
      const settingsMap = new Map(allSettings.map((setting) => [setting.key, setting.value]));

      let config = normalizeConfig(parseJson(settingsMap.get(SETTING_KEY)), availableInputs.value);

      if (config.sequence.length === 0 && Object.keys(config.shortcuts).length === 0) {
        const legacyShortcuts = parseJson(settingsMap.get(LEGACY_SHORTCUTS_KEY));
        const legacySequenceValue = LEGACY_SEQUENCE_KEYS
          .map((key) => parseJson(settingsMap.get(key)))
          .find((value) => value !== null);
        const legacyConfig = {
          sequence: Array.isArray(legacySequenceValue) ? legacySequenceValue : [],
          shortcuts: legacyShortcuts && typeof legacyShortcuts === 'object' ? legacyShortcuts : {},
        };
        config = normalizeConfig(legacyConfig, availableInputs.value);
      }

      sequenceOrder.value = config.sequence;
      itemConfigs.value = config.shortcuts;
      ensureConfigSlots();
    } catch {
      sequenceOrder.value = [];
      itemConfigs.value = {};
      ensureConfigSlots();
    }
  }

  async function saveConfigurationToBackend(): Promise<void> {
    const shortcuts: Record<string, FocusItemConfig> = {};

    for (const input of availableInputs.value) {
      const shortcut = itemConfigs.value[input.id]?.shortcut;
      if (isValidShortcut(shortcut)) {
        shortcuts[input.id] = { shortcut: shortcut.trim().toUpperCase() };
      }
    }

    const payload: FocusSwitcherFullConfig = {
      sequence: sequenceOrder.value.filter((id) => availableInputs.value.some((input) => input.id === id)),
      shortcuts,
    };

    await settingsApi.set(SETTING_KEY, JSON.stringify(payload));
  }

  function updateConfiguration(newConfig: FocusSwitcherFullConfig): void {
    const normalized = normalizeConfig(newConfig, availableInputs.value);
    sequenceOrder.value = normalized.sequence;
    itemConfigs.value = normalized.shortcuts;
    ensureConfigSlots();
    void saveConfigurationToBackend();
  }

  function toggleConfigurator(visible?: boolean): void {
    isConfiguratorVisible.value = visible === undefined ? !isConfiguratorVisible.value : visible;
  }

  function registerFocusAction(id: string, action: FocusAction): () => void {
    if (!availableInputs.value.some((input) => input.id === id)) {
      return () => {};
    }

    const actions = registeredActions.value.get(id) ?? [];
    actions.push(action);
    registeredActions.value.set(id, actions);

    return () => {
      const current = registeredActions.value.get(id);
      if (!current) {
        return;
      }
      const idx = current.indexOf(action);
      if (idx >= 0) {
        current.splice(idx, 1);
      }
      if (current.length === 0) {
        registeredActions.value.delete(id);
      }
    };
  }

  async function focusTarget(id: string): Promise<boolean> {
    const actions = registeredActions.value.get(id);
    if (!actions || actions.length === 0) {
      return false;
    }

    for (const action of actions) {
      try {
        const result = await action();
        if (result === true) {
          return true;
        }
      } catch {
        // ignore and try next action
      }
    }

    return false;
  }

  function getNextFocusTargetId(currentFocusedId: string | null): string | null {
    if (sequenceOrder.value.length === 0) {
      return null;
    }

    if (!currentFocusedId) {
      return sequenceOrder.value[0];
    }

    const currentIndex = sequenceOrder.value.findIndex((id) => id === currentFocusedId);
    if (currentIndex < 0) {
      return sequenceOrder.value[0];
    }

    return sequenceOrder.value[(currentIndex + 1) % sequenceOrder.value.length];
  }

  function getFocusTargetIdByShortcut(shortcut: string): string | null {
    const normalizedShortcut = shortcut.trim().toUpperCase();
    for (const [id, config] of Object.entries(itemConfigs.value)) {
      if (config.shortcut?.trim().toUpperCase() === normalizedShortcut) {
        return id;
      }
    }
    return null;
  }

  const getSequenceInputs = computed(() => {
    const inputMap = new Map(availableInputs.value.map((input) => [input.id, input]));
    return sequenceOrder.value
      .map((id) => inputMap.get(id))
      .filter((input): input is FocusableInput => !!input)
      .map((input) => ({
        ...input,
        shortcut: itemConfigs.value[input.id]?.shortcut,
      }));
  });

  const getAvailableInputsForConfigurator = computed(() => {
    const sequenceSet = new Set(sequenceOrder.value);
    return availableInputs.value
      .filter((input) => !sequenceSet.has(input.id))
      .map((input) => ({
        ...input,
        shortcut: itemConfigs.value[input.id]?.shortcut,
      }));
  });

  nextTick(() => {
    void loadConfigurationFromBackend();
  });

  return {
    availableInputs,
    sequenceOrder,
    itemConfigs,
    isConfiguratorVisible,
    loadConfigurationFromBackend,
    saveConfigurationToBackend,
    updateConfiguration,
    toggleConfigurator,
    registerFocusAction,
    focusTarget,
    getNextFocusTargetId,
    getFocusTargetIdByShortcut,
    getSequenceInputs,
    getAvailableInputsForConfigurator,
  };
});
