<template>
  <Teleport to="body">
    <div v-if="visible" class="ctx-menu" :style="{ top: y + 'px', left: x + 'px' }" @click="emit('close')">
      <div v-if="isDir" class="ctx-item" @click="emit('action', 'enter')">进入</div>
      <div v-else class="ctx-item" @click="emit('action', 'open')">打开</div>
      <div v-if="!isDir" class="ctx-item" @click="emit('action', 'edit')">编辑</div>
      <div class="ctx-divider" />
      <div class="ctx-item" @click="emit('action', 'rename')">重命名</div>
      <div class="ctx-item" @click="emit('action', 'copy-path')">复制路径</div>
      <div v-if="!isDir" class="ctx-item" @click="emit('action', 'download')">下载</div>
      <div class="ctx-item" @click="emit('action', 'mkdir')">新建文件夹</div>
      <div class="ctx-divider" />
      <div class="ctx-item" @click="emit('action', 'chmod')">修改权限</div>
      <div class="ctx-item ctx-danger" @click="emit('action', 'delete')">删除</div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
defineProps<{ visible: boolean; x: number; y: number; isDir: boolean }>();
const emit = defineEmits<{ close: []; action: [type: string] }>();
</script>

<style scoped>
.ctx-menu { position: fixed; z-index: 9500; background: var(--bg-surface0); border: 1px solid var(--border); border-radius: 6px; padding: 4px 0; min-width: 160px; box-shadow: 0 4px 12px rgba(0,0,0,0.3); }
.ctx-item { padding: 6px 14px; font-size: 13px; color: var(--text); cursor: pointer; }
.ctx-item:hover { background: var(--bg-surface1); }
.ctx-danger { color: var(--red); }
.ctx-divider { height: 1px; background: var(--border); margin: 4px 0; }
</style>
