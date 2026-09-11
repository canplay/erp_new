/**
 * @file ImportDialog.vue
 * @description 数据导入对话框 - 组合 ImportPreview + 进度 + 流程控制
 * @date 2026-04-04
 */

<template>
  <q-dialog v-model="showDialog" persistent :maximized="isMobile">
    <q-card :style="{ width: isMobile ? '100%' : '900px', maxWidth: '900px' }">
      <!-- 标题栏 -->
      <q-card-section class="row items-center">
        <div class="text-h6">{{ $t('importMod.title') }}</div>
        <q-space />
        <q-btn flat round dense icon="close" @click="handleClose" />
      </q-card-section>

      <q-separator />

      <!-- 步骤指示器（桌面端） -->
      <q-card-section v-if="!isMobile">
        <q-stepper v-model="currentStep" vertical color="primary" animated>
          <q-step :name="1" :title="$t('importMod.stepUpload')" :done="currentStep > 1" icon="upload_file" />
          <q-step :name="2" :title="$t('importMod.stepMapping')" :done="currentStep > 2" icon="link" :disable="excelHeaders.length === 0" />
          <q-step :name="3" :title="$t('importMod.stepPreview')" :done="currentStep > 3" icon="preview" :disable="importRows.length === 0" />
          <q-step :name="4" :title="$t('importMod.stepResult')" :done="currentStep === 4" icon="check_circle" />
        </q-stepper>
      </q-card-section>

      <!-- 移动端简化视图 -->
      <q-card-section v-else>
        <div v-if="currentStep === 1" class="text-center q-pa-md">
          <q-icon name="upload_file" size="36px" color="grey-5" />
          <div class="q-mt-sm">{{ $t('importMod.tapToUpload') }}</div>
        </div>
        <div v-else-if="currentStep === 2" class="text-subtitle q-mb-sm">{{ $t('importMod.fieldMapping') }}</div>
        <div v-else-if="currentStep === 3" class="text-subtitle q-mb-sm">
          {{ selectedRows.size }} {{ $t('importMod.rowsSelected') }}
        </div>
      </q-card-section>

      <!-- 预览组件 -->
      <ImportPreview
        :current-step="currentStep"
        :max-file-size="options.maxFileSize"
        :accept-formats="options.accept"
        :mapping-rows="mappingRows"
        :mapping-columns="mappingColumns"
        :import-rows="importRows"
        :preview-columns="previewColumns"
        :selected-table-rows="selectedTableRows"
        :selected-count="selectedRows.size"
        :total-rows="importRows.length"
        :excel-headers-length="excelHeaders.length"
        @apply-mapping="handleApplyMapping"
        @select="handleSelectAll"
        @download-errors="handleDownloadErrors"
        @file-selected="(file: File) => handleFileSelected(file)"
        @back="handleBack"
        @next="handleNext"
        @import="handleImport"
        @close="handleClose"
      />

      <q-separator />

      <!-- 底部操作栏 -->
      <q-card-actions align="right" class="q-pa-md">
        <q-btn v-if="currentStep > 1 && currentStep < 4" flat color="grey" :label="$t('common.back')" @click="handleBack" />
        <q-btn v-if="currentStep < 3" color="primary" :label="$t('common.next')" :disable="!canProceed" @click="handleNext" />
        <q-btn v-if="currentStep === 3" color="primary" :label="$t('importMod.startImport')" :loading="isImporting" :disable="selectedRows.size === 0" @click="handleImport" />
        <q-btn v-if="currentStep === 4" color="primary" :label="$t('common.done')" @click="handleClose" />
      </q-card-actions>

      <!-- 导入进度 -->
      <q-linear-progress v-if="isImporting" :value="importProgress / 100" color="primary" class="q-mt-sm" />
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useImport } from '@/composables/useImport';
import ImportPreview from './ImportPreview.vue';

const { t } = useI18n();
const $q = useQuasar();

const props = withDefaults(defineProps<{
  modelValue: boolean;
  targetFields?: string[];
  requiredFields?: string[];
}>(), {
  targetFields: () => [],
  requiredFields: () => [],
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  success: [data: unknown[]];
  error: [error: Error];
}>();

// 状态
const isMobile = computed(() => $q.screen.lt.md);
const {
  isImporting, importProgress, excelHeaders, fieldMappings,
  importRows, selectedRows, parseFile, applyMapping,
  executeImport, downloadErrorReport, reset,
} = useImport();

const showDialog = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val),
});

const currentStep = ref(1);
const selectedTableRows = ref<unknown[]>([]);
const result = ref<{ successCount: number; failCount: number } | null>(null);

watch(selectedTableRows, (newRows) => {
  importRows.value.forEach((row) => {
    (row as { selected?: boolean }).selected = newRows.some((r) => (r as { rowIndex?: number }).rowIndex === row.rowIndex);
  });
});

const dataTypeOptions = ['string', 'number', 'date', 'boolean'];

interface _MappingRow {
  excelColumn: string;
  targetField: string;
  required: boolean;
  dataType: string;
}

const mappingRows = computed(() => fieldMappings.value);

const mappingColumns = [
  { name: 'excelColumn', label: t('importMod.excelColumn'), field: 'excelColumn', align: 'left' as const },
  { name: 'targetField', label: t('importMod.targetField'), field: 'targetField', align: 'left' as const },
  { name: 'required', label: t('importMod.required'), field: 'required', align: 'center' as const },
  { name: 'dataType', label: t('importMod.dataType'), field: 'dataType', align: 'center' as const },
];

const previewColumns = computed(() => {
  if (importRows.value.length === 0) return [];
  return [
    { name: 'rowIndex', label: t('importMod.rowNumber'), field: 'rowIndex', align: 'center' as const },
    { name: 'status', label: t('importMod.status'), field: 'status', align: 'center' as const },
    ...Object.keys(importRows.value[0]?.raw || {}).map((key) => ({
      name: key, label: key, field: key, align: 'left' as const,
    })),
  ];
});

const options = { maxFileSize: 10, accept: ['.xlsx', '.xls', '.csv'] };

const canProceed = computed(() => {
  switch (currentStep.value) {
    case 1: return excelHeaders.value.length > 0;
    case 2: return fieldMappings.value.length > 0;
    case 3: return selectedRows.value.size > 0;
    default: return true;
  }
});

function handleNext() {
  if (canProceed.value) currentStep.value++;
}

function handleBack() {
  if (currentStep.value > 1) currentStep.value--;
}

async function handleApplyMapping() {
  await applyMapping();
  currentStep.value = 3;
}

async function handleImport() {
  const selected = importRows.value.filter((row) => (row as { selected?: boolean }).selected);
  const res = await executeImport((data) => Promise.resolve({ data: { total: selected.length, failed: 0, errors: [] } }));
  result.value = res;
  currentStep.value = 4;
  emit('success', selected);
}

function handleSelectAll(select: boolean) {
  importRows.value.forEach((row) => {
    (row as { selected?: boolean }).selected = select;
  });
}

function handleDownloadErrors() {
  downloadErrorReport();
}

function handleFileSelected(file: File) {
  void parseFile(file).then(() => { currentStep.value = 2; });
}

function handleClose() {
  currentStep.value = 1;
  emit('update:modelValue', false);
}
</script>

<style scoped>
/* 步骤指示器样式 */
</style>
