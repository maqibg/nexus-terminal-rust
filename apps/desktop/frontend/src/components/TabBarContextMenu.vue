<template>
  <Teleport to="body">
    <div v-if="visible" class="ctx-backdrop" @click="$emit('close')"></div>
    <div v-if="visible" class="ctx-menu" :style="{ left: x + 'px', top: y + 'px' }">
      <div class="ctx-item" @click="$emit('action', 'close')">关闭</div>
      <div class="ctx-item" @click="$emit('action', 'close-others')">关闭其他</div>
      <div class="ctx-item" @click="$emit('action', 'close-right')">关闭右侧</div>
      <div class="ctx-item" @click="$emit('action', 'close-left')">关闭左侧</div>
      <div class="ctx-item" @click="$emit('action', 'duplicate')">新建会话</div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
defineProps<{ visible: boolean; x: number; y: number; sessionId: string }>();
defineEmits<{ close: []; action: [type: string] }>();
</script>

<style scoped>
.ctx-backdrop { position: fixed; inset: 0; z-index: 99; }
.ctx-menu { position: fixed; z-index: 100; background: var(--bg-surface0); border-radius: 6px; padding: 4px; box-shadow: 0 4px 12px rgba(0,0,0,0.4); min-width: 120px; border: 1px solid var(--border); }
.ctx-item { padding: 5px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; color: var(--text); }
.ctx-item:hover { background: var(--bg-surface1); }
</style>
