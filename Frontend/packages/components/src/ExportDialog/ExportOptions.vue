<template>
  <div class="export-options">
    <q-select
      v-model="format"
      :options="options"
      emit-value
      map-options
      dense
      outlined
      :label="$t('export.format')"
      @update:model-value="$emit('update:format', $event)"
    />
    <q-input
      v-model="fileName"
      dense
      outlined
      :label="$t('export.fileName')"
      @input="$emit('update:fileName', $event)"
    />
  </div>
</template>

<script setup lang="ts">
interface Props {
  options: Array<{ label: string; value: string }>
  format: string
  fileName: string
}

defineProps<Props>()

defineEmits<{
  (e: 'update:format', value: string): void
  (e: 'update:fileName', value: string): void
}>()
</script>
