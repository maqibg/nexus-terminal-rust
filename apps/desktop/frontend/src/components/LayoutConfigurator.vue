<script setup lang="ts">
import { computed, ref, watch, type Ref } from 'vue';
import draggable from 'vuedraggable';
import LayoutNodeEditor, { type LayoutEditorNode } from './LayoutNodeEditor.vue';
import { useLayoutStore, type LayoutNode as StoreLayoutNode, type PaneName, type LayoutConfig } from '@/stores/layout';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { useAlertDialog } from '@/composables/useAlertDialog';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const layoutStore = useLayoutStore();
const { confirm } = useConfirmDialog();
const { alert } = useAlertDialog();

const localLayoutTree: Ref<LayoutEditorNode | null> = ref(null);
const localSidebarPanes: Ref<{ left: PaneName[]; right: PaneName[] }> = ref({ left: [], right: [] });
const localAvailablePanes: Ref<PaneName[]> = ref([]);

const originalLayoutTree: Ref<LayoutEditorNode | null> = ref(null);
const originalSidebarPanes: Ref<{ left: PaneName[]; right: PaneName[] }> = ref({ left: [], right: [] });

const layoutLockedBoolean = computed(() => layoutStore.layoutLocked);

const isModified = computed(() => {
  const currentLayoutJson = JSON.stringify(localLayoutTree.value);
  const originalLayoutJson = JSON.stringify(originalLayoutTree.value);
  const currentSidebarJson = JSON.stringify(localSidebarPanes.value);
  const originalSidebarJson = JSON.stringify(originalSidebarPanes.value);
  return currentLayoutJson !== originalLayoutJson || currentSidebarJson !== originalSidebarJson;
});

const paneLabels = computed<Record<PaneName, string>>(() => ({
  terminal: '终端',
  commandBar: '命令栏',
  fileManager: '文件管理器',
  editor: '编辑器',
  statusMonitor: '状态监视器',
  commandHistory: '命令历史',
  quickCommands: '快捷指令',
}));

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      openWithSnapshot();
      return;
    }

    clearLocalState();
  },
);

function deepClone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function createEditorPaneNode(pane: PaneName, size = 50): LayoutEditorNode {
  return {
    id: layoutStore.generateId(),
    type: 'pane',
    component: pane,
    size,
  };
}

function toEditorNode(node: StoreLayoutNode): LayoutEditorNode {
  if (node.type === 'pane') {
    return createEditorPaneNode(node.pane ?? 'terminal', node.size ?? 50);
  }

  return {
    id: layoutStore.generateId(),
    type: 'container',
    direction: node.direction ?? 'horizontal',
    size: node.size,
    children: (node.children ?? []).map((child) => toEditorNode(child)),
  };
}

function toStoreNode(node: LayoutEditorNode): StoreLayoutNode {
  if (node.type === 'pane') {
    return {
      type: 'pane',
      pane: node.component ?? 'terminal',
      size: node.size,
    };
  }

  return {
    type: 'split',
    direction: node.direction ?? 'horizontal',
    size: node.size,
    children: (node.children ?? []).map((child) => toStoreNode(child)),
  };
}

function collectMainLayoutUsedPaneNames(node: LayoutEditorNode | null): Set<PaneName> {
  const used = new Set<PaneName>();
  if (!node) {
    return used;
  }

  const walk = (current: LayoutEditorNode) => {
    if (current.type === 'pane' && current.component) {
      used.add(current.component);
      return;
    }

    if (current.type === 'container' && Array.isArray(current.children)) {
      current.children.forEach((child) => walk(child));
    }
  };

  walk(node);
  return used;
}

function getAllLocalUsedPaneNames(layoutNode: LayoutEditorNode | null, sidebars: { left: PaneName[]; right: PaneName[] }): Set<PaneName> {
  const used = collectMainLayoutUsedPaneNames(layoutNode);
  sidebars.left.forEach((pane) => used.add(pane));
  sidebars.right.forEach((pane) => used.add(pane));
  return used;
}

function extractSidebarPanes(node?: StoreLayoutNode): PaneName[] {
  const panes: PaneName[] = [];

  const walk = (current?: StoreLayoutNode) => {
    if (!current) {
      return;
    }

    if (current.type === 'pane' && current.pane) {
      panes.push(current.pane);
      return;
    }

    if (current.type === 'split' && Array.isArray(current.children)) {
      current.children.forEach((child) => walk(child));
    }
  };

  walk(node);
  return panes;
}

function buildSidebarNode(panes: PaneName[]): StoreLayoutNode | undefined {
  if (panes.length === 0) {
    return undefined;
  }

  if (panes.length === 1) {
    return {
      type: 'pane',
      pane: panes[0],
    };
  }

  const averageSize = Number((100 / panes.length).toFixed(2));

  return {
    type: 'split',
    direction: 'vertical',
    children: panes.map((pane, index) => {
      if (index === panes.length - 1) {
        const used = averageSize * (panes.length - 1);
        return {
          type: 'pane',
          pane,
          size: Number((100 - used).toFixed(2)),
        };
      }

      return {
        type: 'pane',
        pane,
        size: averageSize,
      };
    }),
  };
}

function addPaneToAvailableList(paneName: PaneName) {
  if (paneName !== 'terminal') {
    return;
  }

  if (!localAvailablePanes.value.includes('terminal')) {
    const originalIndex = layoutStore.allPossiblePanes.indexOf('terminal');
    let inserted = false;
    for (let index = 0; index < localAvailablePanes.value.length; index += 1) {
      const currentPane = localAvailablePanes.value[index];
      const currentIndex = layoutStore.allPossiblePanes.indexOf(currentPane);
      if (originalIndex < currentIndex) {
        localAvailablePanes.value.splice(index, 0, 'terminal');
        inserted = true;
        break;
      }
    }

    if (!inserted) {
      localAvailablePanes.value.push('terminal');
    }
  }
}

function removePaneFromAvailableList(paneName: PaneName) {
  if (paneName !== 'terminal') {
    return;
  }

  const index = localAvailablePanes.value.indexOf('terminal');
  if (index >= 0) {
    localAvailablePanes.value.splice(index, 1);
  }
}

function initializeAvailablePanes() {
  const used = getAllLocalUsedPaneNames(localLayoutTree.value, localSidebarPanes.value);
  if (used.has('terminal')) {
    localAvailablePanes.value = layoutStore.allPossiblePanes.filter((pane) => pane !== 'terminal');
    return;
  }

  localAvailablePanes.value = [...layoutStore.allPossiblePanes];
}

function openWithSnapshot() {
  const nextTree = toEditorNode(layoutStore.layoutConfig.root);
  const nextSidebars = {
    left: extractSidebarPanes(layoutStore.layoutConfig.leftSidebar),
    right: extractSidebarPanes(layoutStore.layoutConfig.rightSidebar),
  };

  localLayoutTree.value = nextTree;
  originalLayoutTree.value = deepClone(nextTree);
  localSidebarPanes.value = deepClone(nextSidebars);
  originalSidebarPanes.value = deepClone(nextSidebars);
  initializeAvailablePanes();
}

function clearLocalState() {
  localLayoutTree.value = null;
  originalLayoutTree.value = null;
  localSidebarPanes.value = { left: [], right: [] };
  originalSidebarPanes.value = { left: [], right: [] };
  localAvailablePanes.value = [];
}

async function handleLayoutLockChange() {
  layoutStore.toggleLayoutLocked();
}

async function closeDialog() {
  if (!isModified.value) {
    emit('close');
    return;
  }

  const confirmed = await confirm('提示', '有未保存的更改，确定要关闭吗？');
  if (confirmed) {
    emit('close');
  }
}

async function saveLayout() {
  if (!localLayoutTree.value) {
    await alert('保存失败', '主布局为空，请先拖拽添加面板或恢复默认布局。');
    return;
  }

  const nextConfig: LayoutConfig = {
    root: toStoreNode(localLayoutTree.value),
    leftSidebar: buildSidebarNode(localSidebarPanes.value.left),
    rightSidebar: buildSidebarNode(localSidebarPanes.value.right),
  };

  layoutStore.layoutConfig = nextConfig;
  layoutStore.leftSidebarVisible = !!nextConfig.leftSidebar;
  layoutStore.rightSidebarVisible = !!nextConfig.rightSidebar;
  await layoutStore.saveLayout();
  emit('close');
}

async function resetToDefault() {
  const confirmed = await confirm('确认重置', '确定要恢复默认布局和侧栏配置吗？当前更改将丢失。');
  if (!confirmed) {
    return;
  }

  const defaults = layoutStore.getSystemDefaultLayoutConfig();
  localLayoutTree.value = toEditorNode(defaults.root);
  localSidebarPanes.value = {
    left: extractSidebarPanes(defaults.leftSidebar),
    right: extractSidebarPanes(defaults.rightSidebar),
  };
  initializeAvailablePanes();
}

function clonePane(paneName: PaneName): LayoutEditorNode {
  return {
    id: layoutStore.generateId(),
    type: 'pane',
    component: paneName,
    size: 50,
  };
}

function handleNodeUpdate(updatedNode: LayoutEditorNode) {
  localLayoutTree.value = updatedNode;
}

function findAndRemoveNode(
  node: LayoutEditorNode | null,
  parentNodeId: string | undefined,
  nodeIndex: number,
): LayoutEditorNode | null {
  if (!node) {
    return null;
  }

  if (node.id === parentNodeId && node.type === 'container' && node.children && node.children[nodeIndex]) {
    const nextChildren = [...node.children];
    const removed = nextChildren.splice(nodeIndex, 1)[0];
    if (removed.type === 'pane' && removed.component === 'terminal') {
      addPaneToAvailableList('terminal');
    }

    return {
      ...node,
      children: nextChildren,
    };
  }

  if (node.type === 'container' && node.children) {
    const nextChildren = node.children.map((child) => findAndRemoveNode(child, parentNodeId, nodeIndex));
    if (nextChildren.some((child, index) => child !== node.children?.[index])) {
      return {
        ...node,
        children: nextChildren.filter(Boolean) as LayoutEditorNode[],
      };
    }
  }

  return node;
}

async function handleNodeRemove(payload: { parentNodeId: string | undefined; nodeIndex: number }) {
  if (payload.parentNodeId === undefined && payload.nodeIndex === 0) {
    const confirmed = await confirm('确认清空', '确定要清空整个布局吗？所有面板将返回可用列表。');
    if (confirmed) {
      localLayoutTree.value = null;
      addPaneToAvailableList('terminal');
    }
    return;
  }

  if (payload.parentNodeId) {
    localLayoutTree.value = findAndRemoveNode(localLayoutTree.value, payload.parentNodeId, payload.nodeIndex);
  }
}

function removeSidebarPane(side: 'left' | 'right', index: number) {
  const removed = localSidebarPanes.value[side].splice(index, 1)[0];
  if (removed === 'terminal') {
    addPaneToAvailableList('terminal');
  }
}

function onDraggableChange(event: any, side: 'left' | 'right') {
  if (event.added) {
    const addedElement = event.added.element;
    const targetList = localSidebarPanes.value[side];
    const addedIndex = event.added.newIndex;
    if (
      targetList &&
      typeof addedElement === 'object' &&
      addedElement !== null &&
      addedElement.type === 'pane' &&
      typeof addedElement.component === 'string'
    ) {
      targetList.splice(addedIndex, 1, addedElement.component);
    }
  }
}

function handleAvailablePaneDragEnd(event: any) {
  if (event.to !== event.from) {
    const paneName = event.oldIndex !== undefined ? localAvailablePanes.value[event.oldIndex] : null;
    if (paneName === 'terminal') {
      removePaneFromAvailableList('terminal');
    }
  }
}
</script>

<template>
  <div
    v-if="visible"
    class="layout-configurator-mask"
    @click.self="closeDialog"
  >
    <div class="layout-configurator-dialog">
      <header class="dialog-header">
        <h2>布局管理器</h2>
        <button class="dialog-close" title="关闭" @click="closeDialog">×</button>
      </header>

      <main class="dialog-main-grid">
        <section class="available-pane-section">
          <h3>可用面板</h3>
          <draggable
            :list="localAvailablePanes"
            tag="ul"
            class="available-pane-list"
            :item-key="(element: PaneName) => element"
            :group="{ name: 'layout-items', pull: 'clone', put: false }"
            :sort="false"
            :clone="clonePane"
            ghost-class="sortable-ghost"
            chosen-class="sortable-chosen"
            drag-class="sortable-drag"
            @end="handleAvailablePaneDragEnd"
          >
            <template #item="{ element }: { element: PaneName }">
              <li class="available-pane-item">
                <i class="fas fa-grip-vertical"></i>
                <span>{{ paneLabels[element] || element }}</span>
              </li>
            </template>

            <template #footer>
              <li v-if="localAvailablePanes.length === 0" class="empty-tip available-empty-tip">
                所有面板都已在布局中
              </li>
            </template>
          </draggable>
        </section>

        <div class="layout-preview-and-sidebars">
          <section class="layout-preview-section">
            <div class="layout-preview-title-row">
              <h3>主布局预览（拖拽到此处）</h3>

              <div class="layout-lock-switch-wrap">
                <label id="layout-lock-label" class="layout-lock-label" @click="handleLayoutLockChange">锁定布局</label>
                <button
                  type="button"
                  class="layout-lock-switch"
                  :class="{ active: layoutLockedBoolean }"
                  role="switch"
                  :aria-checked="layoutLockedBoolean"
                  aria-labelledby="layout-lock-label"
                  @click="handleLayoutLockChange"
                >
                  <span class="layout-lock-switch-knob" :class="{ active: layoutLockedBoolean }"></span>
                </button>
              </div>
            </div>

            <div class="layout-preview-body">
              <LayoutNodeEditor
                v-if="localLayoutTree"
                :node="localLayoutTree"
                :parent-node="null"
                :node-index="0"
                :pane-labels="paneLabels"
                :group="'layout-items'"
                class="layout-editor-root"
                @update:node="handleNodeUpdate"
                @removeNode="handleNodeRemove"
              />

              <p v-else class="empty-tip main-layout-empty-tip">
                布局为空，请从左侧拖拽面板或添加容器。
              </p>
            </div>

            <div class="layout-preview-actions">
              <button class="secondary-btn" @click="resetToDefault">恢复默认</button>
            </div>
          </section>

          <div class="sidebar-sections-row">
            <section class="sidebar-panel-section">
              <h3>左侧栏面板</h3>
              <draggable
                :list="localSidebarPanes.left"
                tag="ul"
                class="sidebar-pane-list"
                :item-key="(element: PaneName, index: number) => `left-${element}-${index}`"
                group="layout-items"
                :sort="true"
                ghost-class="sortable-ghost"
                chosen-class="sortable-chosen"
                drag-class="sortable-drag"
                @change="(event) => onDraggableChange(event, 'left')"
              >
                <template #item="{ element, index }: { element: PaneName; index: number }">
                  <li class="sidebar-pane-item">
                    <div class="sidebar-pane-label-wrap">
                      <i class="fas fa-grip-vertical"></i>
                      <span>{{ paneLabels[element] || element }}</span>
                    </div>
                    <button class="sidebar-item-remove" title="移除" @click="removeSidebarPane('left', index)">×</button>
                  </li>
                </template>

                <template #footer>
                  <li v-if="localSidebarPanes.left.length === 0" class="empty-tip">
                    从可用面板拖拽到此处
                  </li>
                </template>
              </draggable>
            </section>

            <section class="sidebar-panel-section">
              <h3>右侧栏面板</h3>
              <draggable
                :list="localSidebarPanes.right"
                tag="ul"
                class="sidebar-pane-list"
                :item-key="(element: PaneName, index: number) => `right-${element}-${index}`"
                group="layout-items"
                :sort="true"
                ghost-class="sortable-ghost"
                chosen-class="sortable-chosen"
                drag-class="sortable-drag"
                @change="(event) => onDraggableChange(event, 'right')"
              >
                <template #item="{ element, index }: { element: PaneName; index: number }">
                  <li class="sidebar-pane-item">
                    <div class="sidebar-pane-label-wrap">
                      <i class="fas fa-grip-vertical"></i>
                      <span>{{ paneLabels[element] || element }}</span>
                    </div>
                    <button class="sidebar-item-remove" title="移除" @click="removeSidebarPane('right', index)">×</button>
                  </li>
                </template>

                <template #footer>
                  <li v-if="localSidebarPanes.right.length === 0" class="empty-tip">
                    从可用面板拖拽到此处
                  </li>
                </template>
              </draggable>
            </section>
          </div>
        </div>
      </main>

      <footer class="dialog-footer">
        <button class="secondary-btn" @click="closeDialog">取消</button>
        <button class="primary-btn" :disabled="!isModified" @click="saveLayout">保存{{ isModified ? '*' : '' }}</button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.layout-configurator-mask {
  position: fixed;
  inset: 0;
  z-index: 9500;
  background: rgba(8, 10, 17, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px;
}

.layout-configurator-dialog {
  width: min(980px, 95vw);
  max-height: 90vh;
  border-radius: 10px;
  border: 1px solid rgba(126, 131, 168, 0.45);
  background: #2c2f43;
  color: #d7dcf5;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.45);
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid rgba(126, 131, 168, 0.35);
}

.dialog-header h2 {
  margin: 0;
  font-size: 34px;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.dialog-close {
  border: none;
  background: transparent;
  color: #aeb6da;
  cursor: pointer;
  font-size: 24px;
  line-height: 1;
  width: 28px;
  height: 28px;
  border-radius: 6px;
}

.dialog-close:hover {
  color: #eff2ff;
  background: rgba(255, 255, 255, 0.08);
}

.dialog-main-grid {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 14px 16px;
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr);
  gap: 14px;
}

.available-pane-section,
.layout-preview-section,
.sidebar-panel-section {
  border: 1px solid rgba(126, 131, 168, 0.35);
  border-radius: 8px;
  background: rgba(32, 36, 55, 0.78);
}

.available-pane-section {
  padding: 12px;
  display: flex;
  flex-direction: column;
}

.available-pane-section h3,
.layout-preview-section h3,
.sidebar-panel-section h3 {
  margin: 0 0 10px;
  font-size: 16px;
  color: #e6ebff;
  font-weight: 600;
}

.available-pane-list,
.sidebar-pane-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.available-pane-list {
  flex: 1;
  min-height: 120px;
}

.available-pane-item,
.sidebar-pane-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-height: 34px;
  border: 1px solid rgba(132, 139, 179, 0.45);
  border-radius: 6px;
  background: rgba(66, 72, 101, 0.62);
  color: #d8ddf5;
  font-size: 13px;
  padding: 0 10px;
  margin-bottom: 8px;
}

.available-pane-item {
  justify-content: flex-start;
  cursor: grab;
}

.available-pane-item i,
.sidebar-pane-label-wrap i {
  color: #9aa3c8;
  margin-right: 8px;
}

.layout-preview-and-sidebars {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.layout-preview-section {
  flex: 1;
  min-height: 0;
  padding: 12px;
  display: flex;
  flex-direction: column;
}

.layout-preview-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
}

.layout-preview-title-row h3 {
  margin: 0;
}

.layout-lock-switch-wrap {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.layout-lock-label {
  font-size: 13px;
  color: #c0c8ea;
  cursor: pointer;
  user-select: none;
}

.layout-lock-switch {
  border: none;
  width: 44px;
  height: 24px;
  border-radius: 999px;
  background: rgba(133, 139, 171, 0.6);
  cursor: pointer;
  padding: 0 2px;
  display: inline-flex;
  align-items: center;
  transition: background-color 0.2s ease;
}

.layout-lock-switch.active {
  background: #b489ff;
}

.layout-lock-switch-knob {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #ffffff;
  transform: translateX(0);
  transition: transform 0.2s ease;
}

.layout-lock-switch-knob.active {
  transform: translateX(20px);
}

.layout-preview-body {
  flex: 1;
  min-height: 240px;
  border: 2px dashed rgba(132, 139, 179, 0.52);
  border-radius: 8px;
  background: rgba(50, 55, 80, 0.86);
  padding: 10px;
  overflow: auto;
}

.layout-editor-root {
  min-height: 210px;
}

.layout-preview-actions {
  margin-top: 10px;
  display: flex;
  justify-content: flex-start;
}

.sidebar-sections-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.sidebar-panel-section {
  padding: 10px;
  min-height: 190px;
  display: flex;
  flex-direction: column;
}

.sidebar-pane-list {
  flex: 1;
  min-height: 120px;
  border: 1px dashed rgba(132, 139, 179, 0.45);
  border-radius: 8px;
  background: rgba(27, 30, 47, 0.6);
  padding: 8px;
}

.sidebar-pane-item {
  margin-bottom: 8px;
}

.sidebar-pane-label-wrap {
  display: inline-flex;
  align-items: center;
  min-width: 0;
  overflow: hidden;
}

.sidebar-pane-label-wrap span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-item-remove {
  border: none;
  background: rgba(132, 139, 179, 0.28);
  color: #bfc7ea;
  width: 20px;
  height: 20px;
  border-radius: 4px;
  cursor: pointer;
  line-height: 1;
}

.sidebar-item-remove:hover {
  color: #ffffff;
  background: rgba(243, 139, 168, 0.55);
}

.dialog-footer {
  border-top: 1px solid rgba(126, 131, 168, 0.35);
  padding: 12px 16px;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.secondary-btn,
.primary-btn {
  min-width: 84px;
  height: 32px;
  border-radius: 6px;
  border: 1px solid rgba(132, 139, 179, 0.48);
  font-size: 13px;
  cursor: pointer;
  padding: 0 14px;
}

.secondary-btn {
  background: rgba(86, 92, 124, 0.7);
  color: #e3e8ff;
}

.secondary-btn:hover {
  background: rgba(98, 104, 140, 0.85);
}

.primary-btn {
  background: rgba(180, 137, 255, 0.9);
  border-color: rgba(180, 137, 255, 0.95);
  color: #f7f2ff;
}

.primary-btn:hover:not(:disabled) {
  background: rgba(190, 149, 255, 0.98);
}

.primary-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.empty-tip {
  min-height: 68px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  font-size: 12px;
  color: #8f98bf;
  border: 1px dashed rgba(132, 139, 179, 0.45);
  border-radius: 8px;
  padding: 10px;
}

.main-layout-empty-tip {
  min-height: 188px;
}

.available-empty-tip {
  min-height: 96px;
}

.sortable-ghost {
  opacity: 0.42;
}

.sortable-chosen,
.sortable-drag {
  opacity: 0.9;
}

@media (max-width: 1100px) {
  .dialog-main-grid {
    grid-template-columns: 1fr;
  }

  .sidebar-sections-row {
    grid-template-columns: 1fr;
  }
}
</style>
