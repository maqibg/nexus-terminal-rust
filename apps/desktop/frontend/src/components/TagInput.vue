<template>
  <div class="tag-input">
    <div class="chips">
      <span v-for="tag in modelValue" :key="tag" class="chip">
        {{ tag }}
        <button class="chip-del" @click="remove(tag)">&times;</button>
      </span>
      <input
        ref="inputEl"
        v-model="input"
        class="tag-field"
        placeholder="输入标签..."
        @keydown.enter.prevent="add"
        @focus="showDropdown = true"
        @blur="hideDropdown"
      />
    </div>
    <div v-if="showDropdown && filtered.length" class="dropdown">
      <div v-for="t in filtered" :key="t" class="dropdown-item" @mousedown.prevent="select(t)">{{ t }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps<{ modelValue: string[]; availableTags: string[] }>();
const emit = defineEmits<{ 'update:modelValue': [tags: string[]] }>();

const input = ref('');
const inputEl = ref<HTMLInputElement>();
const showDropdown = ref(false);

const filtered = computed(() =>
  props.availableTags.filter(t => !props.modelValue.includes(t) && t.toLowerCase().includes(input.value.toLowerCase()))
);

function add() {
  const v = input.value.trim();
  if (v && !props.modelValue.includes(v)) emit('update:modelValue', [...props.modelValue, v]);
  input.value = '';
}

function select(tag: string) {
  if (!props.modelValue.includes(tag)) emit('update:modelValue', [...props.modelValue, tag]);
  input.value = '';
}

function remove(tag: string) { emit('update:modelValue', props.modelValue.filter(t => t !== tag)); }
function hideDropdown() { setTimeout(() => showDropdown.value = false, 150); }
</script>

<style scoped>
.tag-input { position: relative; }
.chips { display: flex; flex-wrap: wrap; gap: 4px; padding: 4px; background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; min-height: 32px; align-items: center; }
.chip { display: flex; align-items: center; gap: 2px; padding: 1px 6px; border-radius: 3px; background: rgba(137,180,250,0.15); color: var(--blue); font-size: 12px; }
.chip-del { background: none; border: none; color: var(--blue); cursor: pointer; font-size: 14px; padding: 0; line-height: 1; }
.tag-field { flex: 1; min-width: 60px; background: transparent; border: none; outline: none; color: var(--text); font-size: 13px; }
.dropdown { position: absolute; top: 100%; left: 0; right: 0; background: var(--bg-surface0); border: 1px solid var(--border); border-radius: 4px; max-height: 120px; overflow-y: auto; z-index: 10; margin-top: 2px; }
.dropdown-item { padding: 4px 8px; font-size: 12px; cursor: pointer; color: var(--text); }
.dropdown-item:hover { background: var(--bg-surface1); }
</style>
