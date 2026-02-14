<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="emit('close')">
      <div class="dialog-card">
        <div class="dialog-title">{{ formTitle }}</div>
        <div class="form-tabs">
          <button v-for="tab in tabs" :key="tab.key" class="tab-btn" :class="{ active: activeTab === tab.key }" @click="activeTab = tab.key">
            {{ tab.label }}
          </button>
        </div>
        <div class="form-body">
          <AddConnectionFormBasicInfo v-show="activeTab === 'basic'" :form="form.formData" />
          <AddConnectionFormAuth v-show="activeTab === 'auth'" :form="form.formData" />
          <AddConnectionFormAdvanced v-show="activeTab === 'advanced'" :form="form.formData" />
        </div>
        <div class="form-actions">
          <button class="btn btn-cancel" @click="emit('close')">取消</button>
          <button class="btn btn-test" @click="handleTest" :disabled="form.saving.value">测试</button>
          <button class="btn btn-confirm" @click="handleSave" :disabled="form.saving.value">
            {{ form.saving.value ? '保存中...' : (mode === 'edit' ? '更新' : '创建') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAddConnectionForm } from '@/composables/useAddConnectionForm';
import AddConnectionFormBasicInfo from './AddConnectionFormBasicInfo.vue';
import AddConnectionFormAuth from './AddConnectionFormAuth.vue';
import AddConnectionFormAdvanced from './AddConnectionFormAdvanced.vue';

const props = defineProps<{
  visible: boolean;
  mode: 'create' | 'edit';
  connectionId?: number;
}>();
const emit = defineEmits<{ close: []; saved: [] }>();

const form = useAddConnectionForm(props.mode, props.connectionId);
const activeTab = ref('basic');
const tabs = [
  { key: 'basic', label: '基本信息' },
  { key: 'auth', label: '认证' },
  { key: 'advanced', label: '高级' },
];

const formTitle = props.mode === 'edit' ? '编辑连接' : '新建连接';

onMounted(() => { if (props.mode === 'edit') form.loadConnection(); });

async function handleSave() {
  if (await form.save()) emit('saved');
}
async function handleTest() { await form.testConnection(); }
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.dialog-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; min-width: 480px; max-width: 600px; border: 1px solid var(--border); }
.dialog-title { font-size: 16px; font-weight: 600; margin-bottom: 16px; color: var(--text); }
.form-tabs { display: flex; gap: 4px; margin-bottom: 16px; border-bottom: 1px solid var(--border); padding-bottom: 8px; }
.tab-btn { padding: 6px 12px; border: none; background: none; color: var(--text-sub); cursor: pointer; font-size: 13px; border-radius: 4px; }
.tab-btn:hover { background: var(--bg-surface1); }
.tab-btn.active { color: var(--blue); background: var(--bg-surface1); }
.form-body { min-height: 200px; margin-bottom: 16px; }
.form-actions { display: flex; justify-content: flex-end; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: 13px; }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn-test { background: var(--teal); color: var(--bg-base); }
.btn-confirm { background: var(--blue); color: var(--bg-base); }
</style>
