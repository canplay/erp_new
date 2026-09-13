<template>
  <div class="row items-center justify-end q-mt-sm">
    <q-pagination
      :model-value="page"
      :max="totalPages"
      :max-pages="5"
      direction-links
      boundary-links
      @update:model-value="onPageChange"
    />
    <q-select
      :model-value="rowsPerPage"
      :options="[10, 20, 50, 100]"
      dense
      outlined
      class="q-ml-sm"
      style="min-width: 80px"
      @update:model-value="onRowsPerPageChange"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  page: number;
  rowsPerPage: number;
  total: number;
}>();

const emit = defineEmits<{
  (e: 'request', payload: { page: number; rowsPerPage: number }): void;
}>();

const totalPages = computed(() => Math.max(1, Math.ceil(props.total / props.rowsPerPage)));

function onPageChange(newPage: number) {
  emit('request', { page: newPage, rowsPerPage: props.rowsPerPage });
}

function onRowsPerPageChange(newRowsPerPage: number) {
  emit('request', { page: 1, rowsPerPage: newRowsPerPage });
}
</script>
