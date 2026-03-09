<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';

type SelectValue = string | number | null | undefined;

interface SelectOption {
  label: string;
  value: SelectValue;
  disabled?: boolean;
}

const props = withDefaults(defineProps<{
  modelValue: SelectValue;
  options: SelectOption[];
  disabled?: boolean;
  variant?: 'default' | 'input';
  fitContent?: boolean;
  triggerClass?: string;
  menuClass?: string;
  optionClass?: string;
  id?: string;
  title?: string;
  ariaLabel?: string;
  placeholder?: string;
}>(), {
  disabled: false,
  variant: 'default',
  fitContent: false,
  triggerClass: '',
  menuClass: '',
  optionClass: '',
  id: '',
  title: '',
  ariaLabel: '',
  placeholder: '',
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: SelectValue): void;
  (e: 'change', value: SelectValue): void;
}>();

const rootRef = ref<HTMLElement | null>(null);
const menuOpen = ref(false);
const activeOptionIndex = ref(-1);

const selectedOption = computed(() => props.options.find(option => Object.is(option.value, props.modelValue)));

const selectedLabel = computed(() => {
  if (selectedOption.value) {
    return selectedOption.value.label;
  }
  if (props.placeholder) {
    return props.placeholder;
  }
  if (props.modelValue === null || props.modelValue === undefined) {
    return '';
  }
  return String(props.modelValue);
});

const toggleMenu = () => {
  if (props.disabled || props.options.length === 0) {
    return;
  }
  menuOpen.value = !menuOpen.value;
  if (menuOpen.value) {
    syncActiveOption();
  }
};

const closeMenu = () => {
  menuOpen.value = false;
  activeOptionIndex.value = -1;
};

const selectOption = (option: SelectOption) => {
  if (option.disabled) {
    return;
  }
  emit('update:modelValue', option.value);
  emit('change', option.value);
  closeMenu();
};

const isOptionActive = (option: SelectOption) => Object.is(option.value, props.modelValue);

const enabledOptionIndexes = computed(() =>
  props.options
    .map((option, index) => ({ option, index }))
    .filter(({ option }) => !option.disabled)
    .map(({ index }) => index),
);

const syncActiveOption = () => {
  const selectedIndex = props.options.findIndex((option) => isOptionActive(option) && !option.disabled);
  if (selectedIndex >= 0) {
    activeOptionIndex.value = selectedIndex;
    return;
  }
  activeOptionIndex.value = enabledOptionIndexes.value[0] ?? -1;
};

const moveActiveOption = (direction: 1 | -1) => {
  const indexes = enabledOptionIndexes.value;
  if (indexes.length === 0) {
    activeOptionIndex.value = -1;
    return;
  }

  if (!menuOpen.value) {
    menuOpen.value = true;
    syncActiveOption();
    return;
  }

  const currentPosition = indexes.findIndex((index) => index === activeOptionIndex.value);
  const nextPosition = currentPosition < 0
    ? 0
    : (currentPosition + direction + indexes.length) % indexes.length;
  activeOptionIndex.value = indexes[nextPosition] ?? indexes[0];
};

const selectActiveOption = () => {
  if (!menuOpen.value) {
    toggleMenu();
    return;
  }

  const option = props.options[activeOptionIndex.value];
  if (option && !option.disabled) {
    selectOption(option);
  }
};

const handleTriggerKeydown = (event: KeyboardEvent) => {
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      moveActiveOption(1);
      return;
    case 'ArrowUp':
      event.preventDefault();
      moveActiveOption(-1);
      return;
    case 'Home':
      if (!menuOpen.value || enabledOptionIndexes.value.length === 0) {
        return;
      }
      event.preventDefault();
      activeOptionIndex.value = enabledOptionIndexes.value[0] ?? -1;
      return;
    case 'End':
      if (!menuOpen.value || enabledOptionIndexes.value.length === 0) {
        return;
      }
      event.preventDefault();
      activeOptionIndex.value = enabledOptionIndexes.value[enabledOptionIndexes.value.length - 1] ?? -1;
      return;
    case 'Enter':
    case ' ':
      event.preventDefault();
      selectActiveOption();
      return;
    default:
      return;
  }
};

const handleOutsidePointerDown = (event: MouseEvent) => {
  const target = event.target as Node | null;
  if (!target || !rootRef.value) {
    return;
  }
  if (!rootRef.value.contains(target)) {
    closeMenu();
  }
};

const handleEscape = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    closeMenu();
  }
};

watch(() => props.disabled, (value) => {
  if (value) {
    closeMenu();
  }
});

watch(() => props.options, () => {
  if (props.options.length === 0) {
    closeMenu();
    return;
  }
  syncActiveOption();
});

watch(() => props.modelValue, () => {
  syncActiveOption();
});

onMounted(() => {
  document.addEventListener('mousedown', handleOutsidePointerDown);
  window.addEventListener('keydown', handleEscape);
  syncActiveOption();
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleOutsidePointerDown);
  window.removeEventListener('keydown', handleEscape);
});
</script>

<template>
  <div
    ref="rootRef"
    class="app-select-root"
    :class="{
      'is-open': menuOpen,
      'is-disabled': disabled,
      'is-input-variant': variant === 'input',
      'is-fit-content': fitContent,
    }"
  >
    <button
      :id="id"
      type="button"
      class="app-select-trigger"
      :class="triggerClass"
      :title="title"
      :disabled="disabled"
      :aria-label="ariaLabel"
      :aria-expanded="menuOpen ? 'true' : 'false'"
      aria-haspopup="listbox"
      @click="toggleMenu"
      @keydown="handleTriggerKeydown"
    >
      <span class="app-select-label">{{ selectedLabel }}</span>
      <i class="fas fa-chevron-down app-select-icon"></i>
    </button>

    <div v-if="menuOpen" class="app-select-menu" :class="menuClass" role="listbox">
      <button
        v-for="(option, index) in options"
        :key="`${index}-${typeof option.value}-${String(option.value)}`"
        type="button"
        class="app-select-option"
        :class="[optionClass, { 'is-active': isOptionActive(option), 'is-highlighted': activeOptionIndex === index }]"
        :disabled="option.disabled"
        role="option"
        :aria-selected="isOptionActive(option) ? 'true' : 'false'"
        @click="selectOption(option)"
        @mouseenter="activeOptionIndex = index"
      >
        {{ option.label }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.app-select-root {
  position: relative;
  width: 100%;
  min-width: 0;
}

.app-select-root.is-fit-content {
  width: auto;
}

.app-select-trigger {
  width: 100%;
  min-height: 34px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  padding: 6px 10px;
  font-size: 13px;
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  text-align: left;
  gap: 10px;
  cursor: pointer;
}

.app-select-root.is-fit-content .app-select-trigger {
  width: auto;
}

.app-select-root.is-open .app-select-trigger {
  border-color: var(--blue);
}

.app-select-trigger:focus-visible {
  outline: none;
  box-shadow: 0 0 0 1px var(--blue);
}

.app-select-root.is-disabled .app-select-trigger {
  opacity: 0.65;
  cursor: not-allowed;
}

.app-select-root.is-input-variant .app-select-trigger {
  min-height: 0;
  border-radius: 4px;
  background: var(--bg-mantle);
  padding: 8px;
}

.app-select-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-select-icon {
  color: var(--text-sub);
  font-size: 11px;
  transition: transform 0.15s ease;
}

.app-select-root.is-open .app-select-icon {
  transform: rotate(180deg);
}

.app-select-menu {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  max-height: 260px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 0 0 10px 10px;
  background: var(--bg-base);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.35);
  z-index: 2000;
}

.app-select-option {
  width: 100%;
  border: none;
  background: transparent;
  color: var(--text);
  text-align: left;
  padding: 5px 10px;
  min-height: 30px;
  font-size: 13px;
  line-height: 1.2;
  cursor: pointer;
}

.app-select-option:hover {
  background: color-mix(in srgb, var(--blue) 20%, var(--bg-base));
}

.app-select-option.is-active {
  background: var(--blue);
  color: var(--button-text-color);
}

.app-select-option.is-highlighted:not(.is-active) {
  background: color-mix(in srgb, var(--blue) 20%, var(--bg-base));
}

.app-select-option:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
