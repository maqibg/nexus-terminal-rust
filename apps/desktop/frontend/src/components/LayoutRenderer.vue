<template>
  <template v-if="node.type === 'pane'">
    <component :is="paneComponent" />
  </template>
  <Splitpanes
    v-else
    :horizontal="node.direction === 'vertical'"
    :class="['layout-splitpanes', { 'layout-locked': layoutLocked }]"
    @resize="notifyLayoutResized"
    @resized="notifyLayoutResized"
  >
    <Pane
      v-for="(child, i) in node.children"
      :key="i"
      :size="child.size"
      :min-size="5"
    >
      <LayoutRenderer :node="child" />
    </Pane>
  </Splitpanes>
</template>

<script lang="ts">
import { defineComponent, computed, onBeforeUnmount, h } from 'vue';
import { storeToRefs } from 'pinia';
import { Splitpanes, Pane } from 'splitpanes';
import 'splitpanes/dist/splitpanes.css';
import type { LayoutNode, PaneName } from '@/stores/layout';
import { useLayoutStore } from '@/stores/layout';
import TerminalView from '@/components/TerminalView.vue';
import SftpBrowser from '@/components/SftpBrowser.vue';
import FileEditorContainer from '@/components/FileEditorContainer.vue';
import CommandInputBar from '@/components/CommandInputBar.vue';
import StatusMonitor from '@/components/StatusMonitor.vue';
import CommandHistoryPanel from '@/components/CommandHistoryPanel.vue';
import QuickCommandsPanel from '@/components/QuickCommandsPanel.vue';
import WorkspaceConnectionList from '@/components/WorkspaceConnectionList.vue';

const DockerManagerPlaceholder = defineComponent({
  name: 'DockerManagerPlaceholder',
  setup() {
    return () =>
      h('div', { class: 'docker-manager-placeholder' }, [
        h('i', { class: 'fab fa-docker docker-manager-icon' }),
        h('div', { class: 'docker-manager-title' }, '远程主机 Docker 不可用'),
        h('div', { class: 'docker-manager-desc' }, '请确保远程主机上已安装并运行 Docker。'),
      ]);
  },
});

const componentMap: Record<PaneName, unknown> = {
  connections: WorkspaceConnectionList,
  terminal: TerminalView,
  fileManager: SftpBrowser,
  editor: FileEditorContainer,
  commandBar: CommandInputBar,
  statusMonitor: StatusMonitor,
  commandHistory: CommandHistoryPanel,
  quickCommands: QuickCommandsPanel,
  dockerManager: DockerManagerPlaceholder,
};

export default defineComponent({
  name: 'LayoutRenderer',
  components: { Splitpanes, Pane },
  props: {
    node: { type: Object as () => LayoutNode, required: true },
  },
  setup(props) {
    const layoutStore = useLayoutStore();
    const { layoutLocked } = storeToRefs(layoutStore);

    const paneComponent = computed(() =>
      props.node.pane ? componentMap[props.node.pane] ?? 'div' : 'div'
    );

    let resizeDispatchRaf = 0;

    const notifyLayoutResized = () => {
      if (typeof window === 'undefined') {
        return;
      }

      if (resizeDispatchRaf) {
        window.cancelAnimationFrame(resizeDispatchRaf);
      }

      resizeDispatchRaf = window.requestAnimationFrame(() => {
        window.dispatchEvent(new Event('resize'));
        window.dispatchEvent(new CustomEvent('nexus:layout-resized'));
        resizeDispatchRaf = 0;
      });
    };

    onBeforeUnmount(() => {
      if (resizeDispatchRaf) {
        window.cancelAnimationFrame(resizeDispatchRaf);
        resizeDispatchRaf = 0;
      }
    });

    return { paneComponent, notifyLayoutResized, layoutLocked };
  },
});
</script>

<style scoped>
.layout-splitpanes { height: 100%; }
:deep(.splitpanes__pane) { min-width: 0; min-height: 0; overflow: hidden; }

:deep(.splitpanes__splitter) {
  background: var(--border);
  position: relative;
}
:deep(.splitpanes--horizontal > .splitpanes__splitter) { height: 4px; margin: -2px 0; }
:deep(.splitpanes--vertical > .splitpanes__splitter) { width: 4px; margin: 0 -2px; }
:deep(.splitpanes__splitter:hover) { background: var(--blue); }

.layout-locked :deep(.splitpanes__splitter) {
  pointer-events: none;
  opacity: 0.45;
}

.docker-manager-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  text-align: center;
  padding: 20px;
  background: var(--bg-base);
}

.docker-manager-icon {
  font-size: 38px;
  color: var(--text-dim);
}

.docker-manager-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
}

.docker-manager-desc {
  max-width: 260px;
  line-height: 1.6;
  font-size: 12px;
  color: var(--text-sub);
}
</style>

