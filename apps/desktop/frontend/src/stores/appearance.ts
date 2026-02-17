import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { ITheme } from '@xterm/xterm';
import { settingsApi, type TerminalTheme as ApiTerminalTheme } from '@/lib/api';
import terminalThemePresets from '@/assets/terminal-theme-presets.json';

export const defaultXtermTheme: ITheme = {
  background: '#1e1e1e',
  foreground: '#d4d4d4',
  cursor: '#d4d4d4',
  selectionBackground: '#264f78',
  black: '#000000',
  red: '#cd3131',
  green: '#0dbc79',
  yellow: '#e5e510',
  blue: '#2472c8',
  magenta: '#bc3fbc',
  cyan: '#11a8cd',
  white: '#e5e5e5',
  brightBlack: '#666666',
  brightRed: '#f14c4c',
  brightGreen: '#23d18b',
  brightYellow: '#f5f543',
  brightBlue: '#3b8eea',
  brightMagenta: '#d670d6',
  brightCyan: '#29b8db',
  brightWhite: '#e5e5e5',
};

export const defaultUiTheme: Record<string, string> = {
  '--app-bg-color': '#ffffff',
  '--text-color': '#333333',
  '--text-color-secondary': '#666666',
  '--border-color': '#cccccc',
  '--link-color': '#8E44AD',
  '--link-hover-color': '#B180E0',
  '--link-active-color': '#A06CD5',
  '--link-active-bg-color': '#F3EBFB',
  '--nav-item-active-bg-color': 'var(--link-active-bg-color)',
  '--header-bg-color': '#f0f0f0',
  '--footer-bg-color': '#f0f0f0',
  '--button-bg-color': '#A06CD5',
  '--button-text-color': '#ffffff',
  '--button-hover-bg-color': '#8E44AD',
  '--icon-color': 'var(--text-color-secondary)',
  '--icon-hover-color': 'var(--link-hover-color)',
  '--split-line-color': 'var(--border-color)',
  '--split-line-hover-color': 'var(--border-color)',
  '--input-focus-border-color': 'var(--link-active-color)',
  '--input-focus-glow': 'var(--link-active-color)',
  '--overlay-bg-color': 'rgba(0, 0, 0, 0.6)',
  '--color-success': '#5cb85c',
  '--color-error': '#d9534f',
  '--color-warning': '#f0ad4e',
  '--font-family-sans-serif': 'sans-serif',
  '--base-padding': '1rem',
  '--base-margin': '0.5rem',
};

const DEFAULT_REMOTE_REPO_URL = 'https://github.com/Heavrnl/nexus-terminal/tree/main/doc/custom_html_theme';
const LEGACY_REMOTE_REPO_URL = 'https://github.com/Heavrnl/nexus-terminal/tree/main/packages/backend/html-presets';
const LOCAL_PRESETS_STORAGE_KEY = 'nexus-terminal:local-html-presets';
const REMOTE_REPO_STORAGE_KEY = 'nexus-terminal:remote-html-repository-url';

const BUILTIN_HTML_PRESET_MODULES = import.meta.glob('../assets/html-presets/*.html', {
  eager: true,
  import: 'default',
  query: '?raw',
}) as Record<string, string>;

const BUILTIN_HTML_PRESETS: Record<string, string> = Object.keys(BUILTIN_HTML_PRESET_MODULES).length > 0
  ? Object.fromEntries(
    Object.entries(BUILTIN_HTML_PRESET_MODULES).map(([path, content]) => {
      const fileNameWithQuery = path.split('/').pop() ?? path;
      const fileName = fileNameWithQuery.split('?')[0];
      return [decodeURIComponent(fileName), content];
    }),
  )
  : { '默认黑色.html': '' };

type ExtendedITheme = ITheme & {
  cursorAccent?: string;
  selectionForeground?: string;
  selectionInactiveBackground?: string;
};

interface BuiltinTerminalThemePreset {
  name: string;
  themeData: Partial<ExtendedITheme>;
}

const BUILTIN_TERMINAL_THEME_PRESETS: BuiltinTerminalThemePreset[] = (terminalThemePresets as BuiltinTerminalThemePreset[])
  .filter((preset) => typeof preset?.name === 'string' && !!preset.name.trim() && typeof preset?.themeData === 'object');

type PresetType = 'preset' | 'custom';

export interface TerminalTheme {
  _id?: string;
  name: string;
  themeData: ExtendedITheme;
  isPreset: boolean;
}

interface AppearanceSettingsState {
  customUiTheme?: string;
  activeTerminalThemeId?: number | null;
  terminalFontFamily?: string;
  terminalFontSize?: number;
  editorFontSize?: number;
  editorFontFamily?: string;
  mobileEditorFontSize?: number;
  terminalBackgroundEnabled?: boolean;
  terminalBackgroundImage?: string;
  terminalBackgroundOverlayOpacity?: number;
  terminal_custom_html?: string;
  terminalTextStrokeEnabled?: boolean;
  terminalTextStrokeWidth?: number;
  terminalTextStrokeColor?: string;
  terminalTextShadowEnabled?: boolean;
  terminalTextShadowOffsetX?: number;
  terminalTextShadowOffsetY?: number;
  terminalTextShadowBlur?: number;
  terminalTextShadowColor?: string;
}

const APPEARANCE_ALIASES = {
  customUiTheme: ['customUiTheme', 'custom_ui_theme'],
  activeTerminalThemeId: ['activeTerminalThemeId', 'active_terminal_theme_id'],
  terminalFontFamily: ['terminal_font_family', 'terminalFontFamily'],
  terminalFontSize: ['terminal_font_size', 'terminalFontSize'],
  editorFontSize: ['editor_font_size', 'editorFontSize'],
  editorFontFamily: ['editor_font_family', 'editorFontFamily'],
  mobileEditorFontSize: ['mobile_editor_font_size', 'mobileEditorFontSize'],
  terminalBackgroundEnabled: ['terminalBackgroundEnabled', 'terminal_background_enabled'],
  terminalBackgroundImage: ['terminalBackgroundImage', 'terminal_background_image'],
  terminalBackgroundOverlayOpacity: ['terminalBackgroundOverlayOpacity', 'terminal_background_overlay_opacity'],
  terminalCustomHtml: ['terminal_custom_html', 'terminalCustomHTML'],
  terminalTextStrokeEnabled: ['terminalTextStrokeEnabled', 'terminal_text_stroke_enabled'],
  terminalTextStrokeWidth: ['terminalTextStrokeWidth', 'terminal_text_stroke_width'],
  terminalTextStrokeColor: ['terminalTextStrokeColor', 'terminal_text_stroke_color'],
  terminalTextShadowEnabled: ['terminalTextShadowEnabled', 'terminal_text_shadow_enabled'],
  terminalTextShadowOffsetX: ['terminalTextShadowOffsetX', 'terminal_text_shadow_offset_x'],
  terminalTextShadowOffsetY: ['terminalTextShadowOffsetY', 'terminal_text_shadow_offset_y'],
  terminalTextShadowBlur: ['terminalTextShadowBlur', 'terminal_text_shadow_blur'],
  terminalTextShadowColor: ['terminalTextShadowColor', 'terminal_text_shadow_color'],
} as const;

const toBoolean = (value: string | undefined, fallback = false): boolean => {
  if (value === undefined) {
    return fallback;
  }
  return String(value).toLowerCase() === 'true';
};

const toNumber = (value: string | undefined, fallback: number): number => {
  if (value === undefined) {
    return fallback;
  }
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? parsed : fallback;
};

const toInteger = (value: string | undefined, fallback: number | null): number | null => {
  if (value === undefined) {
    return fallback;
  }
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) ? parsed : fallback;
};

const normalizeName = (name: string): string => (name.endsWith('.html') ? name : `${name}.html`);

export const safeJsonParse = <T>(jsonString: string | undefined | null, defaultValue: T): T => {
  if (!jsonString) {
    return defaultValue;
  }
  try {
    return JSON.parse(jsonString) as T;
  } catch {
    return defaultValue;
  }
};

const backendThemeToITheme = (theme: ApiTerminalTheme): ExtendedITheme => ({
  background: theme.background ?? defaultXtermTheme.background,
  foreground: theme.foreground ?? defaultXtermTheme.foreground,
  cursor: theme.cursor ?? defaultXtermTheme.cursor,
  cursorAccent: theme.cursor_accent ?? undefined,
  selectionBackground: theme.selection_background ?? defaultXtermTheme.selectionBackground,
  selectionForeground: theme.selection_foreground ?? undefined,
  selectionInactiveBackground: theme.selection_inactive_background ?? undefined,
  black: theme.black ?? defaultXtermTheme.black,
  red: theme.red ?? defaultXtermTheme.red,
  green: theme.green ?? defaultXtermTheme.green,
  yellow: theme.yellow ?? defaultXtermTheme.yellow,
  blue: theme.blue ?? defaultXtermTheme.blue,
  magenta: theme.magenta ?? defaultXtermTheme.magenta,
  cyan: theme.cyan ?? defaultXtermTheme.cyan,
  white: theme.white ?? defaultXtermTheme.white,
  brightBlack: theme.bright_black ?? defaultXtermTheme.brightBlack,
  brightRed: theme.bright_red ?? defaultXtermTheme.brightRed,
  brightGreen: theme.bright_green ?? defaultXtermTheme.brightGreen,
  brightYellow: theme.bright_yellow ?? defaultXtermTheme.brightYellow,
  brightBlue: theme.bright_blue ?? defaultXtermTheme.brightBlue,
  brightMagenta: theme.bright_magenta ?? defaultXtermTheme.brightMagenta,
  brightCyan: theme.bright_cyan ?? defaultXtermTheme.brightCyan,
  brightWhite: theme.bright_white ?? defaultXtermTheme.brightWhite,
});

const toApiThemePayload = (
  id: number,
  name: string,
  themeData: ExtendedITheme,
  themeType: string,
): Record<string, unknown> => ({
  id,
  name,
  theme_type: themeType,
  background: themeData.background ?? null,
  foreground: themeData.foreground ?? null,
  cursor: themeData.cursor ?? null,
  selection_background: themeData.selectionBackground ?? null,
  black: themeData.black ?? null,
  red: themeData.red ?? null,
  green: themeData.green ?? null,
  yellow: themeData.yellow ?? null,
  blue: themeData.blue ?? null,
  magenta: themeData.magenta ?? null,
  cyan: themeData.cyan ?? null,
  white: themeData.white ?? null,
  bright_black: themeData.brightBlack ?? null,
  bright_red: themeData.brightRed ?? null,
  bright_green: themeData.brightGreen ?? null,
  bright_yellow: themeData.brightYellow ?? null,
  bright_blue: themeData.brightBlue ?? null,
  bright_magenta: themeData.brightMagenta ?? null,
  bright_cyan: themeData.brightCyan ?? null,
  bright_white: themeData.brightWhite ?? null,
  cursor_accent: themeData.cursorAccent ?? null,
  selection_foreground: themeData.selectionForeground ?? null,
  selection_inactive_background: themeData.selectionInactiveBackground ?? null,
});

const readFileAsDataUrl = (file: File) =>
  new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ''));
    reader.onerror = () => reject(new Error('读取文件失败'));
    reader.readAsDataURL(file);
  });

const parseGitHubTreeUrl = (url: string): { owner: string; repo: string; branch: string; path: string } | null => {
  try {
    const parsed = new URL(url);
    if (parsed.hostname !== 'github.com') {
      return null;
    }
    const segments = parsed.pathname.split('/').filter(Boolean);
    if (segments.length < 4 || segments[2] !== 'tree') {
      return null;
    }
    const owner = segments[0];
    const repo = segments[1];
    const branch = segments[3];
    const path = segments.slice(4).join('/');
    return { owner, repo, branch, path };
  } catch {
    return null;
  }
};

export const useAppearanceStore = defineStore('appearance', () => {
  const appearance = ref<Record<string, string>>({});
  const loaded = ref(false);
  const appearanceSettings = ref<Partial<AppearanceSettingsState>>({});
  const allTerminalThemes = ref<TerminalTheme[]>([]);

  const localHtmlPresets = ref<Array<{ name: string; type: PresetType }>>([]);
  const remoteHtmlPresets = ref<Array<{ name: string; downloadUrl?: string }>>([]);
  const remoteHtmlPresetsRepositoryUrl = ref<string | null>(null);
  const activeHtmlPresetTab = ref<'local' | 'remote'>('local');
  const isLoadingHtmlPresets = ref(false);
  const htmlPresetError = ref<string | null>(null);
  const isStyleCustomizerVisible = ref(false);

  const pickValue = (keys: readonly string[], fallback = ''): string => {
    for (const key of keys) {
      const value = appearance.value[key];
      if (value !== undefined && value !== '') {
        return value;
      }
    }
    return fallback;
  };

  const currentUiTheme = computed<Record<string, string>>(() => {
    const customUiTheme = safeJsonParse<Record<string, string>>(appearanceSettings.value.customUiTheme, {});
    return { ...defaultUiTheme, ...customUiTheme };
  });

  const activeTerminalThemeId = computed<number | null>(() => {
    return appearanceSettings.value.activeTerminalThemeId ?? null;
  });

  const currentTerminalTheme = computed<ITheme>(() => {
    const activeId = activeTerminalThemeId.value;
    if (activeId !== null) {
      const activeTheme = allTerminalThemes.value.find(theme => theme._id === String(activeId));
      if (activeTheme) {
        return activeTheme.themeData;
      }
    }
    const firstTheme = allTerminalThemes.value[0];
    return firstTheme ? firstTheme.themeData : defaultXtermTheme;
  });

  const effectiveTerminalTheme = computed<ITheme>(() => currentTerminalTheme.value);

  const currentTerminalFontFamily = computed(() => appearanceSettings.value.terminalFontFamily ?? `Consolas, 'Courier New', monospace, 'Microsoft YaHei', '微软雅黑'`);
  const currentTerminalFontSize = computed(() => appearanceSettings.value.terminalFontSize ?? 14);
  const currentEditorFontSize = computed(() => appearanceSettings.value.editorFontSize ?? 14);
  const currentEditorFontFamily = computed(() => appearanceSettings.value.editorFontFamily ?? `Consolas, 'Noto Sans SC', 'Microsoft YaHei'`);
  const currentMobileEditorFontSize = computed(() => appearanceSettings.value.mobileEditorFontSize ?? 16);
  const terminalBackgroundImage = computed(() => appearanceSettings.value.terminalBackgroundImage ?? '');
  const isTerminalBackgroundEnabled = computed(() => appearanceSettings.value.terminalBackgroundEnabled ?? true);
  const currentTerminalBackgroundOverlayOpacity = computed(() => appearanceSettings.value.terminalBackgroundOverlayOpacity ?? 0.5);
  const terminalCustomHTML = computed(() => appearanceSettings.value.terminal_custom_html ?? '');

  const terminalTextStrokeEnabled = computed(() => appearanceSettings.value.terminalTextStrokeEnabled ?? false);
  const terminalTextStrokeWidth = computed(() => appearanceSettings.value.terminalTextStrokeWidth ?? 1);
  const terminalTextStrokeColor = computed(() => appearanceSettings.value.terminalTextStrokeColor ?? '#000000');
  const terminalTextShadowEnabled = computed(() => appearanceSettings.value.terminalTextShadowEnabled ?? false);
  const terminalTextShadowOffsetX = computed(() => appearanceSettings.value.terminalTextShadowOffsetX ?? 0);
  const terminalTextShadowOffsetY = computed(() => appearanceSettings.value.terminalTextShadowOffsetY ?? 0);
  const terminalTextShadowBlur = computed(() => appearanceSettings.value.terminalTextShadowBlur ?? 0);
  const terminalTextShadowColor = computed(() => appearanceSettings.value.terminalTextShadowColor ?? 'rgba(0,0,0,0.5)');

  const syncAppearanceSettings = () => {
    appearanceSettings.value = {
      customUiTheme: pickValue(APPEARANCE_ALIASES.customUiTheme, '{}'),
      activeTerminalThemeId: toInteger(pickValue(APPEARANCE_ALIASES.activeTerminalThemeId, undefined), null),
      terminalFontFamily: pickValue(APPEARANCE_ALIASES.terminalFontFamily, currentTerminalFontFamily.value),
      terminalFontSize: toNumber(pickValue(APPEARANCE_ALIASES.terminalFontSize, undefined), 14),
      editorFontSize: toNumber(pickValue(APPEARANCE_ALIASES.editorFontSize, undefined), 14),
      editorFontFamily: pickValue(APPEARANCE_ALIASES.editorFontFamily, currentEditorFontFamily.value),
      mobileEditorFontSize: toNumber(pickValue(APPEARANCE_ALIASES.mobileEditorFontSize, undefined), 16),
      terminalBackgroundEnabled: toBoolean(pickValue(APPEARANCE_ALIASES.terminalBackgroundEnabled, undefined), true),
      terminalBackgroundImage: pickValue(APPEARANCE_ALIASES.terminalBackgroundImage, ''),
      terminalBackgroundOverlayOpacity: toNumber(
        pickValue(APPEARANCE_ALIASES.terminalBackgroundOverlayOpacity, undefined),
        0.5,
      ),
      terminal_custom_html: pickValue(APPEARANCE_ALIASES.terminalCustomHtml, ''),
      terminalTextStrokeEnabled: toBoolean(pickValue(APPEARANCE_ALIASES.terminalTextStrokeEnabled, undefined), false),
      terminalTextStrokeWidth: toNumber(pickValue(APPEARANCE_ALIASES.terminalTextStrokeWidth, undefined), 1),
      terminalTextStrokeColor: pickValue(APPEARANCE_ALIASES.terminalTextStrokeColor, '#000000'),
      terminalTextShadowEnabled: toBoolean(pickValue(APPEARANCE_ALIASES.terminalTextShadowEnabled, undefined), false),
      terminalTextShadowOffsetX: toNumber(pickValue(APPEARANCE_ALIASES.terminalTextShadowOffsetX, undefined), 0),
      terminalTextShadowOffsetY: toNumber(pickValue(APPEARANCE_ALIASES.terminalTextShadowOffsetY, undefined), 0),
      terminalTextShadowBlur: toNumber(pickValue(APPEARANCE_ALIASES.terminalTextShadowBlur, undefined), 0),
      terminalTextShadowColor: pickValue(APPEARANCE_ALIASES.terminalTextShadowColor, 'rgba(0,0,0,0.5)'),
    };
  };

  const applyUiTheme = (theme: Record<string, string>) => {
    const root = document.documentElement;
    Object.entries(theme).forEach(([key, value]) => {
      root.style.setProperty(key, value);
    });
  };

  const applyTheme = () => {
    const mergedTheme = {
      ...currentUiTheme.value,
    };

    Object.entries(appearance.value).forEach(([key, value]) => {
      if (key.startsWith('--')) {
        mergedTheme[key] = value;
      }
    });

    const semanticAliases: Record<string, string> = {
      '--bg-base': 'var(--app-bg-color)',
      '--bg-mantle': 'var(--header-bg-color)',
      '--bg-surface0': 'var(--app-bg-color)',
      '--bg-surface1': 'var(--header-bg-color)',
      '--text': 'var(--text-color)',
      '--text-sub': 'var(--text-color-secondary)',
      '--text-dim': 'var(--text-color-secondary)',
      '--blue': 'var(--link-active-color)',
      '--red': 'var(--color-error)',
      '--green': 'var(--color-success)',
      '--yellow': 'var(--color-warning)',
      '--mauve': 'var(--link-hover-color)',
      '--border': 'var(--border-color)',
      '--ui-dialog-bg': 'var(--app-bg-color)',
      '--ui-item-bg': 'var(--header-bg-color)',
      '--ui-chip-action-bg': 'var(--header-bg-color)',
      '--ui-control-bg': 'var(--header-bg-color)',
      '--ui-handle-bg': 'var(--footer-bg-color)',
      '--ui-switch-off': 'var(--border-color)',
      '--ui-preview-bg': 'var(--app-bg-color)',
      '--ui-sidebar-bg': 'var(--app-bg-color)',
      '--ui-footer-bg': 'var(--footer-bg-color)',
      '--ui-btn-primary-bg': 'var(--header-bg-color)',
      '--ui-btn-primary-border': 'var(--border-color)',
      '--ui-btn-primary-text': 'var(--text-color)',
      '--ui-btn-primary-hover': 'var(--link-active-bg-color)',
      '--ui-menu-bg': 'var(--app-bg-color)',
      '--ui-menu-border': 'var(--border-color)',
      '--ui-menu-hover': 'var(--link-active-bg-color)',
    };

    applyUiTheme({
      ...mergedTheme,
      ...semanticAliases,
    });
  };

  const persistSingle = async (key: string, value: string) => {
    await settingsApi.appearanceSet(key, value);
    appearance.value[key] = value;
  };

  const persistAliases = async (keys: readonly string[], value: string) => {
    for (const key of keys) {
      await persistSingle(key, value);
    }
    syncAppearanceSettings();
    applyTheme();
  };

  const normalizePresetThemeData = (themeData: Partial<ExtendedITheme>): ExtendedITheme => ({
    ...defaultXtermTheme,
    background: themeData.background ?? defaultXtermTheme.background,
    foreground: themeData.foreground ?? defaultXtermTheme.foreground,
    cursor: themeData.cursor ?? defaultXtermTheme.cursor,
    cursorAccent: themeData.cursorAccent ?? undefined,
    selectionBackground: themeData.selectionBackground ?? defaultXtermTheme.selectionBackground,
    selectionForeground: themeData.selectionForeground ?? undefined,
    selectionInactiveBackground: themeData.selectionInactiveBackground ?? undefined,
    black: themeData.black ?? defaultXtermTheme.black,
    red: themeData.red ?? defaultXtermTheme.red,
    green: themeData.green ?? defaultXtermTheme.green,
    yellow: themeData.yellow ?? defaultXtermTheme.yellow,
    blue: themeData.blue ?? defaultXtermTheme.blue,
    magenta: themeData.magenta ?? defaultXtermTheme.magenta,
    cyan: themeData.cyan ?? defaultXtermTheme.cyan,
    white: themeData.white ?? defaultXtermTheme.white,
    brightBlack: themeData.brightBlack ?? defaultXtermTheme.brightBlack,
    brightRed: themeData.brightRed ?? defaultXtermTheme.brightRed,
    brightGreen: themeData.brightGreen ?? defaultXtermTheme.brightGreen,
    brightYellow: themeData.brightYellow ?? defaultXtermTheme.brightYellow,
    brightBlue: themeData.brightBlue ?? defaultXtermTheme.brightBlue,
    brightMagenta: themeData.brightMagenta ?? defaultXtermTheme.brightMagenta,
    brightCyan: themeData.brightCyan ?? defaultXtermTheme.brightCyan,
    brightWhite: themeData.brightWhite ?? defaultXtermTheme.brightWhite,
  });

  const syncBuiltinTerminalThemes = async (themes: ApiTerminalTheme[]): Promise<ApiTerminalTheme[]> => {
    const existingThemeNames = new Set(
      themes
        .map(theme => theme.name?.trim())
        .filter((name): name is string => !!name),
    );

    const missingPresets = BUILTIN_TERMINAL_THEME_PRESETS.filter(
      preset => !existingThemeNames.has(preset.name.trim()),
    );

    if (missingPresets.length === 0) {
      return themes;
    }

    let hasCreatedTheme = false;

    for (const preset of missingPresets) {
      try {
        const normalizedTheme = normalizePresetThemeData(preset.themeData);
        await settingsApi.themeCreate(toApiThemePayload(0, preset.name, normalizedTheme, 'preset'));
        hasCreatedTheme = true;
      } catch (error) {
        console.warn(`[AppearanceStore] 同步终端预设失败: ${preset.name}`, error);
      }
    }

    if (!hasCreatedTheme) {
      return themes;
    }

    return await settingsApi.themeList();
  };

  const loadTerminalThemes = async () => {
    let themes = await settingsApi.themeList();
    if (themes.length === 0) {
      await settingsApi.themeCreate(toApiThemePayload(0, '默认', defaultXtermTheme, 'preset'));
      themes = await settingsApi.themeList();
    }

    themes = await syncBuiltinTerminalThemes(themes);

    allTerminalThemes.value = themes.map(theme => ({
      _id: String(theme.id),
      name: theme.name,
      themeData: backendThemeToITheme(theme),
      isPreset: theme.theme_type === 'preset' || theme.id === 1,
    }));

    if (allTerminalThemes.value.length === 0) {
      allTerminalThemes.value = [{
        _id: '1',
        name: '默认',
        themeData: { ...defaultXtermTheme },
        isPreset: true,
      }];
    }

    const hasActive = activeTerminalThemeId.value !== null
      && allTerminalThemes.value.some(theme => theme._id === String(activeTerminalThemeId.value));
    if (!hasActive && allTerminalThemes.value[0]?._id) {
      await persistAliases(APPEARANCE_ALIASES.activeTerminalThemeId, allTerminalThemes.value[0]._id);
    }
  };

  async function loadAll() {
    const items = await settingsApi.appearanceGetAll();
    const map: Record<string, string> = {};
    for (const item of items) {
      map[item.key] = item.value;
    }
    appearance.value = map;
    syncAppearanceSettings();
    applyTheme();
    await loadTerminalThemes();
    await fetchLocalHtmlPresets();
    await fetchRemoteHtmlPresetsRepositoryUrl();
    loaded.value = true;
  }

  async function loadInitialAppearanceData() {
    await loadAll();
  }

  async function set(key: string, value: string) {
    await persistSingle(key, value);
    syncAppearanceSettings();
    applyTheme();
  }

  function get(key: string, fallback = ''): string {
    return appearance.value[key] ?? fallback;
  }

  async function saveCustomUiTheme(uiTheme: Record<string, string>) {
    const themeJson = JSON.stringify(uiTheme ?? {});
    await persistAliases(APPEARANCE_ALIASES.customUiTheme, themeJson);
  }

  async function resetCustomUiTheme() {
    await persistAliases(APPEARANCE_ALIASES.customUiTheme, '{}');
  }

  async function setActiveTerminalTheme(themeId: string | number) {
    const normalized = String(themeId);
    await persistAliases(APPEARANCE_ALIASES.activeTerminalThemeId, normalized);
  }

  async function setTerminalFontFamily(fontFamily: string) {
    await persistAliases(APPEARANCE_ALIASES.terminalFontFamily, fontFamily);
  }

  async function setTerminalFontSize(size: number) {
    await persistAliases(APPEARANCE_ALIASES.terminalFontSize, String(size));
  }

  async function setEditorFontSize(size: number) {
    await persistAliases(APPEARANCE_ALIASES.editorFontSize, String(size));
  }

  async function setEditorFontFamily(fontFamily: string) {
    await persistAliases(APPEARANCE_ALIASES.editorFontFamily, fontFamily);
  }

  async function setMobileEditorFontSize(size: number) {
    await persistAliases(APPEARANCE_ALIASES.mobileEditorFontSize, String(size));
  }

  async function setTerminalBackgroundEnabled(enabled: boolean) {
    await persistAliases(APPEARANCE_ALIASES.terminalBackgroundEnabled, String(enabled));
  }

  async function setTerminalBackgroundOverlayOpacity(opacity: number) {
    const normalized = Math.min(1, Math.max(0, opacity));
    await persistAliases(APPEARANCE_ALIASES.terminalBackgroundOverlayOpacity, normalized.toFixed(2));
  }

  async function setTerminalCustomHTML(html: string | null) {
    await persistAliases(APPEARANCE_ALIASES.terminalCustomHtml, html ?? '');
  }

  async function setTerminalTextStrokeEnabled(enabled: boolean) {
    await persistAliases(APPEARANCE_ALIASES.terminalTextStrokeEnabled, String(enabled));
  }

  async function setTerminalTextStrokeWidth(width: number) {
    await persistAliases(APPEARANCE_ALIASES.terminalTextStrokeWidth, String(width));
  }

  async function setTerminalTextStrokeColor(color: string) {
    await persistAliases(APPEARANCE_ALIASES.terminalTextStrokeColor, color);
  }

  async function setTerminalTextShadowEnabled(enabled: boolean) {
    await persistAliases(APPEARANCE_ALIASES.terminalTextShadowEnabled, String(enabled));
  }

  async function setTerminalTextShadowOffsetX(offset: number) {
    await persistAliases(APPEARANCE_ALIASES.terminalTextShadowOffsetX, String(offset));
  }

  async function setTerminalTextShadowOffsetY(offset: number) {
    await persistAliases(APPEARANCE_ALIASES.terminalTextShadowOffsetY, String(offset));
  }

  async function setTerminalTextShadowBlur(blur: number) {
    await persistAliases(APPEARANCE_ALIASES.terminalTextShadowBlur, String(blur));
  }

  async function setTerminalTextShadowColor(color: string) {
    await persistAliases(APPEARANCE_ALIASES.terminalTextShadowColor, color);
  }

  async function createTerminalTheme(name: string, themeData: ITheme) {
    await settingsApi.themeCreate(toApiThemePayload(0, name, themeData, 'user'));
    await loadTerminalThemes();
  }

  async function updateTerminalTheme(id: string, name: string, themeData: ITheme) {
    const idNum = Number.parseInt(id, 10);
    if (!Number.isFinite(idNum)) {
      throw new Error('主题 ID 无效');
    }
    await settingsApi.themeUpdate(toApiThemePayload(idNum, name, themeData, 'user'));
    await loadTerminalThemes();
  }

  async function deleteTerminalTheme(id: string) {
    const idNum = Number.parseInt(id, 10);
    if (!Number.isFinite(idNum)) {
      throw new Error('主题 ID 无效');
    }

    await settingsApi.themeDelete(idNum);
    await loadTerminalThemes();

    if (activeTerminalThemeId.value === idNum && allTerminalThemes.value[0]?._id) {
      await setActiveTerminalTheme(allTerminalThemes.value[0]._id);
    }
  }

  async function loadTerminalThemeData(themeId: string): Promise<ITheme | null> {
    const existing = allTerminalThemes.value.find(theme => theme._id === themeId);
    if (existing) {
      return { ...existing.themeData };
    }
    const idNum = Number.parseInt(themeId, 10);
    if (!Number.isFinite(idNum)) {
      return null;
    }
    const theme = await settingsApi.themeGet(idNum);
    return backendThemeToITheme(theme);
  }

  async function uploadTerminalBackground(file: File): Promise<string> {
    const dataUrl = await readFileAsDataUrl(file);
    await persistAliases(APPEARANCE_ALIASES.terminalBackgroundImage, dataUrl);
    return dataUrl;
  }

  async function removeTerminalBackground() {
    await persistAliases(APPEARANCE_ALIASES.terminalBackgroundImage, '');
  }

  const readCustomPresets = (): Record<string, string> => {
    try {
      const raw = localStorage.getItem(LOCAL_PRESETS_STORAGE_KEY);
      if (!raw) {
        return {};
      }
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed === 'object') {
        const output: Record<string, string> = {};
        Object.entries(parsed).forEach(([name, content]) => {
          output[normalizeName(name)] = String(content ?? '');
        });
        return output;
      }
      return {};
    } catch {
      return {};
    }
  };

  const writeCustomPresets = (presetMap: Record<string, string>) => {
    localStorage.setItem(LOCAL_PRESETS_STORAGE_KEY, JSON.stringify(presetMap));
  };

  async function fetchLocalHtmlPresets() {
    const customPresets = readCustomPresets();
    const builtins = Object.keys(BUILTIN_HTML_PRESETS).map(name => ({ name, type: 'preset' as const }));
    const customs = Object.keys(customPresets).map(name => ({ name, type: 'custom' as const }));
    localHtmlPresets.value = [...builtins, ...customs].sort((a, b) => a.name.localeCompare(b.name));
  }

  async function getLocalHtmlPresetContent(name: string): Promise<string> {
    const normalizedName = normalizeName(name);
    if (Object.prototype.hasOwnProperty.call(BUILTIN_HTML_PRESETS, normalizedName)) {
      return BUILTIN_HTML_PRESETS[normalizedName];
    }
    const customPresets = readCustomPresets();
    if (!Object.prototype.hasOwnProperty.call(customPresets, normalizedName)) {
      throw new Error('未找到本地主题内容');
    }
    return customPresets[normalizedName];
  }

  async function createLocalHtmlPreset(name: string, content: string) {
    const normalizedName = normalizeName(name);
    if (Object.prototype.hasOwnProperty.call(BUILTIN_HTML_PRESETS, normalizedName)) {
      throw new Error('预设主题不可覆盖');
    }
    const customPresets = readCustomPresets();
    if (Object.prototype.hasOwnProperty.call(customPresets, normalizedName)) {
      throw new Error('主题名称已存在');
    }
    customPresets[normalizedName] = content;
    writeCustomPresets(customPresets);
    await fetchLocalHtmlPresets();
  }

  async function updateLocalHtmlPreset(name: string, content: string) {
    const normalizedName = normalizeName(name);
    if (Object.prototype.hasOwnProperty.call(BUILTIN_HTML_PRESETS, normalizedName)) {
      throw new Error('预设主题不可修改');
    }
    const customPresets = readCustomPresets();
    if (!Object.prototype.hasOwnProperty.call(customPresets, normalizedName)) {
      throw new Error('未找到要更新的主题');
    }
    customPresets[normalizedName] = content;
    writeCustomPresets(customPresets);
    await fetchLocalHtmlPresets();
  }

  async function deleteLocalHtmlPreset(name: string) {
    const normalizedName = normalizeName(name);
    if (Object.prototype.hasOwnProperty.call(BUILTIN_HTML_PRESETS, normalizedName)) {
      throw new Error('预设主题不可删除');
    }
    const customPresets = readCustomPresets();
    delete customPresets[normalizedName];
    writeCustomPresets(customPresets);
    await fetchLocalHtmlPresets();
  }

  async function fetchRemoteHtmlPresetsRepositoryUrl() {
    const stored = localStorage.getItem(REMOTE_REPO_STORAGE_KEY)?.trim() ?? '';

    if (!stored || stored === LEGACY_REMOTE_REPO_URL) {
      remoteHtmlPresetsRepositoryUrl.value = DEFAULT_REMOTE_REPO_URL;
      localStorage.setItem(REMOTE_REPO_STORAGE_KEY, DEFAULT_REMOTE_REPO_URL);
      return remoteHtmlPresetsRepositoryUrl.value;
    }

    remoteHtmlPresetsRepositoryUrl.value = stored;
    return remoteHtmlPresetsRepositoryUrl.value;
  }

  async function updateRemoteHtmlPresetsRepositoryUrl(url: string) {
    const normalized = url.trim();
    if (!normalized) {
      localStorage.removeItem(REMOTE_REPO_STORAGE_KEY);
      remoteHtmlPresetsRepositoryUrl.value = '';
      remoteHtmlPresets.value = [];
      return;
    }

    localStorage.setItem(REMOTE_REPO_STORAGE_KEY, normalized);
    remoteHtmlPresetsRepositoryUrl.value = normalized;
  }

  async function fetchRemoteHtmlPresets(repoUrlParam?: string) {
    const repoUrl = (repoUrlParam ?? remoteHtmlPresetsRepositoryUrl.value ?? '').trim();
    remoteHtmlPresets.value = [];
    htmlPresetError.value = null;

    if (!repoUrl) {
      return;
    }

    const parsedGitHubUrl = parseGitHubTreeUrl(repoUrl);
    if (!parsedGitHubUrl) {
      htmlPresetError.value = '远程主题仓库地址格式无效';
      return;
    }

    isLoadingHtmlPresets.value = true;
    try {
      const { owner, repo, branch, path } = parsedGitHubUrl;
      const apiUrl = `https://api.github.com/repos/${owner}/${repo}/contents/${path}?ref=${branch}`;
      const response = await fetch(apiUrl, {
        headers: {
          Accept: 'application/vnd.github+json',
        },
      });

      if (!response.ok) {
        throw new Error(`GitHub API 请求失败(${response.status})`);
      }

      const contentItems = await response.json() as Array<{
        type?: string;
        name?: string;
        download_url?: string | null;
      }>;

      const htmlItems = contentItems
        .filter(item => item.type === 'file' && !!item.name && item.name.toLowerCase().endsWith('.html'))
        .map(item => ({
          name: item.name ?? '',
          downloadUrl: item.download_url ?? undefined,
        }))
        .sort((a, b) => a.name.localeCompare(b.name));

      remoteHtmlPresets.value = htmlItems;
      if (htmlItems.length === 0) {
        htmlPresetError.value = '远程仓库中未找到 HTML 主题';
      }
    } catch (error: any) {
      htmlPresetError.value = error?.message ?? '加载远程主题失败';
    } finally {
      isLoadingHtmlPresets.value = false;
    }
  }

  async function getRemoteHtmlPresetContent(fileUrl: string): Promise<string> {
    const response = await fetch(fileUrl);
    if (!response.ok) {
      throw new Error(`下载远程主题失败(${response.status})`);
    }
    return await response.text();
  }

  async function applyHtmlPreset(htmlContent: string) {
    await setTerminalCustomHTML(htmlContent);
  }

  function toggleStyleCustomizer(visible?: boolean) {
    isStyleCustomizerVisible.value = visible === undefined ? !isStyleCustomizerVisible.value : visible;
  }

  return {
    appearance,
    loaded,
    appearanceSettings,
    allTerminalThemes,
    currentUiTheme,
    currentTerminalTheme,
    effectiveTerminalTheme,
    activeTerminalThemeId,
    currentTerminalFontFamily,
    currentTerminalFontSize,
    currentEditorFontSize,
    currentEditorFontFamily,
    currentMobileEditorFontSize,
    terminalBackgroundImage,
    isTerminalBackgroundEnabled,
    currentTerminalBackgroundOverlayOpacity,
    terminalCustomHTML,
    terminalTextStrokeEnabled,
    terminalTextStrokeWidth,
    terminalTextStrokeColor,
    terminalTextShadowEnabled,
    terminalTextShadowOffsetX,
    terminalTextShadowOffsetY,
    terminalTextShadowBlur,
    terminalTextShadowColor,
    localHtmlPresets,
    remoteHtmlPresets,
    remoteHtmlPresetsRepositoryUrl,
    activeHtmlPresetTab,
    isLoadingHtmlPresets,
    htmlPresetError,
    isStyleCustomizerVisible,
    loadAll,
    loadInitialAppearanceData,
    set,
    get,
    applyTheme,
    saveCustomUiTheme,
    resetCustomUiTheme,
    setActiveTerminalTheme,
    setTerminalFontFamily,
    setTerminalFontSize,
    setEditorFontSize,
    setEditorFontFamily,
    setMobileEditorFontSize,
    setTerminalBackgroundEnabled,
    setTerminalBackgroundOverlayOpacity,
    setTerminalCustomHTML,
    setTerminalTextStrokeEnabled,
    setTerminalTextStrokeWidth,
    setTerminalTextStrokeColor,
    setTerminalTextShadowEnabled,
    setTerminalTextShadowOffsetX,
    setTerminalTextShadowOffsetY,
    setTerminalTextShadowBlur,
    setTerminalTextShadowColor,
    createTerminalTheme,
    updateTerminalTheme,
    deleteTerminalTheme,
    loadTerminalThemeData,
    uploadTerminalBackground,
    removeTerminalBackground,
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
    toggleStyleCustomizer,
  };
});
