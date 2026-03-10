<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="quick-command-form-overlay"
      @click.self="emit('close')"
    >
      <div class="quick-command-form-card">
        <h2 class="form-title">{{ isEditing ? '编辑快捷指令' : '添加快捷指令' }}</h2>

        <div class="form-body">
          <aside class="variables-pane">
            <h3 class="pane-title">变量管理</h3>

            <div class="variables-list">
              <div v-if="!localVariables.length" class="no-variables">
                暂无变量。点击下方按钮添加。
              </div>

              <div
                v-for="variable in localVariables"
                :key="variable.id"
                class="variable-card"
              >
                <input
                  v-model="variable.name"
                  type="text"
                  class="field-input"
                  placeholder="变量名"
                />
                <textarea
                  v-model="variable.value"
                  rows="2"
                  class="field-input field-textarea"
                  placeholder="变量值"
                ></textarea>
                <button
                  type="button"
                  class="danger-btn"
                  @click="deleteVariable(variable.id)"
                >
                  删除
                </button>
              </div>
            </div>

            <button
              type="button"
              class="add-variable-btn"
              @click="addVariable"
            >
              + 添加变量
            </button>
          </aside>

          <form class="editor-pane" @submit.prevent="handleSave">
            <div class="editor-fields">
              <div class="field-row">
                <label for="quick-command-name">名称</label>
                <input
                  id="quick-command-name"
                  v-model="name"
                  type="text"
                  class="field-input"
                  placeholder="可选，用于快速识别"
                />
              </div>

              <div class="field-row command-field-row">
                <label for="quick-command-content">指令: <em>*</em></label>
                <textarea
                  id="quick-command-content"
                  v-model="command"
                  class="field-input command-textarea"
                  placeholder='例如：echo "Hello,${USERNAME}"'
                  required
                ></textarea>
                <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
              </div>

              <div class="field-row">
                <label for="quick-command-tags">标签</label>
                <TagInput
                  id="quick-command-tags"
                  v-model="selectedTags"
                  :available-tags="availableTags.map((tag) => tag.name)"
                />
              </div>
            </div>

            <div class="form-footer">
              <button type="button" class="form-btn cancel" @click="emit('close')">取消</button>
              <button type="button" class="form-btn execute" @click="handleExecute">执行</button>
              <button type="submit" class="form-btn save">{{ submitButtonText }}</button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import TagInput from './TagInput.vue';
import { sshApi, type QuickCommand } from '@/lib/api';
import { useCommandHistoryStore } from '@/stores/commandHistory';
import { useQuickCommandTagsStore } from '@/stores/quickCommandTags';
import { useSessionStore } from '@/stores/session';
import { useUINotificationStore } from '@/stores/uiNotifications';

interface LocalVariable {
  id: string;
  name: string;
  value: string;
}

const props = defineProps<{ visible: boolean; editData?: QuickCommand | null }>();
const emit = defineEmits<{ close: []; save: [data: { name: string; command: string; variables?: string; tags?: string[] }] }>();

const name = ref('');
const command = ref('');
const selectedTags = ref<string[]>([]);
const localVariables = ref<LocalVariable[]>([]);
const errorMessage = ref('');

const tagsStore = useQuickCommandTagsStore();
const sessionStore = useSessionStore();
const historyStore = useCommandHistoryStore();
const notificationsStore = useUINotificationStore();
const { items: availableTags } = storeToRefs(tagsStore);

const isEditing = computed(() => !!props.editData?.id);
const submitButtonText = computed(() => (isEditing.value ? '保存' : '添加'));

onMounted(() => {
  void tagsStore.fetchAll();
});

watch(
  () => props.visible,
  (visible) => {
    if (!visible) {
      return;
    }

    errorMessage.value = '';
    if (props.editData) {
      name.value = props.editData.name || '';
      command.value = props.editData.command || '';
      selectedTags.value = [...(props.editData.tags ?? [])];
      localVariables.value = parseVariables(props.editData.variables);
      return;
    }

    name.value = '';
    command.value = '';
    selectedTags.value = [];
    localVariables.value = [];
  },
  { immediate: true },
);

function parseVariables(raw?: string): LocalVariable[] {
  if (!raw) {
    return [];
  }

  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return [];
    }

    return Object.entries(parsed as Record<string, unknown>).map(([varName, varValue]) => ({
      id: createVariableId(),
      name: varName,
      value: String(varValue ?? ''),
    }));
  } catch {
    return [];
  }
}

function createVariableId(): string {
  return `var-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

function addVariable() {
  localVariables.value.push({
    id: createVariableId(),
    name: '',
    value: '',
  });
}

function deleteVariable(variableId: string) {
  localVariables.value = localVariables.value.filter((item) => item.id !== variableId);
}

function escapeRegExp(raw: string): string {
  return raw.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function buildVariablesObject(): Record<string, string> {
  return localVariables.value.reduce((acc, current) => {
    const variableName = current.name.trim();
    if (!variableName) {
      return acc;
    }

    acc[variableName] = current.value;
    return acc;
  }, {} as Record<string, string>);
}

function serializeVariables(): string | undefined {
  const payload = buildVariablesObject();
  if (!Object.keys(payload).length) {
    return undefined;
  }
  return JSON.stringify(payload);
}

function buildNameFallback(commandText: string): string {
  const firstLine = commandText.split(/\r?\n/)[0]?.trim() ?? '';
  if (!firstLine) {
    return '快捷指令';
  }

  return firstLine.length > 28 ? `${firstLine.slice(0, 28)}...` : firstLine;
}

function applyVariables(baseCommand: string): string {
  const variables = buildVariablesObject();
  let result = baseCommand;

  for (const [varName, varValue] of Object.entries(variables)) {
    const placeholder = new RegExp(`\\$\\{${escapeRegExp(varName)}\\}`, 'g');
    result = result.replace(placeholder, varValue);
  }

  return result;
}

function findUndefinedVariables(baseCommand: string): string[] {
  const placeholders = baseCommand.match(/\$\{[^}]+\}/g) || [];
  const variables = buildVariablesObject();
  const undefinedVars: string[] = [];

  for (const placeholder of placeholders) {
    const varName = placeholder.slice(2, -1).trim();
    if (!varName || Object.prototype.hasOwnProperty.call(variables, varName)) {
      continue;
    }
    if (!undefinedVars.includes(varName)) {
      undefinedVars.push(varName);
    }
  }

  return undefinedVars;
}

function handleSave() {
  const commandText = command.value.trim();
  if (!commandText) {
    errorMessage.value = '指令不能为空';
    return;
  }

  errorMessage.value = '';
  emit('save', {
    name: name.value.trim() || buildNameFallback(commandText),
    command: commandText,
    variables: serializeVariables(),
    tags: selectedTags.value.length ? [...selectedTags.value] : undefined,
  });
}

async function handleExecute() {
  const commandText = command.value.trim();
  if (!commandText) {
    errorMessage.value = '指令不能为空';
    return;
  }

  errorMessage.value = '';
  const sessionId = sessionStore.activeSessionId;
  if (!sessionId) {
    notificationsStore.addNotification('warning', '没有活动会话，无法执行命令');
    return;
  }

  const finalCommand = applyVariables(commandText);
  const undefinedVariables = findUndefinedVariables(commandText);
  if (undefinedVariables.length) {
    notificationsStore.addNotification('warning', `以下变量未定义：${undefinedVariables.join(', ')}`);
  }

  const payload = btoa(unescape(encodeURIComponent(`${finalCommand}\n`)));

  try {
    await sshApi.write(sessionId, payload);
    try {
      await historyStore.add(finalCommand, sessionId, sessionStore.activeSession?.connectionId);
      window.dispatchEvent(new Event('nexus:command-history-updated'));
    } catch {
      // ignore history failure
    }

    notificationsStore.addNotification('success', '命令已发送到终端');
    emit('close');
  } catch {
    notificationsStore.addNotification('error', '命令发送失败，请稍后重试');
  }
}
</script>

<style scoped>
.quick-command-form-overlay {
  position: fixed;
  inset: 0;
  z-index: 4300;
  background: rgba(4, 6, 12, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px;
}

.quick-command-form-card {
  width: min(1040px, 95vw);
  height: min(700px, 86vh);
  border-radius: 14px;
  border: 1px solid rgba(138, 149, 180, 0.28);
  background: #222733;
  color: #e3ebff;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.55);
  padding: 20px;
  display: flex;
  flex-direction: column;
  min-height: 520px;
}

.form-title {
  margin: 0 0 16px;
  text-align: center;
  font-size: calc(36px + var(--ui-font-size-offset));
  line-height: 1.1;
  font-weight: 700;
  letter-spacing: 0.3px;
}

.form-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 0.34fr 0.66fr;
  gap: 18px;
}

.variables-pane {
  min-height: 0;
  border-right: 1px solid rgba(138, 149, 180, 0.24);
  padding-right: 16px;
  display: flex;
  flex-direction: column;
}

.pane-title {
  margin: 0;
  color: #c7d2ee;
  font-size: calc(24px + var(--ui-font-size-offset));
  font-weight: 600;
}

.variables-list {
  flex: 1;
  min-height: 0;
  margin-top: 14px;
  padding-right: 4px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.no-variables {
  font-size: calc(13px + var(--ui-font-size-offset));
  line-height: 1.5;
  color: #9cabcf;
  border: 1px dashed rgba(138, 149, 180, 0.3);
  border-radius: 10px;
  padding: 10px 12px;
}

.variable-card {
  background: rgba(15, 19, 30, 0.38);
  border: 1px solid rgba(138, 149, 180, 0.3);
  border-radius: 10px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-input {
  width: 100%;
  border: 1px solid rgba(138, 149, 180, 0.35);
  border-radius: 8px;
  background: #1d2230;
  color: #e3ebff;
  font-size: calc(13px + var(--ui-font-size-offset));
  padding: 8px 10px;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.field-input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.field-input::placeholder {
  color: #7f8aa8;
}

.field-textarea {
  resize: vertical;
  min-height: 52px;
  font-family: 'Cascadia Mono', 'Consolas', monospace;
}

.danger-btn {
  width: 100%;
  border: 1px solid rgba(243, 139, 168, 0.5);
  border-radius: 8px;
  background: transparent;
  color: var(--red, #f38ba8);
  height: 30px;
  font-size: calc(12px + var(--ui-font-size-offset));
  cursor: pointer;
  transition: all 0.15s;
}

.danger-btn:hover {
  background: rgba(243, 139, 168, 0.12);
}

.add-variable-btn {
  margin-top: 12px;
  height: 34px;
  border: 1px solid rgba(203, 166, 247, 0.55);
  border-radius: 8px;
  background: transparent;
  color: var(--mauve, #cba6f7);
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
}

.add-variable-btn:hover {
  background: rgba(203, 166, 247, 0.12);
}

.editor-pane {
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.editor-fields {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding-right: 4px;
}

.field-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-row > label {
  font-size: calc(14px + var(--ui-font-size-offset));
  color: #c4d1f0;
  font-weight: 500;
}

.field-row > label em {
  font-style: normal;
  color: var(--red, #f38ba8);
}

.command-field-row {
  flex: 1;
  min-height: 0;
}

.command-textarea {
  flex: 1;
  min-height: 180px;
  max-height: 100%;
  resize: none;
  white-space: pre;
  overflow: auto;
  font-family: 'Cascadia Mono', 'Consolas', monospace;
}

.error-message {
  margin: 0;
  color: var(--red, #f38ba8);
  font-size: calc(12px + var(--ui-font-size-offset));
}

.form-footer {
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid rgba(138, 149, 180, 0.24);
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.form-btn {
  min-width: 72px;
  height: 34px;
  border-radius: 8px;
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.15s;
}

.form-btn.cancel {
  background: transparent;
  color: #c5cde4;
  border-color: rgba(138, 149, 180, 0.34);
}

.form-btn.cancel:hover {
  background: rgba(138, 149, 180, 0.12);
}

.form-btn.execute {
  background: rgba(166, 227, 161, 0.86);
  border-color: rgba(166, 227, 161, 0.86);
  color: #1b2520;
}

.form-btn.execute:hover {
  background: rgba(166, 227, 161, 1);
}

.form-btn.save {
  background: var(--mauve, #cba6f7);
  border-color: var(--mauve, #cba6f7);
  color: #1f1733;
}

.form-btn.save:hover {
  filter: brightness(1.05);
}

@media (max-width: 980px) {
  .quick-command-form-card {
    width: min(980px, 96vw);
    height: min(760px, 92vh);
  }

  .form-body {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .variables-pane {
    border-right: none;
    border-bottom: 1px solid rgba(138, 149, 180, 0.24);
    padding-right: 0;
    padding-bottom: 12px;
    max-height: 220px;
  }

  .pane-title {
    font-size: calc(20px + var(--ui-font-size-offset));
  }

  .form-title {
    font-size: calc(30px + var(--ui-font-size-offset));
  }
}
</style>
