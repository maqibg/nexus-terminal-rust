<template>
  <div class="node-editor" v-if="node">
    <div class="field">
      <span class="label">类型</span>
      <select v-model="node.type" class="input">
        <option value="terminal">终端</option>
        <option value="sftp">SFTP</option>
        <option value="editor">编辑器</option>
        <option value="split">分割</option>
      </select>
    </div>
    <template v-if="node.type === 'split'">
      <div class="field">
        <span class="label">方向</span>
        <select v-model="node.direction" class="input">
          <option value="horizontal">水平</option>
          <option value="vertical">垂直</option>
        </select>
      </div>
      <div class="field">
        <span class="label">比例 (%)</span>
        <input v-model.number="node.ratio" type="number" min="10" max="90" class="input" />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
defineProps<{ node: { type: string; direction?: string; ratio?: number } | null }>();
</script>

<style scoped>
.node-editor { display: flex; flex-direction: column; gap: 8px; padding: 8px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.label { font-size: 11px; color: var(--text-sub); }
.input { padding: 4px 8px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 3px; color: var(--text); font-size: 12px; }
</style>
