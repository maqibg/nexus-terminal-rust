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
      :key="child.id ?? i"
      :size="child.size"
      :min-size="5"
      :class="paneClass(child)"
    >
      <LayoutRenderer :node="child" />
    </Pane>
  </Splitpanes>
</template>

<script lang="ts">
import { defineAsyncComponent, defineComponent, computed } from 'vue';
import { storeToRefs } from 'pinia';
import { Splitpanes, Pane } from 'splitpanes';
import 'splitpanes/dist/splitpanes.css';
import type { LayoutNode, PaneName } from '@/stores/layout';
import { notifyGlobalLayoutResized, useLayoutStore } from '@/stores/layout';
import TerminalView from '@/components/TerminalView.vue';
import CommandInputBar from '@/components/CommandInputBar.vue';
import StatusMonitor from '@/components/StatusMonitor.vue';

// Async panes reduce initial bundle cost for the workspace route.
const WorkspaceConnectionList = defineAsyncComponent(() => import('@/components/WorkspaceConnectionList.vue'));
const DockerManager = defineAsyncComponent(() => import('@/components/DockerManager.vue'));
const SftpBrowser = defineAsyncComponent(() => import('@/components/SftpBrowser.vue'));
const FileEditorContainer = defineAsyncComponent(() => import('@/components/FileEditorContainer.vue'));
const CommandHistoryPanel = defineAsyncComponent(() => import('@/components/CommandHistoryPanel.vue'));
const QuickCommandsPanel = defineAsyncComponent(() => import('@/components/QuickCommandsPanel.vue'));

const componentMap: Record<PaneName, unknown> = {
  connections: WorkspaceConnectionList,
  dockerManager: DockerManager,
  terminal: TerminalView,
  fileManager: SftpBrowser,
  editor: FileEditorContainer,
  commandBar: CommandInputBar,
  statusMonitor: StatusMonitor,
  commandHistory: CommandHistoryPanel,
  quickCommands: QuickCommandsPanel,
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

    const paneClass = (child: LayoutNode): string => {
      if (child.type !== 'split' || !Array.isArray(child.children)) {
        return '';
      }

      const hasStatusMonitor = child.children.some(
        (subChild) => subChild.type === 'pane' && subChild.pane === 'statusMonitor',
      );
      return hasStatusMonitor ? 'layout-pane-status-column' : '';
    };

    return { paneComponent, paneClass, notifyLayoutResized: notifyGlobalLayoutResized, layoutLocked };
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
:deep(.layout-pane-status-column) { min-width: 335px; }

.layout-locked :deep(.splitpanes__splitter) {
  pointer-events: none;
  opacity: 0.45;
}

</style>
