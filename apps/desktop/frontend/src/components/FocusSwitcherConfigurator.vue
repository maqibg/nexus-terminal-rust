<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="focus-switcher-overlay"
      @click.self="requestClose"
      @keydown.esc.prevent="requestClose"
    >
      <section class="focus-switcher-dialog" role="dialog" aria-modal="true">
        <header class="dialog-header">
          <h2>配置焦点切换器</h2>
          <button class="close-btn" type="button" @click="requestClose" aria-label="关闭">&times;</button>
        </header>

        <div class="dialog-body">
          <p class="hint">提示：按下 Alt 键可在配置的输入源之间快速切换焦点。</p>

          <div class="sequence-layout">
            <section class="panel">
              <h3>可用输入源</h3>
              <div
                class="list-zone"
                @dragover.prevent
                @drop="onDropToAvailable"
              >
                <div
                  v-for="item in availableInputs"
                  :key="item.id"
                  class="focus-item"
                  draggable="true"
                  @dragstart="onDragStart($event, item.id, 'available')"
                  @dragend="resetDrag"
                >
                  <i class="fas fa-grip-vertical drag-icon"></i>
                  <span>{{ item.label }}</span>
                </div>
                <p v-if="availableInputs.length === 0" class="empty-text">所有输入源都已配置</p>
              </div>
            </section>

            <section class="panel">
              <h3>已配置序列 (拖拽排序)</h3>
              <div
                class="list-zone"
                @dragover.prevent
                @drop="onDropToSequence($event, localSequence.length)"
              >
                <div
                  v-for="(item, index) in localSequence"
                  :key="item.id"
                  class="focus-item"
                  draggable="true"
                  @dragstart="onDragStart($event, item.id, 'sequence')"
                  @dragover.prevent
                  @drop="onDropToSequence($event, index)"
                  @dragend="resetDrag"
                >
                  <i class="fas fa-grip-vertical drag-icon"></i>
                  <span>{{ item.label }}</span>
                  <button
                    class="remove-btn"
                    type="button"
                    title="移除"
                    @click="removeFromSequence(index)"
                  >
                    &times;
                  </button>
                </div>
                <p v-if="localSequence.length === 0" class="empty-text">从左侧拖拽输入源到此处</p>
              </div>
            </section>
          </div>

          <section class="panel shortcuts">
            <h3>快捷键设置</h3>
            <div class="shortcut-grid">
              <div
                v-for="item in focusSwitcherStore.availableInputs"
                :key="item.id"
                class="shortcut-row"
              >
                <span class="shortcut-label">{{ item.label }}</span>
                <input
                  v-model="localItemConfigs[item.id].shortcut"
                  class="shortcut-input"
                  type="text"
                  placeholder="例如 Alt+K"
                  @keydown.prevent="captureShortcut($event, item.id)"
                />
              </div>
            </div>
          </section>
        </div>

        <footer class="dialog-footer">
          <button class="btn btn-cancel" type="button" @click="requestClose">取消</button>
          <button class="btn btn-save" type="button" :disabled="!hasChanges" @click="saveConfiguration">保存</button>
        </footer>
      </section>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import {
  useFocusSwitcherStore,
  type FocusItemConfig,
  type FocusSwitcherFullConfig,
  type FocusableInput,
} from '@/stores/focusSwitcher';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const focusSwitcherStore = useFocusSwitcherStore();

const localSequence = ref<FocusableInput[]>([]);
const localItemConfigs = ref<Record<string, FocusItemConfig>>(
  Object.fromEntries(focusSwitcherStore.availableInputs.map((input) => [input.id, {}])) as Record<string, FocusItemConfig>,
);
const hasChanges = ref(false);
const originalSnapshot = ref('');

const draggingId = ref<string | null>(null);
const draggingFrom = ref<'available' | 'sequence' | null>(null);

const availableInputs = computed(() => {
  const selectedIds = new Set(localSequence.value.map((item) => item.id));
  return focusSwitcherStore.availableInputs.filter((item) => !selectedIds.has(item.id));
});

function normalizeShortcut(value: string | undefined): string | undefined {
  if (!value) {
    return undefined;
  }
  const normalized = value.trim().toUpperCase();
  if (!/^ALT\+[A-Z0-9]$/.test(normalized)) {
    return undefined;
  }
  return normalized.replace(/^ALT\+/, 'Alt+');
}

function buildSnapshotConfig(): FocusSwitcherFullConfig {
  const shortcuts: Record<string, FocusItemConfig> = {};
  for (const input of focusSwitcherStore.availableInputs) {
    const shortcut = normalizeShortcut(localItemConfigs.value[input.id]?.shortcut);
    if (shortcut) {
      shortcuts[input.id] = { shortcut };
    }
  }

  return {
    sequence: localSequence.value.map((item) => item.id),
    shortcuts,
  };
}

function syncChangedFlag(): void {
  if (!props.visible) {
    return;
  }
  hasChanges.value = JSON.stringify(buildSnapshotConfig()) !== originalSnapshot.value;
}

function resetDrag(): void {
  draggingId.value = null;
  draggingFrom.value = null;
}

function removeFromSequence(index: number): void {
  if (index >= 0 && index < localSequence.value.length) {
    localSequence.value.splice(index, 1);
  }
}

function getDraggedId(event: DragEvent): string | null {
  return draggingId.value ?? event.dataTransfer?.getData('text/plain') ?? null;
}

function onDragStart(event: DragEvent, id: string, from: 'available' | 'sequence'): void {
  draggingId.value = id;
  draggingFrom.value = from;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', id);
  }
}

function onDropToAvailable(event: DragEvent): void {
  event.preventDefault();
  const id = getDraggedId(event);
  if (!id) {
    resetDrag();
    return;
  }

  const index = localSequence.value.findIndex((item) => item.id === id);
  if (index >= 0) {
    localSequence.value.splice(index, 1);
  }

  resetDrag();
}

function onDropToSequence(event: DragEvent, targetIndex: number): void {
  event.preventDefault();
  const id = getDraggedId(event);
  if (!id) {
    resetDrag();
    return;
  }

  const input = focusSwitcherStore.availableInputs.find((item) => item.id === id);
  if (!input) {
    resetDrag();
    return;
  }

  const fromIndex = localSequence.value.findIndex((item) => item.id === id);
  if (fromIndex >= 0) {
    localSequence.value.splice(fromIndex, 1);
  }

  let insertIndex = targetIndex;
  if (fromIndex >= 0 && fromIndex < targetIndex) {
    insertIndex -= 1;
  }

  insertIndex = Math.max(0, Math.min(insertIndex, localSequence.value.length));
  localSequence.value.splice(insertIndex, 0, input);

  resetDrag();
}

function captureShortcut(event: KeyboardEvent, id: string): void {
  const targetConfig = localItemConfigs.value[id];
  if (!targetConfig) {
    return;
  }

  if (event.key === 'Backspace' || event.key === 'Delete') {
    targetConfig.shortcut = undefined;
    return;
  }

  if (['Alt', 'Control', 'Shift', 'Meta'].includes(event.key)) {
    return;
  }

  if (!event.altKey || event.ctrlKey || event.shiftKey || event.metaKey) {
    return;
  }

  let key = event.key;
  if (key.length === 1) {
    key = key.toUpperCase();
  }

  if (/^[A-Z0-9]$/.test(key)) {
    targetConfig.shortcut = `Alt+${key}`;
  }
}

function saveConfiguration(): void {
  const config = buildSnapshotConfig();
  focusSwitcherStore.updateConfiguration(config);
  originalSnapshot.value = JSON.stringify(config);
  hasChanges.value = false;
  emit('close');
}

function requestClose(): void {
  if (hasChanges.value && !window.confirm('有未保存的更改，确定要关闭吗？')) {
    return;
  }
  emit('close');
}

watch(
  () => props.visible,
  async (visible) => {
    if (!visible) {
      resetDrag();
      return;
    }

    await focusSwitcherStore.loadConfigurationFromBackend();

    const inputMap = new Map(focusSwitcherStore.availableInputs.map((input) => [input.id, input]));
    localSequence.value = focusSwitcherStore.sequenceOrder
      .map((id) => inputMap.get(id))
      .filter((item): item is FocusableInput => !!item);

    const localConfigs: Record<string, FocusItemConfig> = {};
    for (const input of focusSwitcherStore.availableInputs) {
      localConfigs[input.id] = {
        shortcut: focusSwitcherStore.itemConfigs[input.id]?.shortcut,
      };
    }
    localItemConfigs.value = localConfigs;

    originalSnapshot.value = JSON.stringify(buildSnapshotConfig());
    hasChanges.value = false;
  },
  { immediate: true },
);

watch([localSequence, localItemConfigs], syncChangedFlag, { deep: true });
</script>

<style scoped>
.focus-switcher-overlay {
  position: fixed;
  inset: 0;
  z-index: 12000;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
  background: rgba(10, 12, 18, 0.62);
  backdrop-filter: blur(2px);
}

.focus-switcher-dialog {
  width: min(980px, calc(100vw - 40px));
  max-height: calc(100vh - 40px);
  display: flex;
  flex-direction: column;
  border: 1px solid #4c5368;
  border-radius: 10px;
  background: #2b3040;
  color: #d6dbf0;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.55);
  overflow: hidden;
}

.dialog-header,
.dialog-footer {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: #3a3f4f;
}

.dialog-header {
  border-bottom: 1px solid #52586c;
}

.dialog-header h2 {
  margin: 0;
  font-size: 20px;
  letter-spacing: 0.5px;
  font-weight: 700;
  color: #f2f5ff;
}

.close-btn {
  border: none;
  background: transparent;
  color: #aeb4ca;
  font-size: 20px;
  line-height: 1;
  width: 28px;
  height: 28px;
  cursor: pointer;
}

.close-btn:hover {
  color: #ffffff;
}

.dialog-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
  overflow: auto;
  background: #2a2f3d;
}

.hint {
  margin: 0;
  font-size: 13px;
  font-style: italic;
  color: #b9bfd6;
}

.sequence-layout {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  min-height: 260px;
}

.panel {
  min-height: 0;
  display: flex;
  flex-direction: column;
  border: 1px solid #4e5568;
  border-radius: 4px;
  background: #2d3342;
  padding: 12px;
}

.panel h3 {
  margin: 0 0 10px;
  font-size: 15px;
  font-weight: 600;
  color: #dbe0f4;
}

.list-zone {
  flex: 1;
  min-height: 170px;
  border: 1px dashed #6a7084;
  border-radius: 4px;
  padding: 8px;
  background: rgba(18, 22, 30, 0.25);
  overflow: auto;
}

.focus-item {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 36px;
  margin-bottom: 8px;
  border: 1px solid #545b70;
  border-radius: 4px;
  padding: 0 10px;
  color: #d6dbf0;
  font-size: 13px;
  background: #323849;
  cursor: grab;
}

.focus-item:last-child {
  margin-bottom: 0;
}

.focus-item:active {
  cursor: grabbing;
}

.drag-icon {
  color: #9ca4bf;
}

.remove-btn {
  margin-left: auto;
  border: none;
  background: transparent;
  color: #aeb5cd;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
}

.remove-btn:hover {
  color: #ff8080;
}

.empty-text {
  margin: 18px 0;
  text-align: center;
  color: #98a0b9;
  font-size: 13px;
}

.shortcuts {
  max-height: 260px;
}

.shortcut-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 12px;
  overflow: auto;
}

.shortcut-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border: 1px solid #4e5568;
  border-radius: 4px;
  padding: 7px 9px;
  gap: 8px;
  background: #2e3444;
}

.shortcut-label {
  color: #ced3e8;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shortcut-input {
  width: 92px;
  flex-shrink: 0;
  text-align: center;
  font-size: 12px;
  color: #edf1ff;
  background: #242a38;
  border: 1px solid #5b647a;
  border-radius: 4px;
  padding: 4px 6px;
}

.shortcut-input:focus {
  outline: none;
  border-color: #9fb7e8;
}

.shortcut-input::placeholder {
  color: #7d869f;
}

.dialog-footer {
  justify-content: flex-end;
  gap: 10px;
  border-top: 1px solid #52586c;
}

.btn {
  border: 1px solid transparent;
  border-radius: 4px;
  min-width: 70px;
  height: 32px;
  padding: 0 14px;
  font-size: 13px;
  cursor: pointer;
}

.btn-cancel {
  background: #8f95a8;
  color: #eef2ff;
}

.btn-cancel:hover {
  background: #a0a7bc;
}

.btn-save {
  background: #c18cff;
  color: #fff;
}

.btn-save:hover {
  background: #d39dff;
}

.btn-save:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

@media (max-width: 960px) {
  .focus-switcher-dialog {
    width: calc(100vw - 24px);
    max-height: calc(100vh - 24px);
  }

  .sequence-layout {
    grid-template-columns: 1fr;
    min-height: 0;
  }

  .shortcut-grid {
    grid-template-columns: 1fr;
  }
}
</style>