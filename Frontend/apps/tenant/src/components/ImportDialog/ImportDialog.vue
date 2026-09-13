/**
 * @file ImportDialog.vue
 * @description 数据导入对话框 - 主文件（拆分后）
 * @date 2026-04-04
 */

<template>
  <q-dialog v-model="props.modelValue" persistent :maximized="isMobile">
    <q-card :style="{ width: isMobile ? '100%' : '900px', maxWidth: '900px' }">
      <!-- 标题栏 -->
      <q-card-section class="row items-center">
        <div class="text-h6">{{ $t('importMod.title') }}</div>
        <q-space />
        <q-btn flat round dense icon="close" @click="handleClose" />
      </q-card-section>

      <q-separator />

      <!-- 步骤指示器 (桌面端) -->
      <q-card-section v-if="!isMobile">
        <q-stepper v-model="currentStep" vertical color="primary" animated>
          <!-- 步骤1: 上传文件 -->
          <q-step :name="1" :title="$t('importMod.stepUpload')" :done="currentStep > 1" icon="upload_file">
            <UploadZone @file-selected="() => {}" @file-drop="onFileSelected" />
          </q-step>

          <!-- 步骤2: 字段映射 -->
          <q-step
            :name="2"
            :title="$t('importMod.stepMapping')"
            :done="currentStep > 2"
            icon="link"
            :disable="excelHeaders.length === 0"
          >
            <FieldMapping
              :mappings="mappingRows"
              :target-fields="targetFields"
              :data-type-options="dataTypeOptions"
              @apply="handleApplyMapping"
            />
          </q-step>

          <!-- 步骤3: 预览确认 -->
          <q-step
            :name="3"
            :title="$t('importMod.stepPreview')"
            icon="preview"
            :disable="importRows.length === 0"
          >
            <ImportPreview
              :rows="importRows"
              :selected-rows="selectedRows"
              :current-step="currentStep"
              :max-file-size="options.maxFileSize"
              :accept-formats="options.accept"
              :mapping-rows="mappingRows as unknown as Record<string, unknown>[]"
              :mapping-columns="mappingRows.map((r: MappingRow) => r.excelColumn)"
              :target-fields="targetFields || []"
              :data-type-options="dataTypeOptions"
              :show-save-button="false"
              :initial-conditions="[]"
              :storage-key="'import'"
              @select-all="handleSelectAll(true)"
              @unselect-all="handleSelectAll(false)"
              @download-errors="handleDownloadErrors"
            />
          </q-step>

          <!-- 步骤4: 导入结果 -->
          <q-step :name="4" :title="$t('importMod.stepResult')" :done="currentStep === 4" icon="check_circle">
            <ImportResult :result="result" />
          </q-step>
        </q-stepper>
      </q-card-section>

      <!-- 移动端简化视图 -->
      <q-card-section v-else>
        <div v-if="currentStep === 1" class="upload-zone-mobile" @click="triggerFileInput">
          <q-icon name="cloud_upload" size="48px" color="grey-5" />
          <div class="q-mt-sm">{{ $t('importMod.tapToUpload') }}</div>
        </div>
        <div v-else-if="currentStep === 2">
          <div class="text-subtitle q-mb-md">{{ $t('importMod.fieldMapping') }}</div>
          <q-list v-for="(mapping, index) in fieldMappings" :key="index" bordered separator class="q-mb-sm">
            <q-item>
              <q-item-section>{{ mapping.excelColumn }}</q-item-section>
              <q-item-section side>
                <q-select v-model="mapping.targetField" :options="targetFields" dense outlined style="min-width: 120px" />
              </q-item-section>
            </q-item>
          </q-list>
          <q-btn color="primary" :label="$t('importMod.applyMapping')" class="q-mt-md full-width" @click="handleApplyMapping" />
        </div>
        <div v-else-if="currentStep === 3">
          <div class="text-subtitle q-mb-md">{{ selectedRows.size }} {{ $t('importMod.rowsSelected') }}</div>
          <q-list v-for="row in importRows" :key="row.rowIndex" bordered separator class="q-mb-sm">
            <q-item>
              <q-item-section>
                <q-item-label>Row {{ row.rowIndex }}</q-item-label>
                <q-item-label caption v-if="row.errors.length > 0" class="text-negative">{{ row.errors[0]?.message }}</q-item-label>
              </q-item-section>
              <q-item-section side>
                <q-checkbox v-model="row.selected" :disable="row.errors.length > 0" />
              </q-item-section>
            </q-item>
          </q-list>
        </div>
      </q-card-section>

      <q-separator />

      <!-- 底部操作栏 -->
      <q-card-actions align="right" class="q-pa-md">
        <q-btn v-if="currentStep > 1 && currentStep < 4" flat color="grey" :label="$t('common.back')" @click="handleBack" />
        <q-btn v-if="currentStep < 3" color="primary" :label="$t('common.next')" :disable="!canProceed" @click="handleNext" />
        <q-btn v-if="currentStep === 3" color="primary" :label="$t('importMod.startImport')" :loading="isImporting" :disable="selectedRows.size === 0" @click="handleImport" />
        <q-btn v-if="currentStep === 4" color="primary" :label="$t('common.done')" @click="handleClose" />
      </q-card-actions>

      <!-- 导入进度条 -->
      <q-linear-progress v-if="isImporting" :value="importProgress / 100" color="primary" class="q-mt-sm" />
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useImport } from '@erp-new-frontend-monorepo/composables/src/useImport';;
import type { ImportRow } from '@erp-new-frontend-monorepo/composables/src/useImport';
import type { Column } from '@erp-new-frontend-monorepo/composables/src/useImport';

interface MappingRow {
  excelColumn: string;
  targetField: string;
  dataType: string;
  required: boolean;
}

interface Props {
  modelValue?: boolean;
  targetFields?: string[];
  requiredFields?: string[];
  importApi?: (data: Record<string, unknown>[]) => Promise<{ successCount: number; failCount: number; data: unknown[] }>;
  columns?: Column[];
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  targetFields: () => [],
  requiredFields: () => [],
  importApi: undefined as any,
  columns: () => [],
});
import UploadZone from './UploadZone.vue';
import FieldMapping from './FieldMapping.vue';
import ImportPreview from './ImportPreview.vue';
import ImportResult from './ImportResult.vue';

const { t } = useI18n();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  success: [data: unknown[]];
  error: [error: Error];
}>();

const $q = useQuasar();
const isMobile = computed(() => $q.screen.lt.md);

const {
  isImporting, importProgress, excelHeaders, fieldMappings,
  importRows, selectedRows, parseFile, applyMapping, executeImport,
  downloadErrorReport, reset,
} = useImport();

const currentStep = ref(1);
const fileInputRef = ref<HTMLInputElement | null>(null);
const result = ref<{ successCount: number; failCount: number } | null>(null);

const dataTypeOptions = ['string', 'number', 'date', 'boolean'];

const mappingRows = computed(() => fieldMappings.value as unknown as MappingRow[]);

const canProceed = computed(() => {
  switch (currentStep.value) {
    case 1: return excelHeaders.value.length > 0;
    case 2: return fieldMappings.value.length > 0;
    case 3: return selectedRows.value.size > 0;
    default: return true;
  }
});

const options = { maxFileSize: 10, accept: ['.xlsx', '.xls', '.csv'] };

function triggerFileInput() {
  fileInputRef.value?.click();
}

async function onFileSelected(file: File) {
  if (file.size > options.maxFileSize * 1024 * 1024) {
    $q.notify({ type: 'negative', message: t('importMod.fileTooLarge', { size: options.maxFileSize }) });
    return;
  }
  try {
    await parseFile(file);
    currentStep.value = 2;
  } catch (error) {
    // 错误已在 useImport 中处理
    console.warn('【文件解析失败，已处理】', error);
  }
}

function handleApplyMapping() {
  applyMapping();
  currentStep.value = 3;
}

function handleSelectAll(select: boolean) {
  importRows.value.forEach((row) => {
    if (row.errors.length === 0) {
      row.selected = select;
    }
  });
}

function handleDownloadErrors() {
  downloadErrorReport();
}

async function handleImport() {
  if (!props.importApi) {
    $q.notify({ type: 'warning', message: t('importMod.noImportApi') });
    currentStep.value = 4;
    return;
  }
  try {
    const importResult = await executeImport(props.importApi as any);
    result.value = { successCount: importResult.successCount, failCount: importResult.failCount };
    currentStep.value = 4;
    emit('success', importResult.data);
  } catch (err) {
    emit('error', err as Error);
  }
}

function handleBack() {
  currentStep.value--;
}

function handleNext() {
  currentStep.value++;
}

function handleClose() {
  currentStep.value = 1;
  result.value = null;
  void reset();
  emit('update:modelValue', false);
}
</script>

<style scoped>
.upload-zone {
  border: 2px dashed #ccc;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
}
.upload-zone:hover {
  border-color: #1976d2;
  background: rgba(25, 118, 210, 0.05);
}
.upload-zone.drag-over {
  border-color: #1976d2;
  background: rgba(25, 118, 210, 0.1);
}
.upload-zone-mobile {
  border: 2px dashed #ccc;
  border-radius: 8px;
  padding: 32px;
  text-align: center;
  cursor: pointer;
}
</style>
