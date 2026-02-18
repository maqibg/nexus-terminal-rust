<template>
  <Teleport to="body">
    <div v-if="isVisible" class="overlay-backdrop" @click.self="closeOverlay">
      <div class="overlay-card">
        <div class="overlay-header">
          <span class="overlay-title">文件编辑器</span>
          <button class="btn-close" @click="closeOverlay">&times;</button>
        </div>
        <FileEditorContainer />
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import FileEditorContainer from './FileEditorContainer.vue';
import { useFileEditorStore } from '@/stores/fileEditor';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();
const fileEditorStore = useFileEditorStore();
const { popupTrigger, popupFileInfo } = storeToRefs(fileEditorStore);

const popupEditorEnabled = computed(() => settingsStore.getBoolean('showPopupFileEditor', false));
const isVisible = ref(false);

function closeOverlay() {
  isVisible.value = false;
}

watch(
  popupTrigger,
  () => {
    if (!popupEditorEnabled.value || !popupFileInfo.value) {
      isVisible.value = false;
      return;
    }

    isVisible.value = true;
  },
  { immediate: true },
);

watch(popupEditorEnabled, (enabled) => {
  if (!enabled) {
    isVisible.value = false;
  }
});
</script>

<style scoped>
.overlay-backdrop { position: fixed; inset: 0; z-index: 8000; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; }
.overlay-card { width: 80vw; height: 70vh; background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; display: flex; flex-direction: column; overflow: hidden; }
.overlay-header { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: var(--bg-mantle); border-bottom: 1px solid var(--border); }
.overlay-title { font-size: 13px; font-weight: 600; color: var(--text); }
.btn-close { background: none; border: none; color: var(--text-dim); font-size: 18px; cursor: pointer; }
.btn-close:hover { color: var(--red); }
</style>