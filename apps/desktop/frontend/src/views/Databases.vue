<template>
  <div class="page db-page">
    <div class="page-header">
      <h2>数据库</h2>
      <div class="header-actions">
        <div class="page-subtitle">资产树 + SQL 编辑器 + 查询结果</div>
      </div>
    </div>

    <div class="db-type-tabs">
      <button
        v-for="t in tabs"
        :key="t.id"
        type="button"
        class="tab-btn"
        :class="{ active: t.id === activeTabId }"
        @click="activeTabId = t.id"
      >
        {{ t.label }}
      </button>
    </div>

    <component :is="activeTab.component" />
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, ref } from 'vue';

import './databases/databases.css';

const SqlitePanel = defineAsyncComponent(() => import('./databases/SqlitePanel.vue'));
const MysqlPanel = defineAsyncComponent(() => import('./databases/MysqlPanel.vue'));
const PostgresPanel = defineAsyncComponent(() => import('./databases/PostgresPanel.vue'));
const MssqlPanel = defineAsyncComponent(() => import('./databases/MssqlPanel.vue'));
const ClickHousePanel = defineAsyncComponent(() => import('./databases/ClickHousePanel.vue'));
const OraclePanel = defineAsyncComponent(() => import('./databases/OraclePanel.vue'));
const RedisPanel = defineAsyncComponent(() => import('./databases/RedisPanel.vue'));

type TabId = 'sqlite' | 'mysql' | 'postgres' | 'mssql' | 'clickhouse' | 'oracle' | 'redis';

const tabs = [
  { id: 'sqlite', label: 'SQLite', component: SqlitePanel },
  { id: 'mysql', label: 'MySQL / MariaDB', component: MysqlPanel },
  { id: 'postgres', label: 'PostgreSQL', component: PostgresPanel },
  { id: 'mssql', label: 'SQL Server', component: MssqlPanel },
  { id: 'clickhouse', label: 'ClickHouse', component: ClickHousePanel },
  { id: 'oracle', label: 'Oracle', component: OraclePanel },
  { id: 'redis', label: 'Redis', component: RedisPanel },
] as const satisfies readonly { id: TabId; label: string; component: unknown }[];

const activeTabId = ref<TabId>('sqlite');
const activeTab = computed(() => tabs.find(t => t.id === activeTabId.value) ?? tabs[0]);
</script>
