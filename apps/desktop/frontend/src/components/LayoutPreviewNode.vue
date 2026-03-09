<template>
  <div
    v-if="node.type === 'pane'"
    class="preview-pane"
    @click.stop="$emit('select', path)"
  >
    {{ paneLabels[node.pane ?? 'terminal'] }}
  </div>
  <div v-else :class="['preview-split', node.direction === 'horizontal' ? 'horizontal' : 'vertical']">
    <LayoutPreviewNode
      v-for="(child, i) in node.children"
      :key="i"
      :node="child"
      :path="[...path, i]"
      @select="(p: number[]) => $emit('select', p)"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import type { LayoutNode, PaneName } from '@/stores/layout';

const paneLabels: Record<PaneName, string> = {
  connections: '连接',
  dockerManager: 'Docker',
  terminal: '终端',
  fileManager: '文件',
  editor: '编辑器',
  commandBar: '命令栏',
  statusMonitor: '状态',
  commandHistory: '历史',
  quickCommands: '快捷',
};

export default defineComponent({
  name: 'LayoutPreviewNode',
  props: {
    node: { type: Object as () => LayoutNode, required: true },
    path: { type: Array as () => number[], required: true },
  },
  emits: ['select'],
  setup() {
    return { paneLabels };
  },
});
</script>

<style scoped>
.preview-pane {
  border: 1px dashed var(--border); border-radius: 4px; padding: 8px;
  text-align: center; font-size: 11px; color: var(--text-dim); cursor: pointer;
  min-height: 40px; display: flex; align-items: center; justify-content: center; flex: 1;
}
.preview-pane:hover { border-color: var(--blue); color: var(--blue); }
.preview-split { display: flex; gap: 4px; flex: 1; }
.preview-split.horizontal { flex-direction: row; }
.preview-split.vertical { flex-direction: column; }
</style>
