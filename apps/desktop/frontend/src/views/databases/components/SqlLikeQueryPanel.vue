<template>
  <div v-if="!hasConnection" class="empty big">请先添加并选择一个连接。</div>
  <div v-else class="db-panel">
    <div class="panel-header">
      <div class="panel-title">SQL 编辑器</div>
      <div class="panel-actions">
        <input
          v-if="passwordEnabled"
          :value="password"
          class="pwd-input"
          type="password"
          placeholder="密码（可留空，使用已保存密码）"
          @input="emit('update:password', ($event.target as HTMLInputElement).value)"
        />
        <button type="button" class="btn btn-primary" :disabled="queryLoading" @click="emit('runQuery')">
          {{ queryLoading ? '执行中...' : '执行' }}
        </button>
        <button type="button" class="btn btn-ghost" @click="emit('clearQuery')">清空</button>
        <button
          v-if="activeTableName"
          type="button"
          class="btn btn-ghost"
          @click="emit('insertSelectStar', activeTableName)"
          title="插入 SELECT * 语句"
        >
          SELECT *
        </button>
      </div>
    </div>

    <div class="editor-wrap">
      <MonacoEditor
        :model-value="sql"
        language="sql"
        @update:model-value="(v) => emit('update:sql', v)"
      />
    </div>
    <div v-if="queryError" class="error">{{ queryError }}</div>

    <div class="panel-header">
      <div class="panel-title">查询结果</div>
      <div v-if="queryResult" class="meta">
        <span v-if="queryResult.durationMs !== undefined">耗时 {{ queryResult.durationMs }}ms</span>
        <span v-if="queryResult.rowsAffected !== null && queryResult.rowsAffected !== undefined">影响 {{ queryResult.rowsAffected }}</span>
        <span>行数 {{ queryResult.rows.length }}</span>
      </div>
    </div>

    <div v-if="queryLoading" class="empty">执行中...</div>
    <div v-else-if="!queryResult" class="empty">暂无结果</div>
    <div v-else class="table-wrap">
      <table class="table">
        <thead>
          <tr>
            <th v-for="c in queryResult.columns" :key="c">{{ c }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, idx) in queryResult.rows" :key="idx">
            <td v-for="(cell, cidx) in row" :key="cidx">{{ formatCell(cell) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import MonacoEditor from '@/components/MonacoEditor.vue';
import type { DbQueryResult } from '@/lib/api-database';

withDefaults(
  defineProps<{
    hasConnection: boolean;
    passwordEnabled?: boolean;
    password: string;
    sql: string;
    queryLoading: boolean;
    queryError: string;
    queryResult: DbQueryResult | null;
    activeTableName: string | null;
    formatCell: (cell: unknown) => string;
  }>(),
  { passwordEnabled: true },
);

const emit = defineEmits<{
  'update:password': [value: string];
  'update:sql': [value: string];
  runQuery: [];
  clearQuery: [];
  insertSelectStar: [tableName: string];
}>();
</script>
