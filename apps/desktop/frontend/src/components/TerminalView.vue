<template>
  <div class="terminal-view-stack">
    <div v-if="!activeSession" class="terminal-placeholder">
      <span class="placeholder-text">无活动会话</span>
    </div>
    <KeepAlive :max="16">
      <LocalTerminalSessionView
        v-if="activeSession && activeSession.protocol === 'LOCAL'"
        :key="activeSession.id"
        :session-id="activeSession.id"
      />
      <TelnetTerminalView
        v-else-if="activeSession && activeSession.protocol === 'TELNET'"
        :key="activeSession.id"
        :session-id="activeSession.id"
      />
      <SessionTerminalView
        v-else-if="activeSession"
        :key="activeSession.id"
        :session-id="activeSession.id"
      />
    </KeepAlive>
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useSessionStore } from '@/stores/session';
import SessionTerminalView from '@/components/SessionTerminalView.vue';
import TelnetTerminalView from '@/components/TelnetTerminalView.vue';
import LocalTerminalSessionView from '@/components/LocalTerminalSessionView.vue';

const sessionStore = useSessionStore();
const { activeSession } = storeToRefs(sessionStore);
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
