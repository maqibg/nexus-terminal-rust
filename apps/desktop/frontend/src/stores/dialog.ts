import { defineStore } from 'pinia';
import { reactive } from 'vue';

export const useDialogStore = defineStore('dialog', () => {
  const dialogs = reactive<Record<string, { visible: boolean; props?: Record<string, unknown> }>>({});

  function openDialog(name: string, props?: Record<string, unknown>) {
    dialogs[name] = { visible: true, props };
  }

  function closeDialog(name: string) {
    if (dialogs[name]) dialogs[name].visible = false;
  }

  function isOpen(name: string) {
    return dialogs[name]?.visible ?? false;
  }

  function getProps(name: string) {
    return dialogs[name]?.props;
  }

  return { dialogs, openDialog, closeDialog, isOpen, getProps };
});
