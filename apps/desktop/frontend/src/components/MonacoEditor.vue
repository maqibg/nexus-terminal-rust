<template>
  <div ref="editorContainer" class="monaco-container"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import * as monaco from 'monaco-editor';

// Configure Monaco workers
self.MonacoEnvironment = {
  getWorker(_: string, label: string) {
    const getWorkerModule = (url: string) =>
      new Worker(new URL(url, import.meta.url), { type: 'module' });
    if (label === 'json') return getWorkerModule('monaco-editor/esm/vs/language/json/json.worker?worker');
    if (label === 'css' || label === 'scss' || label === 'less') return getWorkerModule('monaco-editor/esm/vs/language/css/css.worker?worker');
    if (label === 'html' || label === 'handlebars' || label === 'razor') return getWorkerModule('monaco-editor/esm/vs/language/html/html.worker?worker');
    if (label === 'typescript' || label === 'javascript') return getWorkerModule('monaco-editor/esm/vs/language/typescript/ts.worker?worker');
    return getWorkerModule('monaco-editor/esm/vs/editor/editor.worker?worker');
  },
};

// Register Catppuccin Mocha theme
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
}>(), { language: 'plaintext', readOnly: false, theme: 'catppuccin-mocha' });

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const editorContainer = ref<HTMLElement>();
let editor: monaco.editor.IStandaloneCodeEditor | null = null;

onMounted(() => {
  if (!editorContainer.value) return;
  editor = monaco.editor.create(editorContainer.value, {
    value: props.modelValue,
    language: props.language,
    theme: props.theme,
    readOnly: props.readOnly,
    minimap: { enabled: false },
    fontSize: 13,
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    automaticLayout: true,
  });
  editor.onDidChangeModelContent(() => {
    emit('update:modelValue', editor!.getValue());
  });
});

watch(() => props.modelValue, (val) => {
  if (editor && val !== editor.getValue()) editor.setValue(val);
});

watch(() => props.language, (lang) => {
  if (editor) {
    const model = editor.getModel();
    if (model) monaco.editor.setModelLanguage(model, lang);
  }
});

watch(() => props.readOnly, (ro) => {
  editor?.updateOptions({ readOnly: ro });
});

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

onBeforeUnmount(() => { editor?.dispose(); });
</script>

<style scoped>
.monaco-container { width: 100%; height: 100%; }
</style>
