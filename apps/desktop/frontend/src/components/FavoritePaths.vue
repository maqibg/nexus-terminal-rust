<template>
  <div class="fav-panel">
    <div class="fav-header">
      <span>收藏路径</span>
      <button class="btn-icon" @click="showAdd = true" title="添加">+</button>
    </div>
    <div class="fav-list">
      <div v-for="f in items" :key="f.id" class="fav-item" @click="$emit('navigate', f.path)">
        <span class="fav-name">{{ f.name }}</span>
        <span class="fav-path">{{ f.path }}</span>
        <button class="fav-edit" @click.stop="openEdit(f)" title="编辑">✎</button>
        <button class="fav-del" @click.stop="handleDelete(f.id)" title="删除">×</button>
      </div>
      <div v-if="!items.length" class="status">暂无收藏</div>
    </div>

    <div v-if="showAdd || showEdit" class="mini-backdrop" @click.self="showAdd = false; showEdit = false">
      <div class="mini-dialog">
        <label>名称 <input v-model="form.name" /></label>
        <label>路径 <input v-model="form.path" /></label>
        <div class="mini-actions">
          <button class="btn-cancel" @click="showAdd = false; showEdit = false">取消</button>
          <button class="btn-save" @click="showEdit ? handleUpdate() : handleAdd()">{{ showEdit ? '保存' : '添加' }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted } from 'vue';
import { favoritePathApi } from '@/lib/api';
import type { FavoritePath } from '@/lib/api';

const props = defineProps<{ connectionId?: number }>();
defineEmits<{ navigate: [path: string] }>();

const items = ref<FavoritePath[]>([]);
const showAdd = ref(false);
const showEdit = ref(false);
const editingId = ref(0);
const form = reactive({ name: '', path: '' });

async function load() {
  try { items.value = await favoritePathApi.list(props.connectionId); } catch { /* ignore */ }
}

async function handleAdd() {
  if (!form.name || !form.path) return;
  try {
    await favoritePathApi.create(form.name, form.path, props.connectionId);
    showAdd.value = false;
    form.name = ''; form.path = '';
    load();
  } catch { /* ignore */ }
}

async function handleDelete(id: number) {
  try { await favoritePathApi.delete(id); load(); } catch { /* ignore */ }
}

function openEdit(f: FavoritePath) {
  editingId.value = f.id;
  form.name = f.name;
  form.path = f.path;
  showEdit.value = true;
}

async function handleUpdate() {
  if (!form.name || !form.path) return;
  try {
    await favoritePathApi.update(editingId.value, form.name, form.path, props.connectionId);
    showEdit.value = false;
    form.name = ''; form.path = '';
    load();
  } catch { /* ignore */ }
}

watch(() => props.connectionId, load);
onMounted(load);
</script>

<style scoped>
.fav-panel { display: flex; flex-direction: column; }
.fav-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 4px 8px; font-size: 0.75rem; color: #a6adc8; border-bottom: 1px solid #313244;
}
.btn-icon {
  width: 18px; height: 18px; border-radius: 3px; border: 1px solid #45475a;
  background: transparent; color: #cdd6f4; cursor: pointer; font-size: 0.8rem;
  display: flex; align-items: center; justify-content: center;
}
.btn-icon:hover { background: #313244; }
.fav-list { overflow-y: auto; max-height: 120px; }
.fav-item {
  display: flex; align-items: center; gap: 6px; padding: 3px 8px; cursor: pointer; font-size: 0.75rem;
}
.fav-item:hover { background: #313244; }
.fav-name { color: #cdd6f4; }
.fav-path { color: #6c7086; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.fav-del {
  background: none; border: none; color: #f38ba8; cursor: pointer; font-size: 0.8rem;
  opacity: 0; padding: 0;
}
.fav-edit {
  background: none; border: none; color: #89b4fa; cursor: pointer; font-size: 0.7rem;
  opacity: 0; padding: 0;
}
.fav-item:hover .fav-del, .fav-item:hover .fav-edit { opacity: 1; }
.status { padding: 6px; text-align: center; color: #6c7086; font-size: 0.7rem; }

.mini-backdrop {
  position: fixed; inset: 0; background: rgba(0,0,0,0.4);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.mini-dialog {
  background: #313244; border-radius: 8px; padding: 1rem; min-width: 260px;
  display: flex; flex-direction: column; gap: 6px;
}
.mini-dialog label { display: flex; flex-direction: column; gap: 3px; font-size: 0.8rem; color: #a6adc8; }
.mini-dialog input {
  padding: 5px 8px; border-radius: 4px; border: 1px solid #45475a;
  background: #1e1e2e; color: #cdd6f4; font-size: 0.8rem; outline: none;
}
.mini-dialog input:focus { border-color: #89b4fa; }
.mini-actions { display: flex; justify-content: flex-end; gap: 6px; margin-top: 4px; }
.btn-cancel {
  padding: 4px 10px; border-radius: 4px; border: 1px solid #45475a;
  background: transparent; color: #a6adc8; cursor: pointer; font-size: 0.8rem;
}
.btn-cancel:hover { background: #45475a; }
.btn-save {
  padding: 4px 10px; border-radius: 4px; border: none;
  background: #89b4fa; color: #1e1e2e; cursor: pointer; font-weight: 600; font-size: 0.8rem;
}
.btn-save:hover { background: #74c7ec; }
</style>
