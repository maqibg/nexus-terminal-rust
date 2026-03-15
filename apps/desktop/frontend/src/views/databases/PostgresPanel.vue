<template>
  <div class="db-layout">
    <aside class="db-sidebar">
      <div class="section-title">PostgreSQL</div>

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
      <input v-model="formName" class="field-input" type="text" placeholder="例如：prod-pg" />
    </div>

    <div class="field-grid">
      <div class="field-block">
        <label class="field-label">Host</label>
        <input v-model="formHost" class="field-input" type="text" placeholder="127.0.0.1" />
      </div>
      <div class="field-block">
        <label class="field-label">Port</label>
        <input v-model.number="formPort" class="field-input" type="number" min="1" max="65535" />
      </div>
    </div>

    <div class="field-grid">
      <div class="field-block">
        <label class="field-label">用户名</label>
        <input v-model="formUsername" class="field-input" type="text" placeholder="postgres" />
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

    <div class="field-grid">
      <div class="field-block">
        <label class="field-label">Database</label>
        <input v-model="formDatabase" class="field-input" type="text" placeholder="例如：app_db" />
      </div>
      <div class="field-block">
        <label class="field-label">Schema（可选）</label>
        <input v-model="formSchema" class="field-input" type="text" placeholder="public" />
      </div>
    </div>
  </DbConnectionDialog>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';

import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { cryptoApi } from '@/lib/api';
import { databaseApi, type PostgresConnection } from '@/lib/api-database';
import { useSqlLikeManager, type SqlConnectionAsset, type SqlLikeDriver } from './useSqlLikeManager';
import DbConnectionList, { type DbConnListItem } from './components/DbConnectionList.vue';
import DbConnectionDialog from './components/DbConnectionDialog.vue';
import SqlLikeAssetsTree from './components/SqlLikeAssetsTree.vue';
import SqlLikeQueryPanel from './components/SqlLikeQueryPanel.vue';

type PgConn = Omit<PostgresConnection, 'password' | 'encrypted_password'>;

function pgQuoteIdent(input: string) {
  return `"${input.replaceAll('"', '""')}"`;
}

const driver: SqlLikeDriver<PgConn> = {
  settingsKey: 'dbManager.postgresConnections',
  buildSelectStar: (tableName, conn) => {
    const schema = (conn.schema || 'public').trim() || 'public';
    return `SELECT * FROM ${pgQuoteIdent(schema)}.${pgQuoteIdent(tableName)} LIMIT 100;`;
  },
  listTables: async (conn, auth) =>
    databaseApi.postgresListTables({ ...conn, password: auth.password, encrypted_password: auth.encryptedPassword }),
  listColumns: async (conn, tableName, auth) =>
    databaseApi.postgresListColumns({ ...conn, password: auth.password, encrypted_password: auth.encryptedPassword }, tableName),
  query: async (conn, sql, auth) =>
    databaseApi.postgresQuery({ ...conn, password: auth.password, encrypted_password: auth.encryptedPassword }, sql),
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
} = useSqlLikeManager<PgConn>(driver);

const { confirm } = useConfirmDialog();

const formName = ref('');
const formHost = ref('127.0.0.1');
const formPort = ref<number>(5432);
const formUsername = ref('postgres');
const formPassword = ref('');
const formDatabase = ref('');
const formSchema = ref('public');

const connectionItems = computed<DbConnListItem[]>(() =>
  connections.value.map((asset) => ({
    id: asset.id,
    name: asset.name,
    subtitle: describeConn(asset.conn),
  })),
);

function describeConn(conn: PgConn): string {
  const schema = conn.schema ? `/${conn.schema}` : '';
  return `${conn.username}@${conn.host}:${conn.port}/${conn.database}${schema}`;
}

function makeDefaultName(conn: PgConn): string {
  return `Postgres · ${conn.host}:${conn.port}/${conn.database}`;
}

const dialogVisible = ref(false);
const dialogMode = ref<'create' | 'edit'>('create');
const dialogBusy = ref(false);
const dialogError = ref('');
const editingId = ref<string | null>(null);
const editingEncryptedPassword = ref<string | undefined>(undefined);
const clearSavedPassword = ref(false);

const dialogTitle = computed(() => (dialogMode.value === 'create' ? '新建连接（PostgreSQL）' : '编辑连接（PostgreSQL）'));
const dialogSaveLabel = computed(() => (dialogMode.value === 'create' ? '创建' : '保存'));

function resetForm() {
  formName.value = '';
  formHost.value = '127.0.0.1';
  formPort.value = 5432;
  formUsername.value = 'postgres';
  formPassword.value = '';
  formDatabase.value = '';
  formSchema.value = 'public';
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
  formHost.value = asset.conn.host;
  formPort.value = asset.conn.port;
  formUsername.value = asset.conn.username;
  formDatabase.value = asset.conn.database;
  formSchema.value = asset.conn.schema ?? 'public';
  formPassword.value = '';
  dialogVisible.value = true;
}

function closeDialog() {
  dialogVisible.value = false;
}

async function saveDialog() {
  const host = formHost.value.trim();
  const username = formUsername.value.trim();
  const database = formDatabase.value.trim();
  if (!host || !username || !database) {
    dialogError.value = '请填写 Host / 用户名 / Database';
    return;
  }

  const schema = formSchema.value.trim();
  const conn: PgConn = {
    host,
    port: Number(formPort.value) || 5432,
    username,
    database,
    schema: schema || undefined,
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

    const asset: SqlConnectionAsset<PgConn> = {
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
