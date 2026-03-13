/**
 * Barrel export — single import point for all Tauri API adapters.
 */
export { tauriInvoke } from './invoke';
export { authApi } from './api-auth';
export type { AuthState, TwoFactorSetupResponse, PasskeyInfo } from './api-auth';
export { connectionsApi } from './api-connections';
export type { Connection, Tag, SshKey, Proxy, ImportResult, ResetDataRequest, ResetDataResult } from './api-connections';
export { sshApi, onSshOutput } from './api-ssh';
export type { SshSession, SshOutputChunk, HostKeyEntry } from './api-ssh';
export { sftpApi } from './api-sftp';
export type { FileEntry } from './api-sftp';
export { desktopApi } from './api-desktop';
export type { OpenRdpPayload, OpenVncPayload, RdpSessionStatus, VncSessionInfo } from './api-desktop';
export { transferApi } from './api-transfer';
export type { TransferTask, TransferTaskDto } from './api-transfer';
export { statusApi } from './api-status';
export type { BackendHealth, RuntimePaths } from './api-status';
export { settingsApi } from './api-settings';
export type { Setting, TerminalTheme, NotificationChannel } from './api-settings';
export { aiApi } from './api-ai';
export { auditApi, historyApi, pathHistoryApi, favoritePathApi, quickCommandApi, quickCommandTagApi } from './api-auxiliary';
export type { AuditLog, CommandHistory, PathHistory, FavoritePath, QuickCommand, QuickCommandTag } from './api-auxiliary';
export { sshSuspendApi } from './api-ssh-suspend';
export type { SuspendedSession } from './api-ssh-suspend';




