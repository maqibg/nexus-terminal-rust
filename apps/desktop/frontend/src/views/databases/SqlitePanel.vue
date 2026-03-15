<template>
  <div class="db-layout">
    <aside class="db-sidebar">
      <div class="section-title">SQLite</div>

      <DbConnectionList
        title="连接"
        :items="sqliteItems"
        :active-id="activeSqliteId"
        create-label="打开"
        :show-edit="false"
        @select="selectSqlite"
        @create="addSqlite"
        @delete="handleDelete"
        @refresh="loadConnections"
      />

      <SqlLikeAssetsTree
        v-if="activeSqlite"
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
      <div v-if="!activeSqlite" class="empty big">请选择或打开一个 SQLite 文件。</div>
      <SqlLikeQueryPanel
        v-else
        :has-connection="true"
        :password-enabled="false"
        :password="''"
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
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';

import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { useSqliteManager } from './useSqliteManager';
import DbConnectionList, { type DbConnListItem } from './components/DbConnectionList.vue';
import SqlLikeAssetsTree from './components/SqlLikeAssetsTree.vue';
import SqlLikeQueryPanel from './components/SqlLikeQueryPanel.vue';

const {
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
} = useSqliteManager();

const { confirm } = useConfirmDialog();

const sqliteItems = computed<DbConnListItem[]>(() =>
  sqliteConnections.value.map((asset) => ({
    id: asset.id,
    name: asset.name,
    subtitle: asset.path,
  })),
);

async function handleDelete(id: string) {
  const asset = sqliteConnections.value.find(c => c.id === id);
  if (!asset) {
    return;
  }
  const ok = await confirm('删除连接', `确认删除 “${asset.name}” 吗？`);
  if (!ok) {
    return;
  }
  await removeSqlite(id);
}

onMounted(() => {
  void loadConnections();
});
</script>
