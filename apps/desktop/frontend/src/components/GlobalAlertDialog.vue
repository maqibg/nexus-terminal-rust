<template>
  <AlertDialog
    :visible="state.visible"
    :title="state.title"
    :message="state.message"
    @close="close"
  />
</template>

<script setup lang="ts">
import AlertDialog from '@/components/AlertDialog.vue';
import { useAlertDialog } from '@/composables/useAlertDialog';

const { state, close } = useAlertDialog();
</script>
