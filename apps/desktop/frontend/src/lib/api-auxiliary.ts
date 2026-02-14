/**
 * Auxiliary APIs — matches backend Tauri command signatures.
 *
 * Backend signatures:
 *   audit_log_list(req: PaginationRequest { limit, offset })
 *   audit_log_count() / audit_log_clear()
 *   command_history_list(req: PaginationRequest { limit, offset })
 *   command_history_clear()
 *   favorite_path_list(connection_id: Option<i64>)
 *   favorite_path_create(req: FavoritePathRequest { name, path, connection_id })
 *   favorite_path_delete(id: i64)
 *   quick_command_list() / quick_command_get(id: i64)
 *   quick_command_create(input: QuickCommandInput)
 *   quick_command_update(id: i64, input: QuickCommandInput)
 *   quick_command_delete(id: i64) / quick_command_use(id: i64)
 */
import { tauriInvoke } from './invoke';

export interface AuditLog {
  id: number;
  timestamp: string;
  action_type: string;
  details?: string;
}

export interface CommandHistory {
  id: number;
  command: string;
  session_id?: string;
  connection_id?: number;
  timestamp: string;
}

export interface FavoritePath {
  id: number;
  name: string;
  path: string;
  connection_id?: number;
  last_used_at?: string;
}

export interface QuickCommand {
  id: number;
  name: string;
  command: string;
  description?: string;
  usage_count: number;
  variables?: string;
  tags: string[];
}

export interface PathHistory {
  id: number;
  path: string;
  connection_id?: number;
  timestamp: string;
}

export interface QuickCommandTag {
  id: number;
  name: string;
}

export const auditApi = {
  list: (limit?: number, offset?: number) =>
    tauriInvoke<AuditLog[]>('audit_log_list', { req: { limit, offset } }),
  count: () => tauriInvoke<number>('audit_log_count'),
  clear: () => tauriInvoke<void>('audit_log_clear'),
};

export const historyApi = {
  list: (limit?: number, offset?: number) =>
    tauriInvoke<CommandHistory[]>('command_history_list', { req: { limit, offset } }),
  add: (command: string, sessionId?: string, connectionId?: number) =>
    tauriInvoke<number>('command_history_add', { req: { command, session_id: sessionId, connection_id: connectionId } }),
  clear: () => tauriInvoke<void>('command_history_clear'),
};

export const pathHistoryApi = {
  list: (connectionId?: number, limit?: number) =>
    tauriInvoke<PathHistory[]>('path_history_list', { req: { connection_id: connectionId, limit } }),
  add: (path: string, connectionId?: number) =>
    tauriInvoke<number>('path_history_add', { req: { path, connection_id: connectionId } }),
  clear: () => tauriInvoke<void>('path_history_clear'),
};

export const favoritePathApi = {
  list: (connectionId?: number) =>
    tauriInvoke<FavoritePath[]>('favorite_path_list', { connection_id: connectionId }),
  create: (name: string, path: string, connectionId?: number) =>
    tauriInvoke<number>('favorite_path_create', {
      req: { name, path, connection_id: connectionId },
    }),
  update: (id: number, name: string, path: string, connectionId?: number) =>
    tauriInvoke<boolean>('favorite_path_update', {
      req: { id, name, path, connection_id: connectionId },
    }),
  delete: (id: number) => tauriInvoke<boolean>('favorite_path_delete', { id }),
};

export const quickCommandApi = {
  list: () => tauriInvoke<QuickCommand[]>('quick_command_list'),
  get: (id: number) => tauriInvoke<QuickCommand>('quick_command_get', { id }),
  create: (data: { name: string; command: string; variables?: string; tags?: string[] }) =>
    tauriInvoke<number>('quick_command_create', { input: data }),
  update: (id: number, data: { name: string; command: string; variables?: string; tags?: string[] }) =>
    tauriInvoke<boolean>('quick_command_update', { id, input: data }),
  delete: (id: number) => tauriInvoke<boolean>('quick_command_delete', { id }),
  use: (id: number) => tauriInvoke<void>('quick_command_use', { id }),
};

export const quickCommandTagApi = {
  list: () => tauriInvoke<QuickCommandTag[]>('quick_command_tag_list'),
  create: (name: string) =>
    tauriInvoke<number>('quick_command_tag_create', { req: { name } }),
  delete: (id: number) => tauriInvoke<boolean>('quick_command_tag_delete', { id }),
  bulkAssign: (tagId: number, quickCommandIds: number[]) =>
    tauriInvoke<void>('quick_command_bulk_assign_tag', { req: { tag_id: tagId, quick_command_ids: quickCommandIds } }),
};
