<script setup lang="ts">
import { computed, ref, type PropType } from 'vue';
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

const contextMenuVisible = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);

const contextMenuStyle = computed(() => ({
  left: `${contextMenuX.value}px`,
  top: `${contextMenuY.value}px`,
}));

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

function openContextMenu(event: MouseEvent) {
  const menuWidth = props.node.type === 'container' ? 196 : 168;
  const menuHeight = props.node.type === 'container' ? 154 : 50;
  const viewportPadding = 8;

  const x = Math.min(event.clientX, window.innerWidth - menuWidth - viewportPadding);
  const y = Math.min(event.clientY, window.innerHeight - menuHeight - viewportPadding);

  contextMenuX.value = Math.max(viewportPadding, x);
  contextMenuY.value = Math.max(viewportPadding, y);
  contextMenuVisible.value = true;
}

function closeContextMenu() {
  contextMenuVisible.value = false;
}

function runContextAction(action: () => void) {
  action();
  closeContextMenu();
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
    @contextmenu.stop.prevent="openContextMenu"
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
          将面板或者容器拖拽到此处
        </div>
      </template>
    </draggable>

    <div v-else class="pane-node-content"></div>

    <teleport to="body">
      <div
        v-if="contextMenuVisible"
        class="node-context-menu-mask"
        @click="closeContextMenu"
        @contextmenu.prevent="closeContextMenu"
      >
        <div class="node-context-menu" :style="contextMenuStyle" @click.stop @contextmenu.prevent>
          <button
            v-if="node.type === 'container'"
            class="node-context-menu-item"
            @click="runContextAction(toggleDirection)"
          >
            <i class="fas fa-sync-alt"></i>
            <span>切换方向</span>
          </button>

          <button
            v-if="node.type === 'container'"
            class="node-context-menu-item"
            @click="runContextAction(addHorizontalContainer)"
          >
            <i class="fas fa-columns"></i>
            <span>添加水平容器</span>
          </button>

          <button
            v-if="node.type === 'container'"
            class="node-context-menu-item"
            @click="runContextAction(addVerticalContainer)"
          >
            <i class="fas fa-bars"></i>
            <span>添加垂直容器</span>
          </button>

          <div v-if="node.type === 'container'" class="node-context-menu-separator"></div>

          <button
            class="node-context-menu-item danger"
            :disabled="!parentNode"
            @click="runContextAction(removeSelf)"
          >
            <i class="fas fa-trash-alt"></i>
            <span>移除当前节点</span>
          </button>
        </div>
      </div>
    </teleport>
  </div>
</template>

<style scoped>
.layout-node-editor {
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.node-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--ui-control-bg);
  padding: 3px 8px;
  margin-bottom: 8px;
  font-size: calc(12px + var(--ui-font-size-offset));
  min-height: 24px;
  border-radius: 4px;
  min-width: 0;
}

.node-info {
  flex: 1 1 auto;
  min-width: 0;
  color: var(--text);
  font-size: calc(12px + var(--ui-font-size-offset));
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-right: 10px;
}

.node-actions {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  gap: 3px;
}

.action-button {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-sub);
  border-radius: 3px;
  padding: 1px 4px;
  cursor: pointer;
  font-size: calc(11px + var(--ui-font-size-offset));
  line-height: 1;
}

.action-button:hover:not(:disabled) {
  color: var(--text);
  border-color: var(--blue);
  background: var(--ui-action-hover);
}

.action-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.remove-button {
  border-color: var(--ui-danger-border);
  color: var(--ui-danger);
}

.remove-button:hover:not(:disabled) {
  background: var(--ui-danger-hover);
}

.node-children-container {
  flex-grow: 1;
  padding: 8px;
  border: 1px dashed var(--ui-item-border);
  min-height: 40px;
  min-width: 0;
  display: flex;
  align-items: stretch;
  gap: 6px;
  border-radius: 6px;
}

.children-direction-horizontal {
  flex-direction: row;
}

.children-direction-vertical {
  flex-direction: column;
}

.child-node-wrapper {
  border: 1px solid transparent;
  position: relative;
  display: flex;
  align-items: stretch;
  min-width: 0;
}

.children-direction-horizontal > .child-node-wrapper {
  flex: 1 1 0;
  flex-direction: column;
  min-width: 0;
}

.children-direction-vertical > .child-node-wrapper {
  width: 100%;
  flex-direction: row;
  align-items: stretch;
}

.children-direction-horizontal > .child-node-wrapper > .drag-handle-node {
  width: 100%;
  height: 14px;
  border-right: none;
  border-bottom: 1px solid var(--border);
  border-radius: 4px 4px 0 0;
}

.drag-handle-node {
  width: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  padding-left: 3px;
  color: var(--text-dim);
  background: var(--ui-handle-bg);
  border: 1px solid var(--border);
  border-right: none;
  border-radius: 4px 0 0 4px;
  cursor: grab;
}

.child-node-wrapper > .layout-node-editor {
  flex-grow: 1;
  min-width: 0;
  margin: 0;
  border: none;
  padding: 0;
  overflow: hidden;
}

.pane-node-content {
  min-height: 30px;
  text-align: center;
  color: var(--text-dim);
  font-size: calc(12px + var(--ui-font-size-offset));
  padding-top: 8px;
}

.empty-container-placeholder {
  flex-grow: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 0;
  min-height: 30px;
  margin: 8px;
  border: 1px dashed var(--ui-item-border);
  border-radius: 6px;
  color: var(--text-dim);
  font-size: calc(12px + var(--ui-font-size-offset));
  text-align: center;
  overflow-wrap: anywhere;
  padding: 8px;
}

.sortable-ghost {
  opacity: 0.45;
  background: var(--ui-ghost-bg) !important;
  border: 1px dashed var(--ui-ghost-border) !important;
}

.sortable-chosen,
.sortable-drag {
  opacity: 0.92;
}

.node-context-menu-mask {
  position: fixed;
  inset: 0;
  z-index: 12000;
  background: transparent;
}

.node-context-menu {
  position: fixed;
  min-width: 176px;
  border: 1px solid var(--ui-menu-border);
  border-radius: 8px;
  background: var(--ui-menu-bg);
  box-shadow: 0 10px 24px var(--ui-dialog-shadow);
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.node-context-menu-item {
  height: 30px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--ui-text-primary);
  font-size: calc(12px + var(--ui-font-size-offset));
  display: inline-flex;
  align-items: center;
  gap: 8px;
  text-align: left;
  padding: 0 10px;
  cursor: pointer;
}

.node-context-menu-item:hover:not(:disabled) {
  background: var(--ui-menu-hover);
}

.node-context-menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.node-context-menu-item i {
  width: 12px;
  text-align: center;
}

.node-context-menu-item.danger {
  color: var(--ui-menu-danger-text);
}

.node-context-menu-item.danger:hover:not(:disabled) {
  background: var(--ui-menu-danger-hover);
}

.node-context-menu-separator {
  height: 1px;
  background: var(--ui-divider);
  margin: 3px 2px;
}
</style>
