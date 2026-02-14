<template>
  <ConfirmDialog
    :visible="state.visible"
    :title="state.title"
    :message="state.message"
    @confirm="accept"
    @cancel="cancel"
  />
</template>

<script setup lang="ts">
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import { useConfirmDialog } from '@/composables/useConfirmDialog';

const { state, accept, cancel } = useConfirmDialog();
</script>
