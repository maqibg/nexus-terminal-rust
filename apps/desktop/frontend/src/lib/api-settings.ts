/**
 * Settings API — matches backend Tauri command signatures.
 *
 * Backend signatures:
 *   settings_get_all() / settings_set(req: SetSettingRequest { key, value })
 *   appearance_get_all() / appearance_set(req: SetSettingRequest { key, value })
 *   theme_list() / theme_get(id: i64) / theme_create(theme) / theme_update(theme) / theme_delete(id: i64)
 *   notification_channel_list()
 */
import { tauriInvoke } from './invoke';

export interface Setting {
  key: string;
  value: string;
}

export interface TerminalTheme {
  id: number;
  name: string;
  theme_type: string;
  background: string;
  foreground: string;
  cursor: string;
  selection_background: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  bright_black: string;
  bright_red: string;
  bright_green: string;
  bright_yellow: string;
  bright_blue: string;
  bright_magenta: string;
  bright_cyan: string;
  bright_white: string;
}

export interface NotificationChannel {
  id: number;
  name: string;
  channel_type: string;
  config: string;
  enabled: boolean;
  enabled_events: string;
}

export const settingsApi = {
  getAll: () => tauriInvoke<Setting[]>('settings_get_all'),
  set: (key: string, value: string) =>
    tauriInvoke<void>('settings_set', { req: { key, value } }),

  appearanceGetAll: () => tauriInvoke<Setting[]>('appearance_get_all'),
  appearanceSet: (key: string, value: string) =>
    tauriInvoke<void>('appearance_set', { req: { key, value } }),

  themeList: () => tauriInvoke<TerminalTheme[]>('theme_list'),
  themeGet: (id: number) => tauriInvoke<TerminalTheme>('theme_get', { id }),
  themeCreate: (theme: Record<string, unknown>) =>
    tauriInvoke<number>('theme_create', { theme }),
  themeUpdate: (theme: Record<string, unknown>) =>
    tauriInvoke<boolean>('theme_update', { theme }),
  themeDelete: (id: number) => tauriInvoke<boolean>('theme_delete', { id }),

  notificationChannelList: () =>
    tauriInvoke<NotificationChannel[]>('notification_channel_list'),
  notificationChannelCreate: (channel: Record<string, unknown>) =>
    tauriInvoke<number>('notification_channel_create', { channel }),
  notificationChannelUpdate: (channel: Record<string, unknown>) =>
    tauriInvoke<boolean>('notification_channel_update', { channel }),
  notificationChannelDelete: (id: number) =>
    tauriInvoke<boolean>('notification_channel_delete', { id }),
};
