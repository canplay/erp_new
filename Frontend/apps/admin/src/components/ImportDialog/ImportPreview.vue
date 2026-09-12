
/**
 * @file ImportPreview.vue
 * @description 导入预览组件 - 数据预览、字段映射、选择确认
 * @date 2026-08-22
 */

<template>
  <div class="import-preview">
    <!-- 上传区域 -->
    <div
      v-if="currentStep === 1"
      class="upload-zone"
      @click="triggerFileInput"
      @dragover.prevent="onDragOver"
      @dragleave="onDragLeave"
      @drop.prevent="onDrop"
    >
      <q-card flat bordered class="q-py-xl">
        <q-card-section class="text-center">
          <q-icon name="cloud_upload" size="64px" color="grey-5" />
          <div class="text-h6 q-mt-md">{{ $t('importMod.dragOrClick') }}</div>
          <div class="text-caption text-grey">
            {{ $t('importMod.supportedFormats') }}: .xlsx, .xls, .csv
          </div>
          <div class="text-caption text-grey">
            {{ $t('importMod.maxFileSize') }}: {{ maxFileSize }}MB
          </div>
        </q-card-section>
      </q-card>
      <input
        ref="fileInputRef"
        type="file"
        :accept="acceptFormats.join(',')"
        style="display: none"
        @change="onFileSelected"
      />
    </div>

    <!-- 字段映射区域 -->
    <div v-if="currentStep === 2" class="mapping-section">
      <div class="text-subtitle1 q-mb-md">{{ $t('importMod.mappingTitle') }}</div>
      <q-table
        :rows="mappingRows"
        :columns="mappingColumns"
        row-key="excelColumn"
        flat
        bordered
      >
        <template v-slot:body-cell-required="props">
          <q-td :props="props">
            <q-toggle v-model="props.row.required" dense />
          </q-td>
        </template>
        <template v-slot:body-cell-targetField="props">
          <q-td :props="props">
            <q-select
              v-model="props.row.targetField"
              :options="targetFields"
              dense
              outlined
              :disable="!props.row.excelColumn"
              style="min-width: 150px"
            />
          </q-td>
        </template>
        <template v-slot:body-cell-dataType="props">
          <q-td :props="props">
            <q-select
              v-model="props.row.dataType"
              :options="dataTypeOptions"
              dense
              outlined
              style="min-width: 100px"
            />
          </q-td>
        </template>
      </q-table>
      <q-btn
        color="primary"
        :label="$t('importMod.applyMapping')"
        class="q-mt-md"
        @click="$emit('applyMapping')"
      />
    </div>

    <!-- 预览确认区域 -->
    <div v-if="currentStep === 3" class="preview-section">
      <div class="row items-center q-mb-md">
        <div class="text-subtitle1">
          {{ $t('importMod.previewTitle') }}: {{ selectedCount }} / {{ totalRows }}
          {{ $t('importMod.rowsSelected') }}
        </div>
        <q-space />
        <q-btn flat color="primary" :label="$t('importMod.selectAll')" @click="$emit('select', true)" />
        <q-btn flat color="grey" :label="$t('importMod.unselectAll')" @click="$emit('select', false)" />
      </div>
      <q-table
        :rows="importRows"
        :columns="previewColumns"
        row-key="rowIndex"
        flat
        bordered
        selection="multiple"
        v-model:selected="selectedTableRows"
        :pagination="{ rowsPerPage: 10 }"
      >
        <template v-slot:body-cell-status="props">
          <q-td :props="props">
            <q-badge
              v-if="props.row.errors.length > 0"
              color="negative"
              :label="`${props.row.errors.length} ${$t('importMod.errors')}`"
            />
            <q-badge v-else color="positive" :label="$t('importMod.valid')" />
          </q-td>
        </template>
        <template v-slot:body-cell-selected="props">
          <q-td :props="props">
            <q-checkbox
              v-model="props.row.selected"
              :disable="props.row.errors.length > 0"
              dense
            />
          </q-td>
        </template>
      </q-table>
      <q-btn
        color="negative"
        :label="$t('importMod.downloadErrors')"
        class="q-mt-md"
        flat
        @click="$emit('downloadErrors')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import type { Column } from '@/composables/useExport';

const props = withDefaults(defineProps<{
  currentStep: number;
  maxFileSize: number;
  acceptFormats: string[];
  mappingRows: Array<{ excelColumn?: string; targetField?: string; dataType?: string; required?: boolean }>;
  mappingColumns: Column[];
  importRows: Array<{ rowIndex: number; selected: boolean; errors: Array<{ message?: string }> }>;
  targetFields: Array<{ label?: string; value?: string }>;
  selectedTableRows: Array<{ rowIndex: number }>;
  selectedCount: number;
  totalRows: number;
  excelHeadersLength: number;
}>(), {
  currentStep: 1,
  maxFileSize: 10,
  acceptFormats: () => ['.xlsx', '.xls', '.csv'],
  mappingRows: () => [],
  mappingColumns: () => [],
  importRows: () => [],
  targetFields: () => [],
  selectedTableRows: () => [],
  selectedCount: 0,
  totalRows: 0,
  excelHeadersLength: 0,
});

const emit = defineEmits<{
  'update:currentStep': [step: number];
  'applyMapping': [];
  'select': [selectAll: boolean];
  'downloadErrors': [];
  'fileSelected': [file: File];
  'dragOver': [e: DragEvent];
  'dragLeave': [e: DragEvent];
  'drop': [e: DragEvent];
  'back': [];
  'next': [];
  'import': [];
  'close': [];
}>();

const $q = useQuasar();
const fileInputRef = ref<HTMLInputElement | null>(null);
const dataTypeOptions = ['string', 'number', 'date', 'boolean'];

function triggerFileInput() {
  fileInputRef.value?.click();
}

function onDragOver(e: DragEvent) {
  (e.currentTarget as HTMLElement).classList.add('drag-over');
  emit('dragOver', e);
}

function onDragLeave(e: DragEvent) {
  (e.currentTarget as HTMLElement).classList.remove('drag-over');
  emit('dragLeave', e);
}

function onDrop(e: DragEvent) {
  (e.currentTarget as HTMLElement).classList.remove('drag-over');
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    emit('fileSelected', files[0]!);
  }
  emit('drop', e);
}

async function onFileSelected(e: Event) {
  const input = e.target as HTMLInputElement;
  if (input.files && input.files.length > 0) {
    const file = input.files[0]!;
    if (file.size > props.maxFileSize * 1024 * 1024) {
      $q.notify({
        type: 'negative',
        message: `文件大小超过 ${props.maxFileSize}MB`,
      });
      return;
    }
    emit('fileSelected', file);
  }
}
</script>

<style scoped>
.upload-zone {
  cursor: pointer;
}

.upload-zone :deep(.q-card) {
  transition: all 0.2s ease;
}

.upload-zone :deep(.q-card:hover) {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.upload-zone.drag-over :deep(.q-card) {
  border-color: #667eea !important;
  background: rgba(102, 126, 234, 0.05);
}

.mapping-section,
.preview-section {
  padding: 8px 0;
}
</style>
