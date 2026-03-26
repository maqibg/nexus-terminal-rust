<template>
  <div class="qc-panel">
    <div class="qc-header">
      <span>快捷命令</span>
      <button class="btn-icon" @click="showAdd = true" title="新建">+</button>
    </div>
    <div v-if="loading" class="status">加载中...</div>
    <div v-else class="qc-list">
      <div v-for="cmd in commands" :key="cmd.id" class="qc-item" @click="executeCmd(cmd)" @contextmenu.prevent="showCtx($event, cmd)">
        <span class="qc-name">{{ cmd.name }}</span>
        <span v-if="cmd.description" class="qc-desc">{{ cmd.description }}</span>
        <span class="qc-cmd">{{ cmd.command }}</span>
      </div>
      <div v-if="!commands.length" class="status">暂无快捷命令</div>
    </div>

    <!-- Context menu -->
    <div v-if="ctxCmd" class="ctx-backdrop" @click="ctxCmd = null"></div>
    <div v-if="ctxCmd" class="ctx-menu" :style="{ left: ctxPos.x + 'px', top: ctxPos.y + 'px' }">
      <div class="ctx-item" @click="handleEdit(ctxCmd!)">编辑</div>
      <div class="ctx-item danger" @click="handleDelete(ctxCmd!)">删除</div>
    </div>

    <!-- Add/Edit dialog -->
    <div v-if="showAdd" class="mini-backdrop" @click.self="closeForm">
      <div class="mini-dialog">
        <h4>{{ editingCmd ? '编辑' : '新建' }}快捷命令</h4>
        <label>名称 <input v-model="form.name" /></label>
        <label>描述 <input v-model="form.description" placeholder="可选" /></label>
        <label>命令 <input v-model="form.command" /></label>
        <p class="hint">支持变量占位符: &#123;&#123;var&#125;&#125;</p>
        <div v-if="formError" class="error">{{ formError }}</div>
        <div class="mini-actions">
          <button class="btn-cancel" @click="closeForm">取消</button>
          <button class="btn-save" @click="handleSave">保存</button>
        </div>
      </div>
    </div>

    <!-- Variable input dialog -->
    <div v-if="showVarDialog" class="mini-backdrop" @click.self="showVarDialog = false">
      <div class="mini-dialog">
        <h4>填写变量</h4>
        <label v-for="v in varNames" :key="v">{{ v }} <input v-model="varValues[v]" /></label>
        <div class="mini-actions">
          <button class="btn-cancel" @click="showVarDialog = false">取消</button>
          <button class="btn-save" @click="executeWithVars">执行</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { quickCommandApi } from '@/lib/api';
import type { QuickCommand } from '@/lib/api';
import { useConfirmDialog } from '@/composables/useConfirmDialog';

const emit = defineEmits<{ execute: [command: string] }>();
const { confirm } = useConfirmDialog();

const commands = ref<QuickCommand[]>([]);
const loading = ref(false);
const showAdd = ref(false);
const editingCmd = ref<QuickCommand | null>(null);
const formError = ref('');
const form = reactive({ name: '', command: '', description: '' });

const showVarDialog = ref(false);
const varNames = ref<string[]>([]);
const varValues = reactive<Record<string, string>>({});
let pendingCommand = '';

const ctxCmd = ref<QuickCommand | null>(null);
const ctxPos = ref({ x: 0, y: 0 });

async function load() {
  loading.value = true;
  try { commands.value = await quickCommandApi.list(); } catch { /* ignore */ }
  finally { loading.value = false; }
}

function showCtx(e: MouseEvent, cmd: QuickCommand) {
  ctxCmd.value = cmd;
  ctxPos.value = { x: e.clientX, y: e.clientY };
}

function handleEdit(cmd: QuickCommand) {
  ctxCmd.value = null;
  editingCmd.value = cmd;
  form.name = cmd.name;
  form.command = cmd.command;
  form.description = cmd.description ?? '';
  showAdd.value = true;
}

function closeForm() {
  showAdd.value = false;
  editingCmd.value = null;
  form.name = ''; form.command = ''; form.description = '';
  formError.value = '';
}

function executeCmd(cmd: QuickCommand) {
  const vars = cmd.command.match(/\{\{(\w+)\}\}/g);
  if (vars) {
    varNames.value = [...new Set(vars.map(v => v.slice(2, -2)))];
    for (const v of varNames.value) varValues[v] = '';
    pendingCommand = cmd.command;
    showVarDialog.value = true;
  } else {
    emit('execute', cmd.command);
  }
}

function executeWithVars() {
  let cmd = pendingCommand;
  for (const [k, v] of Object.entries(varValues)) cmd = cmd.replaceAll(`{{${k}}}`, v);
  showVarDialog.value = false;
  emit('execute', cmd);
}

async function handleSave() {
  if (!form.name || !form.command) { formError.value = '名称和命令必填'; return; }
  try {
    if (editingCmd.value) {
      await quickCommandApi.update(editingCmd.value.id, { ...form });
    } else {
      await quickCommandApi.create({ ...form });
    }
    closeForm();
    load();
  } catch (e: any) {
    formError.value = e.message;
  }
}

async function handleDelete(cmd: QuickCommand) {
  ctxCmd.value = null;
  if (!(await confirm('删除快捷命令', `确定删除 “${cmd.name}” 吗？`))) return;
  try { await quickCommandApi.delete(cmd.id); load(); } catch { /* ignore */ }
}

onMounted(load);
</script>

<style scoped>
.qc-panel { display: flex; flex-direction: column; height: 100%; }
.qc-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 6px 10px; border-bottom: 1px solid var(--border); font-size: calc(0.8rem + var(--ui-font-size-offset)); color: var(--text-sub);
}
.btn-icon {
  width: 22px; height: 22px; border-radius: 4px; border: 1px solid var(--border);
  background: transparent; color: var(--text); cursor: pointer; font-size: calc(0.9rem + var(--ui-font-size-offset));
  display: flex; align-items: center; justify-content: center;
}
.btn-icon:hover { background: var(--bg-surface1); }
.qc-list { flex: 1; overflow-y: auto; padding: 4px; }
.qc-item {
  padding: 6px 8px; border-radius: 4px; cursor: pointer;
  display: flex; flex-direction: column; gap: 1px; margin-bottom: 2px;
}
.qc-item:hover { background: var(--bg-surface1); }
.qc-name { font-size: calc(0.8rem + var(--ui-font-size-offset)); color: var(--text); }
.qc-desc { font-size: calc(0.7rem + var(--ui-font-size-offset)); color: var(--text-dim); }
.qc-cmd { font-size: calc(0.7rem + var(--ui-font-size-offset)); color: var(--blue); font-family: monospace; }
.status { padding: 8px; text-align: center; color: var(--text-dim); font-size: calc(0.75rem + var(--ui-font-size-offset)); }

.ctx-backdrop { position: fixed; inset: 0; z-index: 99; }
.ctx-menu {
  position: fixed; z-index: 100; background: var(--ui-menu-bg); border-radius: 6px;
  padding: 4px; box-shadow: 0 4px 12px rgba(0,0,0,0.4); min-width: 80px;
}
.ctx-item { padding: 4px 10px; border-radius: 4px; cursor: pointer; font-size: calc(0.8rem + var(--ui-font-size-offset)); color: var(--text); }
.ctx-item:hover { background: var(--ui-menu-hover); }
.ctx-item.danger { color: var(--red); }

.mini-backdrop {
  position: fixed; inset: 0; background: rgba(0,0,0,0.4);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.mini-dialog {
  background: var(--bg-surface0); border-radius: 8px; padding: 1rem; min-width: 300px;
  border: 1px solid var(--border);
  display: flex; flex-direction: column; gap: 8px;
}
.mini-dialog h4 { margin: 0; font-size: calc(0.9rem + var(--ui-font-size-offset)); color: var(--text); font-weight: 500; }
.mini-dialog label { display: flex; flex-direction: column; gap: 4px; font-size: calc(0.8rem + var(--ui-font-size-offset)); color: var(--text-sub); }
.mini-dialog input {
  padding: 6px 8px; border-radius: 4px; border: 1px solid var(--border);
  background: var(--bg-base); color: var(--text); font-size: calc(0.85rem + var(--ui-font-size-offset)); outline: none;
}
.mini-dialog input:focus { border-color: var(--blue); box-shadow: 0 0 0 1px var(--blue); }
.mini-actions { display: flex; justify-content: flex-end; gap: 6px; }
.btn-cancel {
  padding: 4px 10px; border-radius: 4px; border: 1px solid var(--border);
  background: transparent; color: var(--text-sub); cursor: pointer; font-size: calc(0.8rem + var(--ui-font-size-offset));
}
.btn-cancel:hover { background: var(--bg-surface1); color: var(--text); }
.btn-save {
  padding: 4px 10px; border-radius: 4px; border: none;
  background: var(--blue); color: var(--button-text-color); cursor: pointer; font-weight: 600; font-size: calc(0.8rem + var(--ui-font-size-offset));
}
.btn-save:hover { filter: brightness(1.05); }
.error { color: var(--red); font-size: calc(0.8rem + var(--ui-font-size-offset)); }
</style>
