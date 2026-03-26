<template>
  <PromptDialog
    :visible="state.visible"
    :title="state.title"
    :message="state.message"
    :initial-value="state.initialValue"
    :placeholder="state.placeholder"
    :confirm-text="state.confirmText"
    :cancel-text="state.cancelText"
    :input-type="state.inputType"
    :validator="state.validate"
    @confirm="accept"
    @cancel="cancel"
  />
</template>

<script setup lang="ts">
import PromptDialog from '@/components/PromptDialog.vue';
import { usePromptDialog } from '@/composables/usePromptDialog';

const { state, accept, cancel } = usePromptDialog();
</script>
