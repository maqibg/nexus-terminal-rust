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
        :placeholder="placeholderText"
        @keydown.enter.prevent="add"
        @focus="showDropdown = true"
        @blur="hideDropdown"
      />
    </div>
    <div v-if="showDropdown && filtered.length" class="dropdown">
      <div
        v-for="tagOption in filtered"
        :key="tagOption"
        class="dropdown-item"
        @mousedown.prevent="select(tagOption)"
      >
        {{ tagOption }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';

const props = defineProps<{ modelValue: string[]; availableTags: string[]; placeholder?: string }>();
const emit = defineEmits<{ 'update:modelValue': [tags: string[]] }>();

const input = ref('');
const inputEl = ref<HTMLInputElement>();
const showDropdown = ref(false);

const placeholderText = computed(() => props.placeholder || '输入搜索或创建标签...');

const filtered = computed(() =>
  props.availableTags.filter(
    (tag) => !props.modelValue.includes(tag) && tag.toLowerCase().includes(input.value.toLowerCase()),
  ),
);

function add() {
  const value = input.value.trim();
  if (value && !props.modelValue.includes(value)) {
    emit('update:modelValue', [...props.modelValue, value]);
  }
  input.value = '';
}

function select(tag: string) {
  if (!props.modelValue.includes(tag)) {
    emit('update:modelValue', [...props.modelValue, tag]);
  }
  input.value = '';
}

function remove(tag: string) {
  emit('update:modelValue', props.modelValue.filter((item) => item !== tag));
}

function hideDropdown() {
  setTimeout(() => {
    showDropdown.value = false;
  }, 150);
}
</script>

<style scoped>
.tag-input {
  position: relative;
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  min-height: 38px;
  padding: 5px 6px;
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  background: var(--bg-base, #1e1e2e);
}

.chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  max-width: 100%;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(203, 166, 247, 0.2);
  color: var(--mauve, #cba6f7);
  font-size: 12px;
  line-height: 1.3;
}

.chip-del {
  border: none;
  background: transparent;
  color: inherit;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  padding: 0;
}

.tag-field {
  flex: 1;
  min-width: 120px;
  border: none;
  background: transparent;
  color: var(--text, #cdd6f4);
  font-size: 13px;
  line-height: 1.5;
  outline: none;
}

.tag-field::placeholder {
  color: var(--text-dim, #6c7086);
}

.dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  max-height: 180px;
  overflow-y: auto;
  background: var(--bg-surface0, #313244);
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  box-shadow: 0 12px 28px rgba(0, 0, 0, 0.42);
  z-index: 3400;
}

.dropdown-item {
  padding: 7px 10px;
  font-size: 12px;
  color: var(--text, #cdd6f4);
  cursor: pointer;
}

.dropdown-item:hover {
  background: rgba(137, 180, 250, 0.12);
}
</style>
