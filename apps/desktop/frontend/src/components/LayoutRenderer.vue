<template>
  <template v-if="node.type === 'pane'">
    <component :is="paneComponent" />
  </template>
  <Splitpanes
    v-else
    :horizontal="node.direction === 'vertical'"
    class="layout-splitpanes"
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
import { defineComponent, computed, onBeforeUnmount } from 'vue';
import { Splitpanes, Pane } from 'splitpanes';
import 'splitpanes/dist/splitpanes.css';
import type { LayoutNode, PaneName } from '@/stores/layout';
import TerminalView from '@/components/TerminalView.vue';
import SftpBrowser from '@/components/SftpBrowser.vue';
import FileEditorContainer from '@/components/FileEditorContainer.vue';
import CommandInputBar from '@/components/CommandInputBar.vue';
import StatusMonitor from '@/components/StatusMonitor.vue';
import CommandHistoryPanel from '@/components/CommandHistoryPanel.vue';
import QuickCommandsPanel from '@/components/QuickCommandsPanel.vue';

const componentMap: Record<PaneName, unknown> = {
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

    return { paneComponent, notifyLayoutResized };
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
</style>

