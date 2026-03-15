<template>
  <div class="db-layout">
    <aside class="db-sidebar">
      <div class="section-title">Oracle / ODBC</div>

      <DbConnectionList
        title="连接"
        :items="connectionItems"
        :active-id="activeConnectionId"
        @select="selectConnection"
        @create="openCreate"
        @edit="openEdit"
        @delete="handleDelete"
        @refresh="loadConnections"
      />

      <SqlLikeAssetsTree
        v-if="activeConnection"
        :tables="tables"
        :tables-loading="tablesLoading"
        :active-table-name="activeTableName"
        :columns="columns"
        :columns-loading="columnsLoading"
        @refreshTables="loadTables"
        @selectTable="selectTable"
        @refreshColumns="loadColumns"
      />
    </aside>

    <section class="db-main">
      <SqlLikeQueryPanel
        :has-connection="!!activeConnection"
        v-model:password="activePassword"
        v-model:sql="sqlText"
        :query-loading="queryLoading"
        :query-error="queryError"
        :query-result="queryResult"
        :active-table-name="activeTableName"
        :format-cell="formatCell"
        @runQuery="runQuery"
        @clearQuery="clearQuery"
        @insertSelectStar="insertSelectStar"
      />
    </section>
  </div>

  <DbConnectionDialog
    :visible="dialogVisible"
    :title="dialogTitle"
    :save-label="dialogSaveLabel"
    :busy="dialogBusy"
    :error="dialogError"
    @close="closeDialog"
    @save="saveDialog"
  >
    <div class="field-block">
      <label class="field-label">名称（可选）</label>
      <input v-model="formName" class="field-input" type="text" placeholder="例如：prod-oracle" />
    </div>

    <div class="field-block">
      <label class="field-label">ODBC 连接串</label>
      <input
        v-model="formConnectionString"
        class="field-input"
        type="text"
        placeholder="例如：DSN=MyOracle; 或 Driver={...};Dbq=...;"
      />
      <div class="hint">建议连接串不包含密码，密码请单独输入（可保存到本地）。</div>
    </div>

    <div class="field-grid">
      <div class="field-block">
        <label class="field-label">用户名（可选）</label>
        <input v-model="formUsername" class="field-input" type="text" placeholder="可留空" />
      </div>
      <div class="field-block">
        <label class="field-label">密码（保存到本地）</label>
        <input v-model="formPassword" class="field-input" type="password" placeholder="新建可留空；编辑留空保持" />
      </div>
    </div>

    <div v-if="dialogMode === 'edit'" class="field-row">
      <label class="checkbox">
        <input v-model="clearSavedPassword" type="checkbox" />
        <span>清除已保存密码</span>
      </label>
    </div>
  </DbConnectionDialog>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';

import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { cryptoApi } from '@/lib/api';
import { databaseApi, type OracleConnection } from '@/lib/api-database';
import { useSqlLikeManager, type SqlConnectionAsset, type SqlLikeDriver } from './useSqlLikeManager';
import DbConnectionList, { type DbConnListItem } from './components/DbConnectionList.vue';
import DbConnectionDialog from './components/DbConnectionDialog.vue';
import SqlLikeAssetsTree from './components/SqlLikeAssetsTree.vue';
import SqlLikeQueryPanel from './components/SqlLikeQueryPanel.vue';

type OracleConn = Omit<OracleConnection, 'password' | 'encrypted_password'>;

function oracleQuoteIdent(input: string) {
  return `"${input.replaceAll('"', '""')}"`;
}

function resolveConnectionString(conn: OracleConn): string {
  if (typeof conn.connectionString === 'string' && conn.connectionString.trim()) {
    return conn.connectionString;
  }
  const legacy = conn as unknown as { connection_string?: unknown };
  return typeof legacy.connection_string === 'string' ? legacy.connection_string : '';
}

const driver: SqlLikeDriver<OracleConn> = {
  settingsKey: 'dbManager.oracleConnections',
  buildSelectStar: (tableName) => `SELECT * FROM ${oracleQuoteIdent(tableName)} FETCH FIRST 100 ROWS ONLY;`,
  listTables: async (conn, auth) =>
    databaseApi.oracleListTables({
      ...conn,
      connectionString: resolveConnectionString(conn),
      password: auth.password,
      encrypted_password: auth.encryptedPassword,
    }),
  listColumns: async (conn, tableName, auth) =>
    databaseApi.oracleListColumns(
      {
        ...conn,
        connectionString: resolveConnectionString(conn),
        password: auth.password,
        encrypted_password: auth.encryptedPassword,
      },
      tableName,
    ),
  query: async (conn, sql, auth) =>
    databaseApi.oracleQuery(
      {
        ...conn,
        connectionString: resolveConnectionString(conn),
        password: auth.password,
        encrypted_password: auth.encryptedPassword,
      },
      sql,
    ),
};

const {
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
  selectConnection,
  loadTables,
  selectTable,
  loadColumns,
  insertSelectStar,
  clearQuery,
  runQuery,
  formatCell,
  updateConnection,
  removeConnection,
} = useSqlLikeManager<OracleConn>(driver);

const formName = ref('');
const formConnectionString = ref('');
const formUsername = ref('');
const formPassword = ref('');

const { confirm } = useConfirmDialog();

const connectionItems = computed<DbConnListItem[]>(() =>
  connections.value.map((asset) => ({
    id: asset.id,
    name: asset.name,
    subtitle: describeConn(asset.conn),
  })),
);

function describeConn(conn: OracleConn): string {
  const cs = resolveConnectionString(conn);
  return conn.username ? `${conn.username} · ${cs}` : cs;
}

function makeDefaultName(conn: OracleConn): string {
  return conn.username ? `Oracle · ${conn.username}` : 'Oracle';
}

const dialogVisible = ref(false);
const dialogMode = ref<'create' | 'edit'>('create');
const dialogBusy = ref(false);
const dialogError = ref('');
const editingId = ref<string | null>(null);
const editingEncryptedPassword = ref<string | undefined>(undefined);
const clearSavedPassword = ref(false);

const dialogTitle = computed(() => (dialogMode.value === 'create' ? '新建连接（Oracle / ODBC）' : '编辑连接（Oracle / ODBC）'));
const dialogSaveLabel = computed(() => (dialogMode.value === 'create' ? '创建' : '保存'));

function resetForm() {
  formName.value = '';
  formConnectionString.value = '';
  formUsername.value = '';
  formPassword.value = '';
}

function openCreate() {
  dialogMode.value = 'create';
  editingId.value = null;
  editingEncryptedPassword.value = undefined;
  clearSavedPassword.value = false;
  dialogError.value = '';
  resetForm();
  dialogVisible.value = true;
}

function openEdit(id: string) {
  const asset = connections.value.find(c => c.id === id);
  if (!asset) {
    return;
  }

  dialogMode.value = 'edit';
  editingId.value = id;
  editingEncryptedPassword.value = asset.encryptedPassword;
  clearSavedPassword.value = false;
  dialogError.value = '';
  formName.value = asset.name;
  formConnectionString.value = resolveConnectionString(asset.conn);
  formUsername.value = asset.conn.username ?? '';
  formPassword.value = '';
  dialogVisible.value = true;
}

function closeDialog() {
  dialogVisible.value = false;
}

async function saveDialog() {
  const connectionString = formConnectionString.value.trim();
  if (!connectionString) {
    dialogError.value = '请填写 ODBC 连接串';
    return;
  }

  const username = formUsername.value.trim();
  const conn: OracleConn = {
    connectionString,
    username: username || undefined,
  };

  dialogBusy.value = true;
  dialogError.value = '';
  try {
    const password = formPassword.value.trim() || undefined;
    let encryptedPassword = dialogMode.value === 'edit' ? editingEncryptedPassword.value : undefined;
    if (dialogMode.value === 'edit' && clearSavedPassword.value) {
      encryptedPassword = undefined;
    }
    if (password) {
      encryptedPassword = await cryptoApi.encrypt(password);
    }

    const asset: SqlConnectionAsset<OracleConn> = {
      id: editingId.value ?? crypto.randomUUID(),
      name: formName.value.trim() || makeDefaultName(conn),
      conn,
      encryptedPassword,
    };
    if (dialogMode.value === 'create') {
      await addConnection(asset, '');
    } else {
      await updateConnection(asset);
    }
    dialogVisible.value = false;
  } catch (e: unknown) {
    dialogError.value = e instanceof Error ? e.message : '保存失败';
  } finally {
    dialogBusy.value = false;
  }
}

async function handleDelete(id: string) {
  const asset = connections.value.find(c => c.id === id);
  if (!asset) {
    return;
  }
  const ok = await confirm('删除连接', `确认删除 “${asset.name}” 吗？`);
  if (!ok) {
    return;
  }
  await removeConnection(id);
}

onMounted(() => {
  void loadConnections();
});
</script>
