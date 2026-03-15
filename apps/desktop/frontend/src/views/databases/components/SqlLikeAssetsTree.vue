<template>
  <div class="tree-section">
    <div class="section-title row">
      <span>资产树</span>
      <button type="button" class="btn btn-ghost btn-sm" :disabled="tablesLoading" @click="emit('refreshTables')">刷新</button>
    </div>

    <div v-if="tablesLoading" class="empty">加载中...</div>
    <div v-else-if="tables.length === 0" class="empty">暂无表</div>
    <div v-else class="table-list">
      <button
        v-for="t in tables"
        :key="t.name"
        type="button"
        class="table-item"
        :class="{ active: t.name === activeTableName }"
        @click="emit('selectTable', t.name)"
      >
        <span class="table-name">{{ t.name }}</span>
        <span class="table-kind">{{ t.kind }}</span>
      </button>
    </div>

    <div v-if="activeTableName" class="columns-section">
      <div class="section-title row">
        <span>列：{{ activeTableName }}</span>
        <button type="button" class="btn btn-ghost btn-sm" :disabled="columnsLoading" @click="emit('refreshColumns', activeTableName)">
          刷新
        </button>
      </div>
      <div v-if="columnsLoading" class="empty">加载中...</div>
      <div v-else-if="columns.length === 0" class="empty">暂无列信息</div>
      <div v-else class="columns-list">
        <div v-for="c in columns" :key="c.name" class="column-item">
          <span class="column-name">{{ c.name }}</span>
          <span class="column-type">{{ c.dataType || 'UNKNOWN' }}</span>
          <span v-if="c.primaryKey" class="column-tag">PK</span>
          <span v-else-if="c.notNull" class="column-tag">NN</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { DbColumn, DbTable } from '@/lib/api-database';

defineProps<{
  tables: DbTable[];
  tablesLoading: boolean;
  activeTableName: string | null;
  columns: DbColumn[];
  columnsLoading: boolean;
}>();

const emit = defineEmits<{
  refreshTables: [];
  selectTable: [tableName: string];
  refreshColumns: [tableName: string];
}>();
</script>
