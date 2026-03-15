<template>
  <div class="db-layout">
    <aside class="db-sidebar">
      <div class="section-title">SQL Server</div>
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
      <input v-model="formName" class="field-input" type="text" placeholder="例如：prod-mssql" />
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
        <input v-model="formUsername" class="field-input" type="text" placeholder="sa" />
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
        <input v-model="formDatabase" class="field-input" type="text" placeholder="例如：master" />
      </div>
      <div class="field-block">
        <label class="field-label">Schema（可选）</label>
        <input v-model="formSchema" class="field-input" type="text" placeholder="dbo" />
      </div>
    </div>

    <div class="field-row">
      <label class="checkbox">
        <input v-model="formEncrypt" type="checkbox" />
        <span>加密（Encrypt）</span>
      </label>
      <label class="checkbox">
        <input v-model="formTrustServerCert" type="checkbox" />
        <span>信任证书（TrustServerCertificate）</span>
      </label>
    </div>
    <div class="hint">提示：自签名/内网证书常需要开启“信任证书”。</div>
  </DbConnectionDialog>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { cryptoApi } from '@/lib/api';
import { databaseApi, type MssqlConnection } from '@/lib/api-database';
import { useSqlLikeManager, type SqlConnectionAsset, type SqlLikeDriver } from './useSqlLikeManager';
import DbConnectionList, { type DbConnListItem } from './components/DbConnectionList.vue';
import DbConnectionDialog from './components/DbConnectionDialog.vue';
import SqlLikeAssetsTree from './components/SqlLikeAssetsTree.vue';
import SqlLikeQueryPanel from './components/SqlLikeQueryPanel.vue';

type MssqlConn = Omit<MssqlConnection, 'password' | 'encrypted_password'>;

function mssqlQuoteIdent(input: string) {
  return `[${input.replaceAll(']', ']]')}]`;
}

function normalizeMssqlConn(conn: MssqlConn): MssqlConn {
  const legacy = conn as unknown as { trust_server_certificate?: unknown };
  if (typeof conn.trustServerCertificate === 'boolean') {
    return conn;
  }
  if (typeof legacy.trust_server_certificate === 'boolean') {
    return { ...conn, trustServerCertificate: legacy.trust_server_certificate };
  }
  return conn;
}

const driver: SqlLikeDriver<MssqlConn> = {
  settingsKey: 'dbManager.mssqlConnections',
  buildSelectStar: (tableName, conn) => {
    const schema = (conn.schema || 'dbo').trim() || 'dbo';
    return `SELECT TOP 100 * FROM ${mssqlQuoteIdent(schema)}.${mssqlQuoteIdent(tableName)};`;
  },
  listTables: async (conn, auth) =>
    databaseApi.mssqlListTables({ ...normalizeMssqlConn(conn), password: auth.password, encrypted_password: auth.encryptedPassword }),
  listColumns: async (conn, tableName, auth) =>
    databaseApi.mssqlListColumns(
      { ...normalizeMssqlConn(conn), password: auth.password, encrypted_password: auth.encryptedPassword },
      tableName,
    ),
  query: async (conn, sql, auth) =>
    databaseApi.mssqlQuery({ ...normalizeMssqlConn(conn), password: auth.password, encrypted_password: auth.encryptedPassword }, sql),
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
} = useSqlLikeManager<MssqlConn>(driver);

const { confirm } = useConfirmDialog();

const formName = ref('');
const formHost = ref('127.0.0.1');
const formPort = ref<number>(1433);
const formUsername = ref('sa');
const formPassword = ref('');
const formDatabase = ref('master');
const formSchema = ref('dbo');
const formEncrypt = ref(true);
const formTrustServerCert = ref(false);

const connectionItems = computed<DbConnListItem[]>(() =>
  connections.value.map((asset) => ({
    id: asset.id,
    name: asset.name,
    subtitle: describeConn(asset.conn),
  })),
);

function describeConn(conn: MssqlConn): string {
  const schema = conn.schema ? `/${conn.schema}` : '';
  return `${conn.username}@${conn.host}:${conn.port}/${conn.database}${schema}`;
}

function makeDefaultName(conn: MssqlConn): string {
  return `MSSQL · ${conn.host}:${conn.port}/${conn.database}`;
}

const dialogVisible = ref(false);
const dialogMode = ref<'create' | 'edit'>('create');
const dialogBusy = ref(false);
const dialogError = ref('');
const editingId = ref<string | null>(null);
const editingEncryptedPassword = ref<string | undefined>(undefined);
const clearSavedPassword = ref(false);

const dialogTitle = computed(() => (dialogMode.value === 'create' ? '新建连接（SQL Server）' : '编辑连接（SQL Server）'));
const dialogSaveLabel = computed(() => (dialogMode.value === 'create' ? '创建' : '保存'));

function resetForm() {
  formName.value = '';
  formHost.value = '127.0.0.1';
  formPort.value = 1433;
  formUsername.value = 'sa';
  formPassword.value = '';
  formDatabase.value = 'master';
  formSchema.value = 'dbo';
  formEncrypt.value = true;
  formTrustServerCert.value = false;
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
  formSchema.value = asset.conn.schema ?? 'dbo';
  formEncrypt.value = asset.conn.encrypt ?? true;
  formTrustServerCert.value = asset.conn.trustServerCertificate ?? false;
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
  const conn: MssqlConn = {
    host,
    port: Number(formPort.value) || 1433,
    username,
    database,
    schema: schema || undefined,
    trustServerCertificate: formTrustServerCert.value,
    encrypt: formEncrypt.value,
  };

  const asset: SqlConnectionAsset<MssqlConn> = {
    id: editingId.value ?? crypto.randomUUID(),
    name: formName.value.trim() || makeDefaultName(conn),
    conn,
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
    asset.encryptedPassword = encryptedPassword;
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
