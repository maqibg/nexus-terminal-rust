import { computed, ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';

import { databaseApi, type SqliteColumn, type SqliteQueryResult, type SqliteTable } from '@/lib/api-database';
import { useSettingsStore } from '@/stores/settings';
import { useUINotificationStore } from '@/stores/uiNotifications';

interface SqliteConnectionAsset {
  id: string;
  name: string;
  path: string;
}

const SETTINGS_KEY = 'dbManager.sqliteConnections';
const SQLITE_FILE_FILTERS = [{ name: 'SQLite', extensions: ['db', 'sqlite', 'sqlite3'] }];
const SQLITE_DIALOG_TITLE = '选择 SQLite 数据库文件';
const DEFAULT_SELECT_LIMIT = 100;

function basename(path: string) {
  const cleaned = path.replaceAll('\\', '/');
  const parts = cleaned.split('/');
  return parts[parts.length - 1] || path;
}

function sqliteQuoteIdentifier(input: string) {
  return `"${input.replaceAll('"', '""')}"`;
}

export function useSqliteManager() {
  const settingsStore = useSettingsStore();
  const notifications = useUINotificationStore();

  const sqliteConnections = ref<SqliteConnectionAsset[]>([]);
  const activeSqliteId = ref<string | null>(null);
  const activeTableName = ref<string | null>(null);

  const tables = ref<SqliteTable[]>([]);
  const tablesLoading = ref(false);
  const columns = ref<SqliteColumn[]>([]);
  const columnsLoading = ref(false);

  const sqlText = ref('');
  const queryLoading = ref(false);
  const queryError = ref('');
  const queryResult = ref<SqliteQueryResult | null>(null);

  const activeSqlite = computed(() => sqliteConnections.value.find(c => c.id === activeSqliteId.value) ?? null);

  async function ensureSettingsLoaded() {
    if (!settingsStore.loaded) {
      await settingsStore.loadAll();
    }
  }

  async function loadConnections() {
    await ensureSettingsLoaded();

    const raw = settingsStore.get(SETTINGS_KEY, '[]');
    try {
      const parsed = JSON.parse(raw) as SqliteConnectionAsset[];
      sqliteConnections.value = Array.isArray(parsed) ? parsed : [];
    } catch {
      sqliteConnections.value = [];
    }

    const currentId = activeSqliteId.value;
    const hasCurrent = typeof currentId === 'string' && sqliteConnections.value.some(c => c.id === currentId);
    if (!hasCurrent) {
      activeSqliteId.value = sqliteConnections.value[0]?.id ?? null;
      clearActiveContext();
    }

    if (activeSqliteId.value) {
      await loadTables();
    }
  }

  async function persistConnections() {
    await ensureSettingsLoaded();
    await settingsStore.set(SETTINGS_KEY, JSON.stringify(sqliteConnections.value));
  }

  function clearActiveContext() {
    activeTableName.value = null;
    tables.value = [];
    columns.value = [];
    sqlText.value = '';
    queryResult.value = null;
    queryError.value = '';
  }

  async function addSqlite() {
    const selected = await open({
      multiple: false,
      title: SQLITE_DIALOG_TITLE,
      filters: SQLITE_FILE_FILTERS,
    });
    if (!selected || typeof selected !== 'string') {
      return;
    }

    const asset: SqliteConnectionAsset = {
      id: crypto.randomUUID(),
      name: basename(selected),
      path: selected,
    };

    sqliteConnections.value = [asset, ...sqliteConnections.value];
    activeSqliteId.value = asset.id;
    clearActiveContext();

    await persistConnections();
    await loadTables();
  }

  async function selectSqlite(id: string) {
    if (activeSqliteId.value === id) {
      return;
    }

    activeSqliteId.value = id;
    clearActiveContext();
    await loadTables();
  }

  async function removeSqlite(id: string) {
    const next = sqliteConnections.value.filter(c => c.id !== id);
    sqliteConnections.value = next;

    if (activeSqliteId.value === id) {
      activeSqliteId.value = next[0]?.id ?? null;
      clearActiveContext();
      if (activeSqliteId.value) {
        await loadTables();
      }
    }

    await persistConnections();
  }

  async function loadTables() {
    const conn = activeSqlite.value;
    if (!conn) {
      return;
    }

    tablesLoading.value = true;
    try {
      tables.value = await databaseApi.sqliteListTables(conn.path);
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

  async function loadColumns(table: string) {
    const conn = activeSqlite.value;
    if (!conn) {
      return;
    }

    columnsLoading.value = true;
    try {
      columns.value = await databaseApi.sqliteListColumns(conn.path, table);
    } catch (err) {
      notifications.addNotification('error', err instanceof Error ? err.message : '加载列失败');
      columns.value = [];
    } finally {
      columnsLoading.value = false;
    }
  }

  function insertSelectStar(table: string) {
    sqlText.value = `SELECT * FROM ${sqliteQuoteIdentifier(table)} LIMIT ${DEFAULT_SELECT_LIMIT};`;
  }

  function clearQuery() {
    sqlText.value = '';
    queryResult.value = null;
    queryError.value = '';
  }

  async function runQuery() {
    const conn = activeSqlite.value;
    if (!conn) {
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
      queryResult.value = await databaseApi.sqliteQuery(conn.path, sql);
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
    sqliteConnections,
    activeSqliteId,
    activeSqlite,
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
    addSqlite,
    selectSqlite,
    removeSqlite,
    loadTables,
    selectTable,
    loadColumns,
    insertSelectStar,
    clearQuery,
    runQuery,
    formatCell,
  };
}
