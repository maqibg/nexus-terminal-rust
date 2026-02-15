<template>
  <div ref="editorContainer" class="monaco-container"></div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import * as monaco from 'monaco-editor';

self.MonacoEnvironment = {
  getWorker(_: string, label: string) {
    const getWorkerModule = (url: string) =>
      new Worker(new URL(url, import.meta.url), { type: 'module' });

    if (label === 'json') {
      return getWorkerModule('monaco-editor/esm/vs/language/json/json.worker?worker');
    }
    if (label === 'css' || label === 'scss' || label === 'less') {
      return getWorkerModule('monaco-editor/esm/vs/language/css/css.worker?worker');
    }
    if (label === 'html' || label === 'handlebars' || label === 'razor') {
      return getWorkerModule('monaco-editor/esm/vs/language/html/html.worker?worker');
    }
    if (label === 'typescript' || label === 'javascript') {
      return getWorkerModule('monaco-editor/esm/vs/language/typescript/ts.worker?worker');
    }

    return getWorkerModule('monaco-editor/esm/vs/editor/editor.worker?worker');
  },
};

monaco.editor.defineTheme('catppuccin-mocha', {
  base: 'vs-dark',
  inherit: true,
  rules: [],
  colors: {
    'editor.background': '#1e1e2e',
    'editor.foreground': '#cdd6f4',
    'editor.selectionBackground': '#45475a',
    'editor.lineHighlightBackground': '#313244',
    'editorCursor.foreground': '#f5e0dc',
    'editorWhitespace.foreground': '#6c7086',
  },
});

const props = withDefaults(defineProps<{
  modelValue: string;
  language?: string;
  readOnly?: boolean;
  theme?: string;
  fontFamily?: string;
  fontSize?: number;
  initialScrollTop?: number;
  initialScrollLeft?: number;
}>(), {
  language: 'plaintext',
  readOnly: false,
  theme: 'vs-dark',
  fontFamily: 'Consolas, "Courier New", monospace',
  fontSize: 14,
  initialScrollTop: 0,
  initialScrollLeft: 0,
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'request-save': [];
  'update:scrollPosition': [position: { scrollTop: number; scrollLeft: number }];
  'update:fontSize': [fontSize: number];
}>();

const editorContainer = ref<HTMLElement>();
let editor: monaco.editor.IStandaloneCodeEditor | null = null;
let wheelHandler: ((event: WheelEvent) => void) | null = null;

function clampFontSize(size: number): number {
  return Math.min(40, Math.max(8, size));
}

onMounted(() => {
  if (!editorContainer.value) {
    return;
  }

  editor = monaco.editor.create(editorContainer.value, {
    value: props.modelValue,
    language: props.language,
    theme: props.theme,
    readOnly: props.readOnly,
    minimap: { enabled: true },
    fontFamily: props.fontFamily,
    fontSize: props.fontSize,
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    automaticLayout: true,
  });

  editor.onDidChangeModelContent(() => {
    const current = editor?.getValue() ?? '';
    if (current !== props.modelValue) {
      emit('update:modelValue', current);
    }
  });

  editor.onDidScrollChange(() => {
    if (!editor) {
      return;
    }

    emit('update:scrollPosition', {
      scrollTop: editor.getScrollTop(),
      scrollLeft: editor.getScrollLeft(),
    });
  });

  editor.addAction({
    id: 'nexus-file-save',
    label: 'Save File',
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS],
    run: () => {
      emit('request-save');
    },
  });

  const domNode = editor.getDomNode();
  if (domNode) {
    wheelHandler = (event: WheelEvent) => {
      if (!event.ctrlKey || !editor) {
        return;
      }

      event.preventDefault();
      const current = editor.getOption(monaco.editor.EditorOption.fontSize);
      const base = typeof current === 'number' ? current : props.fontSize;
      const next = clampFontSize(base + (event.deltaY < 0 ? 1 : -1));
      if (next !== base) {
        editor.updateOptions({ fontSize: next });
        emit('update:fontSize', next);
      }
    };

    domNode.addEventListener('wheel', wheelHandler, { passive: false });
  }

  if ((props.initialScrollTop ?? 0) > 0 || (props.initialScrollLeft ?? 0) > 0) {
    editor.setScrollPosition({
      scrollTop: props.initialScrollTop ?? 0,
      scrollLeft: props.initialScrollLeft ?? 0,
    });
  }
});

watch(
  () => props.modelValue,
  (value) => {
    if (!editor || value === editor.getValue()) {
      return;
    }

    editor.setValue(value);
  },
);

watch(
  () => props.language,
  (language) => {
    if (!editor) {
      return;
    }

    const model = editor.getModel();
    if (model && language) {
      monaco.editor.setModelLanguage(model, language);
    }
  },
);

watch(
  () => props.theme,
  (theme) => {
    if (theme) {
      monaco.editor.setTheme(theme);
    }
  },
);

watch(
  () => props.readOnly,
  (readOnly) => {
    editor?.updateOptions({ readOnly });
  },
);

watch(
  () => props.fontFamily,
  (fontFamily) => {
    if (fontFamily) {
      editor?.updateOptions({ fontFamily });
    }
  },
);

watch(
  () => props.fontSize,
  (fontSize) => {
    editor?.updateOptions({ fontSize: clampFontSize(fontSize) });
  },
);

function focusEditor(): boolean {
  if (!editor) {
    return false;
  }

  editor.focus();
  return true;
}

defineExpose({
  focusEditor,
});

onBeforeUnmount(() => {
  const domNode = editor?.getDomNode();
  if (domNode && wheelHandler) {
    domNode.removeEventListener('wheel', wheelHandler);
  }
  wheelHandler = null;

  editor?.dispose();
  editor = null;
});
</script>

<style scoped>
.monaco-container {
  width: 100%;
  height: 100%;
  min-height: 300px;
  text-align: left;
}
</style>