<script setup lang="ts">
import { computed, ref, watch } from 'vue';

interface GenericTag {
  id: number;
  name: string;
}

type TagValue = number | string;

const props = withDefaults(defineProps<{
  modelValue: TagValue[];
  availableTags?: Array<GenericTag | string>;
  placeholder?: string;
  allowCreate?: boolean;
  allowDelete?: boolean;
}>(), {
  availableTags: () => [],
  placeholder: '添加或选择标签...',
  allowCreate: true,
  allowDelete: true,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: TagValue[]): void;
  (e: 'create-tag', tagName: string): void;
  (e: 'delete-tag', tagId: number): void;
}>();

const inputValue = ref('');
const inputRef = ref<HTMLInputElement | null>(null);
const showSuggestions = ref(false);

const isNumericMode = computed(() => {
  if (props.modelValue.length > 0) {
    return typeof props.modelValue[0] === 'number';
  }
  if (props.availableTags.length > 0) {
    return typeof props.availableTags[0] === 'object';
  }
  return false;
});

const availableObjectTags = computed<GenericTag[]>(() => {
  if (!isNumericMode.value) {
    return [];
  }
  return props.availableTags
    .filter(tag => typeof tag === 'object')
    .map(tag => ({ id: Number((tag as GenericTag).id), name: String((tag as GenericTag).name) }));
});

const availableStringTags = computed<string[]>(() => {
  if (isNumericMode.value) {
    return [];
  }
  return props.availableTags
    .map((tag) => {
      if (typeof tag === 'string') {
        return tag;
      }
      return String(tag.name);
    })
    .filter(tagName => tagName.trim().length > 0);
});

const selectedTagIds = ref<number[]>([]);
const selectedTagNames = ref<string[]>([]);

const arraysEqual = (a: TagValue[], b: TagValue[]): boolean => {
  if (a.length !== b.length) return false;
  return a.every((val, idx) => val === b[idx]);
};

const normalizeToNumbers = (value: TagValue[]): number[] => {
  return value
    .filter(item => typeof item === 'number')
    .map(item => Number(item));
};

const normalizeToStrings = (value: TagValue[]): string[] => {
  return value
    .map(item => String(item))
    .filter(item => item.trim().length > 0);
};

watch(
  () => props.modelValue,
  (value) => {
    if (isNumericMode.value) {
      const normalized = normalizeToNumbers(value);
      if (!arraysEqual(normalized, selectedTagIds.value)) {
        selectedTagIds.value = normalized;
      }
      return;
    }

    const normalized = normalizeToStrings(value);
    if (!arraysEqual(normalized, selectedTagNames.value)) {
      selectedTagNames.value = normalized;
    }
  },
  { immediate: true },
);

watch(selectedTagIds, (value) => {
  if (!isNumericMode.value) {
    return;
  }
  const normalized = normalizeToNumbers(props.modelValue);
  if (!arraysEqual(value, normalized)) {
    emit('update:modelValue', [...value]);
  }
});

watch(selectedTagNames, (value) => {
  if (isNumericMode.value) {
    return;
  }
  const normalized = normalizeToStrings(props.modelValue);
  if (!arraysEqual(value, normalized)) {
    emit('update:modelValue', [...value]);
  }
});

const selectedTags = computed<GenericTag[]>(() => {
  if (!isNumericMode.value) {
    return [];
  }

  const tagMap = new Map<number, GenericTag>();
  availableObjectTags.value.forEach(tag => tagMap.set(tag.id, tag));
  return selectedTagIds.value
    .map(tagId => tagMap.get(tagId))
    .filter((tag): tag is GenericTag => Boolean(tag));
});

const stringSuggestions = computed(() => {
  if (!showSuggestions.value || isNumericMode.value) {
    return [] as string[];
  }
  const q = inputValue.value.trim().toLowerCase();
  return availableStringTags.value
    .filter(tag => !selectedTagNames.value.includes(tag))
    .filter(tag => !q || tag.toLowerCase().includes(q));
});

const objectSuggestions = computed(() => {
  if (!showSuggestions.value || !isNumericMode.value) {
    return [] as GenericTag[];
  }
  const q = inputValue.value.trim().toLowerCase();
  return availableObjectTags.value
    .filter(tag => !selectedTagIds.value.includes(tag.id))
    .filter(tag => !q || tag.name.toLowerCase().includes(q));
});

const handleFocus = () => {
  if (isNumericMode.value) {
    showSuggestions.value = objectSuggestions.value.length > 0;
    return;
  }
  showSuggestions.value = stringSuggestions.value.length > 0;
};

const handleBlur = () => {
  setTimeout(() => {
    showSuggestions.value = false;
  }, 120);
};

const removeTagLocally = (tag: GenericTag | string) => {
  if (isNumericMode.value) {
    const tagId = (tag as GenericTag).id;
    selectedTagIds.value = selectedTagIds.value.filter(id => id !== tagId);
    return;
  }
  const tagName = String(tag);
  selectedTagNames.value = selectedTagNames.value.filter(item => item !== tagName);
};

const selectTag = (tag: GenericTag | string) => {
  if (isNumericMode.value) {
    const tagId = (tag as GenericTag).id;
    if (!selectedTagIds.value.includes(tagId)) {
      selectedTagIds.value = [...selectedTagIds.value, tagId];
    }
  } else {
    const tagName = String(tag);
    if (!selectedTagNames.value.includes(tagName)) {
      selectedTagNames.value = [...selectedTagNames.value, tagName];
    }
  }

  inputValue.value = '';
  showSuggestions.value = false;
  inputRef.value?.focus();
};

const handleDeleteTagGlobally = (tag: GenericTag) => {
  emit('delete-tag', tag.id);
};

const handleKeyDown = () => {
  const value = inputValue.value.trim();
  if (!value) {
    return;
  }

  if (isNumericMode.value) {
    const existingTag = availableObjectTags.value.find(tag => tag.name.toLowerCase() === value.toLowerCase());
    if (existingTag) {
      selectTag(existingTag);
      return;
    }
    if (props.allowCreate) {
      emit('create-tag', value);
    }
    inputValue.value = '';
    showSuggestions.value = false;
    return;
  }

  const existingName = availableStringTags.value.find(tag => tag.toLowerCase() === value.toLowerCase());
  if (existingName) {
    selectTag(existingName);
    return;
  }
  if (props.allowCreate && !selectedTagNames.value.includes(value)) {
    selectedTagNames.value = [...selectedTagNames.value, value];
  }
  inputValue.value = '';
  showSuggestions.value = false;
};
</script>

<template>
  <div class="tag-input-root" @click="inputRef?.focus()">
    <div class="selected-tags">
      <template v-if="isNumericMode">
        <span v-for="tag in selectedTags" :key="tag.id" class="tag-chip">
          {{ tag.name }}
          <button type="button" class="chip-remove" @click.stop="removeTagLocally(tag)">&times;</button>
          <button
            v-if="allowDelete"
            type="button"
            class="chip-delete"
            @click.stop="handleDeleteTagGlobally(tag)"
            title="删除该标签"
          >
            <i class="fas fa-trash-alt"></i>
          </button>
        </span>
      </template>
      <template v-else>
        <span v-for="tag in selectedTagNames" :key="tag" class="tag-chip">
          {{ tag }}
          <button type="button" class="chip-remove" @click.stop="removeTagLocally(tag)">&times;</button>
        </span>
      </template>

      <input
        ref="inputRef"
        v-model="inputValue"
        class="tag-input"
        :placeholder="placeholder"
        @focus="handleFocus"
        @blur="handleBlur"
        @keydown.enter.prevent="handleKeyDown"
      />
    </div>

    <ul v-if="isNumericMode && showSuggestions && objectSuggestions.length > 0" class="suggestion-list">
      <li
        v-for="item in objectSuggestions"
        :key="item.id"
        class="suggestion-item"
        @mousedown.prevent="selectTag(item)"
      >
        {{ item.name }}
      </li>
    </ul>

    <ul v-if="!isNumericMode && showSuggestions && stringSuggestions.length > 0" class="suggestion-list">
      <li
        v-for="item in stringSuggestions"
        :key="item"
        class="suggestion-item"
        @mousedown.prevent="selectTag(item)"
      >
        {{ item }}
      </li>
    </ul>
  </div>
</template>

<style scoped>
.tag-input-root {
  position: relative;
}

.selected-tags {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  min-height: 38px;
  padding: 6px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--bg-surface1);
  color: var(--text);
  font-size: calc(12px + var(--ui-font-size-offset));
}

.chip-remove,
.chip-delete {
  border: none;
  background: transparent;
  color: var(--text-sub);
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.chip-delete:hover {
  color: var(--red);
}

.tag-input {
  flex: 1;
  min-width: 120px;
  border: none;
  background: transparent;
  color: var(--text);
  font-size: calc(13px + var(--ui-font-size-offset));
  outline: none;
}

.suggestion-list {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  z-index: 30;
  margin: 0;
  padding: 4px 0;
  list-style: none;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-surface0);
  max-height: 180px;
  overflow-y: auto;
}

.suggestion-item {
  padding: 6px 10px;
  color: var(--text);
  font-size: calc(13px + var(--ui-font-size-offset));
  cursor: pointer;
}

.suggestion-item:hover {
  background: var(--bg-surface1);
}
</style>
