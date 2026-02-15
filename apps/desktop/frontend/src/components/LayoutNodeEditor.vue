<script setup lang="ts">
import { computed, type PropType } from 'vue';
import draggable from 'vuedraggable';
import { useLayoutStore, type PaneName } from '@/stores/layout';

export interface LayoutEditorNode {
  id: string;
  type: 'pane' | 'container';
  component?: PaneName;
  direction?: 'horizontal' | 'vertical';
  children?: LayoutEditorNode[];
  size?: number;
}

const props = defineProps({
  node: {
    type: Object as PropType<LayoutEditorNode>,
    required: true,
  },
  parentNode: {
    type: Object as PropType<LayoutEditorNode | null>,
    default: null,
  },
  nodeIndex: {
    type: Number,
    default: -1,
  },
  paneLabels: {
    type: Object as PropType<Record<PaneName, string>>,
    required: true,
  },
  group: {
    type: String,
    default: 'layout-items',
  },
});

const emit = defineEmits<{
  'update:node': [node: LayoutEditorNode];
  removeNode: [payload: { parentNodeId: string | undefined; nodeIndex: number }];
}>();

const layoutStore = useLayoutStore();

const childrenList = computed({
  get: () => props.node.children ?? [],
  set: (newChildren: LayoutEditorNode[]) => {
    emit('update:node', { ...props.node, children: newChildren });
  },
});

const nodeTitle = computed(() => {
  if (props.node.type === 'pane') {
    const pane = props.node.component;
    if (!pane) {
      return '未知面板';
    }
    return props.paneLabels[pane] ?? pane;
  }

  return props.node.direction === 'horizontal' ? '容器(水平)' : '容器(垂直)';
});

function addHorizontalContainer() {
  const nextChild: LayoutEditorNode = {
    id: layoutStore.generateId(),
    type: 'container',
    direction: 'horizontal',
    children: [],
    size: 50,
  };

  const nextChildren = [...(props.node.children ?? []), nextChild];
  emit('update:node', { ...props.node, children: nextChildren });
}

function addVerticalContainer() {
  const nextChild: LayoutEditorNode = {
    id: layoutStore.generateId(),
    type: 'container',
    direction: 'vertical',
    children: [],
    size: 50,
  };

  const nextChildren = [...(props.node.children ?? []), nextChild];
  emit('update:node', { ...props.node, children: nextChildren });
}

function toggleDirection() {
  if (props.node.type !== 'container') {
    return;
  }

  emit('update:node', {
    ...props.node,
    direction: props.node.direction === 'horizontal' ? 'vertical' : 'horizontal',
  });
}

function removeSelf() {
  emit('removeNode', { parentNodeId: props.parentNode?.id, nodeIndex: props.nodeIndex });
}

function handleChildUpdate(updatedChildNode: LayoutEditorNode, index: number) {
  if (!props.node.children) {
    return;
  }

  const nextChildren = [...props.node.children];
  nextChildren[index] = updatedChildNode;
  emit('update:node', { ...props.node, children: nextChildren });
}

function handleChildRemove(payload: { parentNodeId: string | undefined; nodeIndex: number }) {
  emit('removeNode', payload);
}
</script>

<template>
  <div
    class="layout-node-editor"
    :class="[
      `node-type-${node.type}`,
      node.direction ? `direction-${node.direction}` : '',
    ]"
    :data-node-id="node.id"
  >
    <div class="node-controls">
      <span class="node-info">{{ nodeTitle }}</span>

      <div class="node-actions">
        <button
          v-if="node.type === 'container'"
          class="action-button"
          title="切换方向"
          @click="toggleDirection"
        >
          <i class="fas fa-sync-alt"></i>
        </button>

        <button
          v-if="node.type === 'container'"
          class="action-button"
          title="添加水平容器"
          @click="addHorizontalContainer"
        >
          <i class="fas fa-columns"></i>
          <span>H</span>
        </button>

        <button
          v-if="node.type === 'container'"
          class="action-button"
          title="添加垂直容器"
          @click="addVerticalContainer"
        >
          <i class="fas fa-bars"></i>
          <span>V</span>
        </button>

        <button
          class="action-button remove-button"
          title="移除"
          :disabled="!parentNode"
          @click="removeSelf"
        >
          <i class="fas fa-trash-alt"></i>
        </button>
      </div>
    </div>

    <draggable
      v-if="node.type === 'container'"
      v-model="childrenList"
      :group="group"
      item-key="id"
      tag="div"
      class="node-children-container"
      :class="[`children-direction-${node.direction}`]"
      handle=".drag-handle-node"
      ghost-class="sortable-ghost"
      chosen-class="sortable-chosen"
      drag-class="sortable-drag"
      :animation="150"
    >
      <template #item="{ element: childNode, index }">
        <div :key="childNode.id" class="child-node-wrapper">
          <i class="fas fa-grip-vertical drag-handle-node" title="拖拽"></i>

          <LayoutNodeEditor
            :node="childNode"
            :parent-node="node"
            :node-index="index"
            :pane-labels="paneLabels"
            :group="group"
            @update:node="handleChildUpdate($event, index)"
            @removeNode="handleChildRemove"
          />
        </div>
      </template>

      <template #footer>
        <div v-if="childrenList.length === 0" class="empty-container-placeholder">
          从可用面板拖拽到此处
        </div>
      </template>
    </draggable>

    <div v-else class="pane-node-content"></div>
  </div>
</template>

<style scoped>
.layout-node-editor {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: rgba(22, 25, 39, 0.55);
  min-height: 44px;
}

.node-controls {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  background: rgba(45, 50, 73, 0.9);
  border-bottom: 1px solid var(--border);
  padding: 4px 8px;
  min-height: 28px;
}

.node-info {
  color: var(--text);
  font-size: 12px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.action-button {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-sub);
  border-radius: 4px;
  padding: 2px 5px;
  min-width: 22px;
  height: 22px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-size: 11px;
}

.action-button:hover:not(:disabled) {
  color: var(--text);
  border-color: var(--blue);
  background: rgba(137, 180, 250, 0.12);
}

.action-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.remove-button {
  border-color: rgba(243, 139, 168, 0.7);
  color: #f38ba8;
}

.remove-button:hover:not(:disabled) {
  background: rgba(243, 139, 168, 0.18);
}

.node-children-container {
  flex: 1;
  display: flex;
  gap: 6px;
  border: 1px dashed rgba(132, 139, 179, 0.45);
  border-radius: 6px;
  margin: 6px;
  padding: 6px;
  min-height: 44px;
}

.children-direction-horizontal {
  flex-direction: row;
}

.children-direction-vertical {
  flex-direction: column;
}

.child-node-wrapper {
  display: flex;
  align-items: stretch;
  gap: 0;
  min-width: 0;
  flex: 1 1 auto;
}

.children-direction-vertical > .child-node-wrapper {
  width: 100%;
}

.drag-handle-node {
  width: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-dim);
  background: rgba(45, 50, 73, 0.85);
  border: 1px solid var(--border);
  border-right: none;
  border-radius: 6px 0 0 6px;
  cursor: grab;
}

.children-direction-vertical > .child-node-wrapper > .drag-handle-node {
  width: 100%;
  height: 16px;
  border-right: 1px solid var(--border);
  border-bottom: none;
  border-radius: 6px 6px 0 0;
}

.child-node-wrapper > .layout-node-editor {
  flex: 1;
  min-width: 0;
}

.pane-node-content {
  min-height: 4px;
}

.empty-container-placeholder {
  flex: 1;
  min-height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px dashed rgba(132, 139, 179, 0.45);
  border-radius: 6px;
  color: var(--text-dim);
  font-size: 12px;
  text-align: center;
  padding: 8px;
}

.sortable-ghost {
  opacity: 0.45;
}

.sortable-chosen,
.sortable-drag {
  opacity: 0.92;
}
</style>
