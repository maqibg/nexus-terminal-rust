import { computed, ref, shallowRef } from 'vue';

import type { DbColumn, DbQueryResult, DbTable } from '@/lib/api-database';
import { useSettingsStore } from '@/stores/settings';
import { useUINotificationStore } from '@/stores/uiNotifications';

export interface SqlConnectionAsset<TConn> {
  id: string;
  name: string;
  conn: TConn;
  encryptedPassword?: string;
}

export interface SqlLikeAuth {
  password?: string;
  encryptedPassword?: string;
}

export interface SqlLikeDriver<TConn> {
  settingsKey: string;
  buildSelectStar: (tableName: string, conn: TConn) => string;
  listTables: (conn: TConn, auth: SqlLikeAuth) => Promise<DbTable[]>;
  listColumns: (conn: TConn, tableName: string, auth: SqlLikeAuth) => Promise<DbColumn[]>;
  query: (conn: TConn, sql: string, auth: SqlLikeAuth) => Promise<DbQueryResult>;
}

function parseAssetList<TConn>(raw: string): SqlConnectionAsset<TConn>[] {
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return [];
    }

    return parsed
      .filter((item): item is SqlConnectionAsset<TConn> => !!item && typeof item === 'object')
      .filter(item => typeof item.id === 'string' && typeof item.name === 'string' && !!item.conn && typeof item.conn === 'object')
      .map((item) => ({
        id: item.id,
        name: item.name,
        conn: item.conn,
        encryptedPassword:
          typeof item.encryptedPassword === 'string'
            ? item.encryptedPassword
            : typeof (item as { encrypted_password?: unknown }).encrypted_password === 'string'
              ? String((item as { encrypted_password?: unknown }).encrypted_password)
              : undefined,
      }));
  } catch {
    return [];
  }
}

export function useSqlLikeManager<TConn extends Record<string, unknown>>(driver: SqlLikeDriver<TConn>) {
  const settingsStore = useSettingsStore();
  const notifications = useUINotificationStore();

  const connections = shallowRef<SqlConnectionAsset<TConn>[]>([]);
  const activeConnectionId = ref<string | null>(null);
  const activeTableName = ref<string | null>(null);

  const passwords = ref<Record<string, string>>({});

  const tables = ref<DbTable[]>([]);
  const tablesLoading = ref(false);
  const columns = ref<DbColumn[]>([]);
  const columnsLoading = ref(false);

  const sqlText = ref('');
  const queryLoading = ref(false);
  const queryError = ref('');
  const queryResult = ref<DbQueryResult | null>(null);

  const activeConnection = computed(() => connections.value.find(c => c.id === activeConnectionId.value) ?? null);

  const activePassword = computed<string>({
    get() {
      const id = activeConnectionId.value;
      return id ? passwords.value[id] ?? '' : '';
    },
    set(next) {
      const id = activeConnectionId.value;
      if (!id) {
        return;
      }
      passwords.value = { ...passwords.value, [id]: next };
    },
  });

  async function ensureSettingsLoaded() {
    if (!settingsStore.loaded) {
      await settingsStore.loadAll();
    }
  }

  function clearActiveContext() {
    activeTableName.value = null;
    tables.value = [];
    columns.value = [];
    sqlText.value = '';
    queryResult.value = null;
    queryError.value = '';
  }

  async function persistConnections() {
    await ensureSettingsLoaded();
    await settingsStore.set(driver.settingsKey, JSON.stringify(connections.value));
  }

  async function loadConnections() {
    await ensureSettingsLoaded();
    const raw = settingsStore.get(driver.settingsKey, '[]');
    connections.value = parseAssetList<TConn>(raw);

    const ids = new Set(connections.value.map(c => c.id));
    passwords.value = Object.fromEntries(Object.entries(passwords.value).filter(([id]) => ids.has(id)));

    const currentId = activeConnectionId.value;
    const hasCurrent = typeof currentId === 'string' && connections.value.some(c => c.id === currentId);
    if (!hasCurrent) {
      activeConnectionId.value = connections.value[0]?.id ?? null;
      clearActiveContext();
    }

    if (activeConnectionId.value) {
      await loadTables();
    }
  }

  async function addConnection(asset: SqlConnectionAsset<TConn>, password: string) {
    connections.value = [asset, ...connections.value];
    activeConnectionId.value = asset.id;
    if (password) {
      passwords.value = { ...passwords.value, [asset.id]: password };
    }
    clearActiveContext();
    await persistConnections();
    await loadTables();
  }

  async function updateConnection(asset: SqlConnectionAsset<TConn>, password?: string) {
    const idx = connections.value.findIndex(c => c.id === asset.id);
    if (idx < 0) {
      return;
    }

    const next = [...connections.value];
    next[idx] = asset;
    connections.value = next;

    if (password) {
      passwords.value = { ...passwords.value, [asset.id]: password };
    }

    await persistConnections();

    if (activeConnectionId.value === asset.id) {
      clearActiveContext();
      await loadTables();
    }
  }

  async function removeConnection(id: string) {
    const next = connections.value.filter(c => c.id !== id);
    connections.value = next;

    const { [id]: _pw, ...rest } = passwords.value;
    passwords.value = rest;

    if (activeConnectionId.value === id) {
      activeConnectionId.value = next[0]?.id ?? null;
      clearActiveContext();
      if (activeConnectionId.value) {
        await loadTables();
      }
    }

    await persistConnections();
  }

  async function selectConnection(id: string) {
    if (activeConnectionId.value === id) {
      return;
    }

    activeConnectionId.value = id;
    clearActiveContext();
    await loadTables();
  }

  async function loadTables() {
    const asset = activeConnection.value;
    if (!asset) {
      return;
    }

    tablesLoading.value = true;
    try {
      const auth: SqlLikeAuth = {
        password: activePassword.value.trim() || undefined,
        encryptedPassword: asset.encryptedPassword,
      };
      tables.value = await driver.listTables(asset.conn, auth);
    } catch (err) {
      notifications.addNotification('error', err instanceof Error ? err.message : '加载表失败');
      tables.value = [];
    } finally {
      tablesLoading.value = false;
    }
  }

  async function selectTable(name: string) {
    activeTableName.value = name;
    await loadColumns(name);
  }

  async function loadColumns(tableName: string) {
    const asset = activeConnection.value;
    if (!asset) {
      return;
    }

    columnsLoading.value = true;
    try {
      const auth: SqlLikeAuth = {
        password: activePassword.value.trim() || undefined,
        encryptedPassword: asset.encryptedPassword,
      };
      columns.value = await driver.listColumns(asset.conn, tableName, auth);
    } catch (err) {
      notifications.addNotification('error', err instanceof Error ? err.message : '加载列失败');
      columns.value = [];
    } finally {
      columnsLoading.value = false;
    }
  }

  function insertSelectStar(tableName: string) {
    const asset = activeConnection.value;
    if (!asset) {
      return;
    }
    sqlText.value = driver.buildSelectStar(tableName, asset.conn);
  }

  function clearQuery() {
    sqlText.value = '';
    queryResult.value = null;
    queryError.value = '';
  }

  async function runQuery() {
    const asset = activeConnection.value;
    if (!asset) {
      return;
    }

    const sql = sqlText.value.trim();
    if (!sql) {
      queryError.value = '请输入 SQL';
      return;
    }

    queryLoading.value = true;
    queryError.value = '';
    queryResult.value = null;

    try {
      const auth: SqlLikeAuth = {
        password: activePassword.value.trim() || undefined,
        encryptedPassword: asset.encryptedPassword,
      };
      queryResult.value = await driver.query(asset.conn, sql, auth);
    } catch (err) {
      queryError.value = err instanceof Error ? err.message : '执行失败';
    } finally {
      queryLoading.value = false;
    }
  }

  function formatCell(cell: unknown): string {
    if (cell === null || cell === undefined) {
      return '';
    }
    if (typeof cell === 'string') {
      return cell;
    }
    if (typeof cell === 'number' || typeof cell === 'boolean') {
      return String(cell);
    }
    try {
      return JSON.stringify(cell);
    } catch {
      return String(cell);
    }
  }

  return {
    connections,
    activeConnectionId,
    activeConnection,
    activePassword,
    activeTableName,
    tables,
    tablesLoading,
    columns,
    columnsLoading,
    sqlText,
    queryLoading,
    queryError,
    queryResult,
    loadConnections,
    addConnection,
    updateConnection,
    selectConnection,
    removeConnection,
    loadTables,
    selectTable,
    loadColumns,
    insertSelectStar,
    clearQuery,
    runQuery,
    formatCell,
  };
}
