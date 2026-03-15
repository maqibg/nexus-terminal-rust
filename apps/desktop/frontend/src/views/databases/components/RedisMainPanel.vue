<template>
  <div v-if="!hasConnection" class="empty big">请先添加并选择一个连接。</div>
  <div v-else class="db-panel">
    <div class="panel-header">
      <div class="panel-title">Redis 命令</div>
      <div class="panel-actions">
        <input
          :value="password"
          class="pwd-input"
          type="password"
          placeholder="可留空，使用已保存密码"
          @input="emit('update:password', ($event.target as HTMLInputElement).value)"
        />
        <div class="db-switch">
          <span class="db-label">DB</span>
          <input
            :value="db"
            class="db-input"
            type="number"
            min="0"
            @input="emit('update:db', Number(($event.target as HTMLInputElement).value) || 0)"
            @keydown.enter.prevent="emit('applyDbSwitch')"
          />
          <button type="button" class="btn btn-ghost btn-sm" title="切换 DB 并刷新 Keys" @click="emit('applyDbSwitch')">切换</button>
        </div>
        <button type="button" class="btn btn-primary" :disabled="commandLoading" @click="emit('runCommand')">
          {{ commandLoading ? '执行中...' : '执行' }}
        </button>
        <button type="button" class="btn btn-ghost" @click="emit('clearCommand')">清空</button>
        <button v-if="activeKey" type="button" class="btn btn-ghost" title="插入 GET key" @click="emit('insertGet', activeKey)">
          GET
        </button>
      </div>
    </div>

    <div class="editor-wrap">
      <MonacoEditor
        :model-value="command"
        language="plaintext"
        @update:model-value="(v) => emit('update:command', v)"
      />
    </div>
    <div class="hint">输入一条命令（支持引号），例如：<code>HGETALL &quot;user:1&quot;</code></div>
    <div v-if="commandError" class="error">{{ commandError }}</div>

    <div class="panel-header">
      <div class="panel-title">Key 详情</div>
      <div v-if="keyDetail" class="meta">
        <span>Type {{ keyDetail.keyType }}</span>
        <span>TTL {{ keyDetail.ttlSeconds === null ? '-' : `${keyDetail.ttlSeconds}s` }}</span>
        <span v-if="keyDetail.length !== null && keyDetail.length !== undefined">Len {{ keyDetail.length }}</span>
        <span v-if="keyDetail.encoding">Enc {{ keyDetail.encoding }}</span>
        <span v-if="keyDetail.memoryUsageBytes !== null && keyDetail.memoryUsageBytes !== undefined">Mem {{ keyDetail.memoryUsageBytes }}B</span>
      </div>
    </div>

    <div v-if="keyLoading" class="empty">加载中...</div>
    <div v-else-if="!keyDetail" class="empty">未选择 Key</div>
    <div v-else class="key-detail">
      <div class="selected-key" :title="keyDetail.key">{{ keyDetail.key }}</div>
      <div v-if="keyDetail.metaError" class="error">{{ keyDetail.metaError }}</div>
      <pre class="json">{{ formatJson(keyDetail.value) }}</pre>
    </div>

    <div class="panel-header">
      <div class="panel-title">命令输出</div>
      <div v-if="commandResult" class="meta">
        <span>耗时 {{ commandResult.durationMs }}ms</span>
      </div>
    </div>

    <div v-if="commandLoading" class="empty">执行中...</div>
    <div v-else-if="!commandResult" class="empty">未执行命令</div>
    <pre v-else class="json">{{ formatJson(commandResult.result) }}</pre>
  </div>
</template>

<script setup lang="ts">
import MonacoEditor from '@/components/MonacoEditor.vue';
import type { RedisCommandResult, RedisKeyDetail } from '@/lib/api-database';

defineProps<{
  hasConnection: boolean;
  password: string;
  db: number;
  command: string;
  commandLoading: boolean;
  commandError: string;
  commandResult: RedisCommandResult | null;
  activeKey: string;
  keyDetail: RedisKeyDetail | null;
  keyLoading: boolean;
}>();

const emit = defineEmits<{
  'update:password': [value: string];
  'update:db': [value: number];
  'update:command': [value: string];
  applyDbSwitch: [];
  runCommand: [];
  clearCommand: [];
  insertGet: [key: string];
}>();

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}
</script>

<style scoped>
.db-switch{display:inline-flex;align-items:center;gap:6px;padding:4px 6px;border:1px solid var(--border);border-radius:10px;background:var(--bg-surface1)}
.db-label{color:var(--text-dim);font-size:calc(12px + var(--ui-font-size-offset))}
.db-input{width:70px;padding:4px 6px;border-radius:8px;border:1px solid var(--border);background:var(--bg-base);color:var(--text);font-size:calc(12px + var(--ui-font-size-offset))}
.key-detail{display:flex;flex-direction:column;gap:8px}
.selected-key{font-family:Consolas,'Courier New',monospace;font-size:calc(12px + var(--ui-font-size-offset));color:var(--text);white-space:pre-wrap;word-break:break-all;border:1px solid var(--border);border-radius:10px;padding:8px 10px;background:var(--bg-surface1)}
.hint{font-size:calc(12px + var(--ui-font-size-offset));color:var(--text-dim)}
.json{margin:0;padding:10px;border-radius:10px;background:var(--bg-surface1);border:1px solid var(--border);overflow:auto;font-size:calc(12px + var(--ui-font-size-offset));line-height:1.4;max-height:260px}
code{font-family:Consolas,'Courier New',monospace;color:var(--text)}
</style>
