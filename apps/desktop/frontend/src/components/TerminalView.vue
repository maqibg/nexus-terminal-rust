<template>
  <div class="terminal-view-stack">
    <div v-if="!activeSessionId" class="terminal-placeholder">
      <span class="placeholder-text">无活动会话</span>
    </div>
    <KeepAlive :max="16">
      <SessionTerminalView
        v-if="activeSessionId"
        :key="activeSessionId"
        :session-id="activeSessionId"
      />
    </KeepAlive>
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useSessionStore } from '@/stores/session';
import SessionTerminalView from '@/components/SessionTerminalView.vue';

const sessionStore = useSessionStore();
const { activeSessionId } = storeToRefs(sessionStore);
</script>

<style scoped>
.terminal-view-stack {
  width: 100%;
  height: 100%;
}

.terminal-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  background: var(--bg-base);
}

.placeholder-text {
  color: var(--text-dim);
  font-size: calc(14px + var(--ui-font-size-offset));
}
</style>
