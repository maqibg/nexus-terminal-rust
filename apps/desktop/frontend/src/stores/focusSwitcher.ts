import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useFocusSwitcherStore = defineStore('focusSwitcher', () => {
  const visible = ref(false);
  const targets = ref<string[]>([]);
  const activeTarget = ref<string | null>(null);

  function open() { visible.value = true; }
  function close() { visible.value = false; }
  function toggle() { visible.value = !visible.value; }

  function setTargets(list: string[]) { targets.value = list; }
  function setActive(target: string) { activeTarget.value = target; }

  return { visible, targets, activeTarget, open, close, toggle, setTargets, setActive };
});
