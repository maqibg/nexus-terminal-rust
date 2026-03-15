import { computed, ref } from 'vue';
import { databaseApi, type RedisCommandResult, type RedisConnection, type RedisKeyDetail, type RedisKeyInfo } from '@/lib/api-database';
import { useSettingsStore } from '@/stores/settings';
import { useUINotificationStore } from '@/stores/uiNotifications';
export type ConnectionAsset<TConn> = { id: string; name: string; conn: TConn; encryptedPassword?: string };
export type RedisConn = Omit<RedisConnection, 'password' | 'encrypted_password'>;
const SETTINGS_KEY = 'dbManager.redisConnections';
function parseAssetList<TConn extends Record<string, unknown>>(raw: string): ConnectionAsset<TConn>[] {
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed
      .filter((item): item is ConnectionAsset<TConn> => !!item && typeof item === 'object')
      .filter(item => typeof item.id === 'string' && typeof item.name === 'string' && !!item.conn && typeof item.conn === 'object')
      .map(item => ({
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

function parseArgs(input: string): string[] {
  const text = input.trim();
  if (!text) {
    return [];
  }
  const args: string[] = [];
  let current = '';
  let quote: '"' | "'" | null = null;
  let escaped = false;
  for (const ch of text) {
    if (escaped) {
      current += ch;
      escaped = false;
      continue;
    }
    if (ch === '\\') {
      escaped = true;
      continue;
    }
    if (quote) {
      if (ch === quote) {
        quote = null;
      } else {
        current += ch;
      }
      continue;
    }
    if (ch === '"' || ch === "'") {
      quote = ch;
      continue;
    }
    if (/\s/.test(ch)) {
      if (current) {
        args.push(current);
        current = '';
      }
      continue;
    }
    current += ch;
  }
  if (current) {
    args.push(current);
  }
  return args;
}

export function useRedisManager() {
  const settingsStore = useSettingsStore();
  const notifications = useUINotificationStore();
  const connections = ref<ConnectionAsset<RedisConn>[]>([]);
  const activeConnectionId = ref<string | null>(null);
  const passwords = ref<Record<string, string>>({});
  const dbOverrides = ref<Record<string, number>>({});
  const scanPattern = ref('');
  const scanLimit = ref<number>(200);
  const keys = ref<RedisKeyInfo[]>([]);
  const keysLoading = ref(false);
  const activeKey = ref('');
  const keyDetail = ref<RedisKeyDetail | null>(null);
  const keyLoading = ref(false);

  const commandText = ref('PING');
  const commandLoading = ref(false);
  const commandError = ref('');
  const commandResult = ref<RedisCommandResult | null>(null);
  const activeConnection = computed(() => connections.value.find(c => c.id === activeConnectionId.value) ?? null);

  const activeDb = computed<number>({
    get() {
      const id = activeConnectionId.value;
      if (!id) {
        return 0;
      }
      const override = dbOverrides.value[id];
      if (typeof override === 'number' && override >= 0) {
        return Math.round(override);
      }
      const base = activeConnection.value?.conn.db;
      return typeof base === 'number' && base >= 0 ? Math.round(base) : 0;
    },
    set(next) {
      const id = activeConnectionId.value;
      if (!id) {
        return;
      }
      dbOverrides.value = { ...dbOverrides.value, [id]: Math.max(0, Math.round(next || 0)) };
    },
  });

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

  async function persistConnections() {
    await ensureSettingsLoaded();
    await settingsStore.set(SETTINGS_KEY, JSON.stringify(connections.value));
  }

  function clearActiveContext() {
    keys.value = [];
    keysLoading.value = false;
    activeKey.value = '';
    keyDetail.value = null;
    keyLoading.value = false;
    commandError.value = '';
    commandResult.value = null;
  }

  function buildAuthConn(asset: ConnectionAsset<RedisConn>): RedisConnection {
    const password = activePassword.value.trim() || undefined;
    return {
      ...asset.conn,
      db: activeDb.value,
      password,
      encrypted_password: asset.encryptedPassword,
    };
  }

  function pruneRecord<T>(input: Record<string, T>, ids: Set<string>): Record<string, T> {
    const next: Record<string, T> = {};
    for (const [key, value] of Object.entries(input)) {
      if (ids.has(key)) {
        next[key] = value;
      }
    }
    return next;
  }

  async function loadConnections() {
    await ensureSettingsLoaded();
    connections.value = parseAssetList<RedisConn>(settingsStore.get(SETTINGS_KEY, '[]'));

    const ids = new Set(connections.value.map(c => c.id));
    passwords.value = pruneRecord(passwords.value, ids);
    dbOverrides.value = pruneRecord(dbOverrides.value, ids);

    const currentId = activeConnectionId.value;
    const hasCurrent = typeof currentId === 'string' && ids.has(currentId);
    if (!hasCurrent) {
      activeConnectionId.value = connections.value[0]?.id ?? null;
      clearActiveContext();
    }

    if (activeConnectionId.value) {
      await scanKeys();
    }
  }

  async function addConnection(asset: ConnectionAsset<RedisConn>, password: string) {
    connections.value = [asset, ...connections.value];
    activeConnectionId.value = asset.id;
    if (password) {
      passwords.value = { ...passwords.value, [asset.id]: password };
    }
    clearActiveContext();
    await persistConnections();
    await scanKeys();
  }

  async function updateConnection(asset: ConnectionAsset<RedisConn>, password?: string) {
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
      await scanKeys();
    }
  }

  async function selectConnection(id: string) {
    if (activeConnectionId.value === id) {
      return;
    }
    activeConnectionId.value = id;
    clearActiveContext();
    await scanKeys();
  }

  async function removeConnection(id: string) {
    const next = connections.value.filter(c => c.id !== id);
    connections.value = next;
    const { [id]: _db, ...restDb } = dbOverrides.value;
    dbOverrides.value = restDb;
    const { [id]: _pw, ...restPw } = passwords.value;
    passwords.value = restPw;

    if (activeConnectionId.value === id) {
      activeConnectionId.value = next[0]?.id ?? null;
      clearActiveContext();
      if (activeConnectionId.value) {
        await scanKeys();
      }
    }

    await persistConnections();
  }

  async function scanKeys() {
    const asset = activeConnection.value;
    if (!asset) {
      return;
    }
    keysLoading.value = true;
    try {
      const pattern = scanPattern.value.trim() || undefined;
      const limit = Math.min(20000, Math.max(1, Math.round(scanLimit.value || 200)));
      keys.value = await databaseApi.redisScanKeys(buildAuthConn(asset), { pattern, limit });
    } catch (err) {
      notifications.addNotification('error', err instanceof Error ? err.message : '加载 Key 失败');
      keys.value = [];
    } finally {
      keysLoading.value = false;
    }
  }

  async function selectKey(key: string) {
    activeKey.value = key;
    await loadKeyDetail(key);
  }

  async function loadKeyDetail(key: string) {
    const asset = activeConnection.value;
    if (!asset) {
      return;
    }
    keyLoading.value = true;
    try {
      keyDetail.value = await databaseApi.redisGetKey(buildAuthConn(asset), key, 400);
    } catch (err) {
      notifications.addNotification('error', err instanceof Error ? err.message : '加载 Key 失败');
      keyDetail.value = null;
    } finally {
      keyLoading.value = false;
    }
  }

  function insertGet(key: string) {
    commandText.value = `GET "${key.replaceAll('\"', '\\\\\"')}"`;
  }

  function clearCommand() {
    commandText.value = '';
    commandResult.value = null;
    commandError.value = '';
  }

  async function runCommand() {
    const asset = activeConnection.value;
    if (!asset) {
      return;
    }
    const args = parseArgs(commandText.value);
    if (args.length === 0) {
      commandError.value = '请输入命令';
      return;
    }
    commandLoading.value = true;
    commandError.value = '';
    commandResult.value = null;
    try {
      commandResult.value = await databaseApi.redisCommand(buildAuthConn(asset), args);
    } catch (err) {
      commandError.value = err instanceof Error ? err.message : '执行失败';
    } finally {
      commandLoading.value = false;
    }
  }

  return {
    connections,
    activeConnectionId,
    activeConnection,
    activeDb,
    activePassword,
    scanPattern,
    scanLimit,
    keys,
    keysLoading,
    activeKey,
    keyDetail,
    keyLoading,
    commandText,
    commandLoading,
    commandError,
    commandResult,
    loadConnections,
    addConnection,
    updateConnection,
    selectConnection,
    removeConnection,
    scanKeys,
    selectKey,
    insertGet,
    clearCommand,
    runCommand,
  };
}
