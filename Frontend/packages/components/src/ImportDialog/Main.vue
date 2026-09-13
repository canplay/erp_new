<template>
  <div class="import-dialog">
    <q-dialog v-model="showDialog" persistent>
      <q-card class="import-dialog__card">
        <q-card-section class="row items-center q-pb-none">
          <div class="text-h6">{{ $t('import.title') }}</div>
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-card-section class="import-dialog__content">
          <StepperIndicator :currentStep="currentStep" />

          <q-separator />

          <StepPanel
            :step="currentStep"
            :file="selectedFile"
            :is-uploading="isUploading"
            :upload-progress="uploadProgress"
            :validate-result="validateResult as { success?: boolean; message?: string; errors?: string[] }"
            :preview-data="previewData as Record<string, unknown>[]"
            @select-file="handleFileSelect"
            @start-upload="startUpload"
            @next-step="nextStep"
            @prev-step="prevStep"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat color="grey" :label="$t('common.cancel')" v-close-popup @click="reset" />
          <q-btn
            v-if="currentStep > 0"
            flat
            color="primary"
            :label="$t('common.prevStep')"
            @click="prevStep"
          />
          <q-btn
            v-if="currentStep < 2"
            color="primary"
            :label="$t('common.nextStep')"
            :disable="!canNext"
            @click="nextStep"
          />
          <q-btn
            v-if="currentStep === 2"
            color="positive"
            :label="$t('import.import')"
            :disable="!canImport"
            @click="importData"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useImportDialog } from './useImportDialog'
import StepperIndicator from './StepperIndicator.vue'
import StepPanel from './StepPanel.vue'

const emit = defineEmits<{
  (e: 'import', data: { success?: boolean; message?: string; errors?: string[] }[]): void
  (e: 'close'): void
}>()

const {
  showDialog,
  currentStep,
  selectedFile,
  isUploading,
  uploadProgress,
  validateResult,
  previewData,
  canNext,
  canImport,
  handleFileSelect,
  startUpload,
  nextStep,
  prevStep,
  importData,
  reset
} = useImportDialog({
  onImport: emit as any,
  onClose: emit
})
</script>

<style scoped>
.import-dialog__card {
  width: 700px;
  max-width: 90vw;
}

.import-dialog__content {
  min-height: 300px;
}
</style>
