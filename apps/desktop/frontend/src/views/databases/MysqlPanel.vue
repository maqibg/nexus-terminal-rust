<template>
  <div class="db-layout">
    <aside class="db-sidebar">
      <div class="section-title">MySQL / MariaDB</div>

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
      <input v-model="formName" class="field-input" type="text" placeholder="例如：prod-mysql" />
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
        <input v-model="formUsername" class="field-input" type="text" placeholder="root" />
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

    <div class="field-block">
      <label class="field-label">Database</label>
      <input v-model="formDatabase" class="field-input" type="text" placeholder="例如：app_db" />
    </div>
  </DbConnectionDialog>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';

import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { cryptoApi } from '@/lib/api';
import { databaseApi, type MysqlConnection } from '@/lib/api-database';
import { useSqlLikeManager, type SqlConnectionAsset, type SqlLikeDriver } from './useSqlLikeManager';
import DbConnectionList, { type DbConnListItem } from './components/DbConnectionList.vue';
import DbConnectionDialog from './components/DbConnectionDialog.vue';
import SqlLikeAssetsTree from './components/SqlLikeAssetsTree.vue';
import SqlLikeQueryPanel from './components/SqlLikeQueryPanel.vue';

type MysqlConn = Omit<MysqlConnection, 'password' | 'encrypted_password'>;

const driver: SqlLikeDriver<MysqlConn> = {
  settingsKey: 'dbManager.mysqlConnections',
  buildSelectStar: (tableName) => `SELECT * FROM \`${tableName.replaceAll('`', '``')}\` LIMIT 100;`,
  listTables: async (conn, auth) =>
    databaseApi.mysqlListTables({ ...conn, password: auth.password, encrypted_password: auth.encryptedPassword }),
  listColumns: async (conn, tableName, auth) =>
    databaseApi.mysqlListColumns({ ...conn, password: auth.password, encrypted_password: auth.encryptedPassword }, tableName),
  query: async (conn, sql, auth) =>
    databaseApi.mysqlQuery({ ...conn, password: auth.password, encrypted_password: auth.encryptedPassword }, sql),
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
} = useSqlLikeManager<MysqlConn>(driver);

const { confirm } = useConfirmDialog();

const connectionItems = computed<DbConnListItem[]>(() =>
  connections.value.map((asset) => ({
    id: asset.id,
    name: asset.name,
    subtitle: describeConn(asset.conn),
  })),
);

const formName = ref('');
const formHost = ref('127.0.0.1');
const formPort = ref<number>(3306);
const formUsername = ref('root');
const formPassword = ref('');
const formDatabase = ref('');

function describeConn(conn: MysqlConn): string {
  return `${conn.username}@${conn.host}:${conn.port}/${conn.database}`;
}

function makeDefaultName(conn: MysqlConn): string {
  return `MySQL · ${conn.host}:${conn.port}/${conn.database}`;
}

const dialogVisible = ref(false);
const dialogMode = ref<'create' | 'edit'>('create');
const dialogBusy = ref(false);
const dialogError = ref('');
const editingId = ref<string | null>(null);
const editingEncryptedPassword = ref<string | undefined>(undefined);
const clearSavedPassword = ref(false);

const dialogTitle = computed(() => (dialogMode.value === 'create' ? '新建连接（MySQL / MariaDB）' : '编辑连接（MySQL / MariaDB）'));
const dialogSaveLabel = computed(() => (dialogMode.value === 'create' ? '创建' : '保存'));

function resetForm() {
  formName.value = '';
  formHost.value = '127.0.0.1';
  formPort.value = 3306;
  formUsername.value = 'root';
  formPassword.value = '';
  formDatabase.value = '';
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

  const conn: MysqlConn = {
    host,
    port: Number(formPort.value) || 3306,
    username,
    database,
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
    if (dialogMode.value === 'create') {
      const asset: SqlConnectionAsset<MysqlConn> = {
        id: crypto.randomUUID(),
        name: formName.value.trim() || makeDefaultName(conn),
        conn,
        encryptedPassword,
      };
      await addConnection(asset, '');
    } else if (editingId.value) {
      const asset: SqlConnectionAsset<MysqlConn> = {
        id: editingId.value,
        name: formName.value.trim() || makeDefaultName(conn),
        conn,
        encryptedPassword,
      };
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
