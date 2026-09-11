<template>
  <div class="step-panel">
    <!-- 步骤1: 上传文件 -->
    <div v-if="step === 0" class="step-panel__upload">
      <q-uploader
        :label="$t('import.selectFile')"
        accept=".csv,.xlsx,.xls"
        :max-file-size="10485760"
        @add="onFileSelect"
      />
      <div v-if="isUploading" class="upload-progress q-mt-md">
        <q-linear-progress :value="uploadProgress / 100" color="primary" />
        <div class="text-caption q-mt-xs">{{ $t('import.uploading') }} {{ uploadProgress }}%</div>
      </div>
      <q-btn
        v-if="!isUploading"
        color="primary"
        :label="$t('import.upload')"
        class="q-mt-md"
        :disable="!file"
        @click="$emit('startUpload')"
      />
    </div>

    <!-- 步骤2: 验证结果 -->
    <div v-if="step === 1" class="step-panel__validate">
      <div v-if="validateResult" class="validate-result">
        <q-icon :name="validateResult.success ? 'check_circle' : 'error'" :color="validateResult.success ? 'positive' : 'negative'" size="32px" />
        <div class="text-h6 q-mt-sm">{{ validateResult.message }}</div>
        <div v-if="!validateResult.success" class="text-caption text-negative q-mt-xs">
          {{ validateResult.errors?.join(', ') }}
        </div>
      </div>
      <q-btn
        v-if="validateResult?.success"
        color="primary"
        :label="$t('import.preview')"
        class="q-mt-md"
        @click="$emit('nextStep')"
      />
    </div>

    <!-- 步骤3: 数据预览 -->
    <div v-if="step === 2" class="step-panel__preview">
      <q-table
        :rows="previewData"
        :columns="previewColumns"
        row-key="index"
        dense
        flat
        virtual-scroll
        virtual-scroll-item-size="40"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  step: number
  file: File | null
  isUploading: boolean
  uploadProgress: number
  validateResult: { success?: boolean; message?: string; errors?: string[] }
  previewData: Record<string, unknown>[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'selectFile', file: File): void
  (e: 'startUpload'): void
  (e: 'nextStep'): void
  (e: 'prevStep'): void
}>()

const previewColumns = computed(() => {
  if (!props.previewData || props.previewData.length === 0) return []
  return Object.keys(props.previewData[0] ?? {}).map(key => ({
    name: key,
    label: key,
    field: key,
    align: 'left' as const
  }))
})

function onFileSelect(files: File[]): void {
  const file = files[0]
  if (file) {
    emit('selectFile', file)
  }
}
</script>

<style scoped>
.step-panel__upload,
.step-panel__validate,
.step-panel__preview {
  padding: 16px 0;
}

.upload-progress {
  margin-top: 16px;
}

.validate-result {
  text-align: center;
}
</style>
