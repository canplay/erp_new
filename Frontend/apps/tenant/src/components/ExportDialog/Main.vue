<template>
  <div class="export-dialog">
    <q-dialog v-model="showDialog" persistent>
      <q-card class="export-dialog__card">
        <q-card-section class="row items-center q-pb-none">
          <div class="text-h6">{{ $t('export.title') }}</div>
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-card-section class="export-dialog__content">
          <div class="text-subtitle2 q-mb-md">{{ $t('export.options') }}</div>

          <q-select
            v-model="selectedFormat"
            :options="formatOptions"
            emit-value
            map-options
            dense
            outlined
            :label="$t('export.format')"
            class="q-mb-md"
          />

          <q-input
            v-model="fileName"
            dense
            outlined
            :label="$t('export.fileName')"
            class="q-mb-md"
          />

          <div v-if="isExporting" class="export-progress">
            <q-linear-progress :value="exportProgress / 100" color="primary" />
            <div class="text-caption q-mt-xs">{{ $t('export.exporting') }} {{ exportProgress }}%</div>
          </div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat color="grey" :label="$t('common.cancel')" v-close-popup />
          <q-btn
            color="primary"
            :label="$t('export.export')"
            :disable="isExporting || !fileName"
            @click="startExport"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useExportDialog } from './useExportDialog'

interface Props {
  formatOptions?: Array<{ label: string; value: string }>
}

const props = withDefaults(defineProps<Props>(), {
  formatOptions: () => [
    { label: 'Excel', value: 'excel' },
    { label: 'CSV', value: 'csv' },
    { label: 'PDF', value: 'pdf' }
  ]
})

const emit = defineEmits<{
  (e: 'export', payload: { format: string; fileName: string }): void
  (e: 'close'): void
}>()

const {
  showDialog,
  selectedFormat,
  fileName,
  isExporting,
  exportProgress,
  startExport
} = useExportDialog({
  formatOptions: props.formatOptions,
  onExport: emit,
  onClose: emit
})
</script>

<style scoped>
.export-dialog__card {
  width: 400px;
}

.export-dialog__content {
  min-height: 200px;
}

.export-progress {
  margin-top: 16px;
}
</style>
