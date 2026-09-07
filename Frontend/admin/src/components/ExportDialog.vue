/**
 * @file ExportDialog.vue
 * @description 数据导入对话框 - 组合 ExportPreview + ExportProgress + 上传逻辑
 * @date 2026-04-04
 */

<template>
  <q-dialog v-model="isOpen" persistent>
    <q-card style="min-width: 500px">
      <q-card-section>
        <div class="text-h6">{{ title }}</div>
      </q-card-section>

      <!-- 拖拽区域 -->
      <q-card-section v-if="!isUploading && !uploadSuccess && !uploadError && !fileToUpload">
        <div
          class="drop-zone"
          :class="{ 'drop-zone-active': isDragging }"
          @dragover.prevent="onDragOver"
          @dragleave.prevent="onDragLeave"
          @drop.prevent="onDrop"
        >
          <q-icon name="cloud_upload" size="48px" color="grey-6" />
          <div class="q-mt-md text-grey-7">
            拖拽文件到此处，或
            <q-btn flat color="primary" label="点击选择" @click="selectFile" />
          </div>
          <div class="text-caption text-grey-5 q-mt-sm">
            支持的文件类型: {{ acceptedTypes }}
            <br />
            最大文件大小: {{ formatFileSize(maxFileSize) }}
          </div>
        </div>

        <!-- 分片上传选项 -->
        <div v-if="enableChunkedUpload" class="q-mt-md">
          <q-toggle v-model="useChunkedUpload" label="启用分片上传（适合大文件）" />
          <div v-if="useChunkedUpload" class="q-mt-sm text-caption text-grey">
            分片大小: {{ formatFileSize(chunkSize) }}
          </div>
        </div>
      </q-card-section>

      <!-- 预览和文件信息 -->
      <ExportPreview
        :file-info="fileInfo ?? undefined"
        :file-to-upload="fileToUpload"
        :show-extra-params="showExtraParams"
        :preview-data="previewData"
        :total-rows="totalRows"
        :preview-columns="previewColumns"
        @clear-file="clearFile"
      />

      <!-- 进度和状态 -->
      <ExportProgress
        :is-uploading="isUploading"
        :upload-progress="uploadProgress"
        :upload-speed="uploadSpeed"
        :upload-success="uploadSuccess"
        :upload-error="uploadError"
        @close="handleClose"
        @retry="retryUpload"
      />

      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" color="grey" @click="handleClose" />
        <q-btn
          v-if="fileToUpload && !isUploading && !uploadSuccess"
          color="primary"
          label="开始上传"
          :loading="isUploading"
          @click="startUpload"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useQuasar } from 'quasar';
import { useAuthStore } from '@/stores/auth';
import ExcelJS from 'exceljs';
import ExportPreview from './ExportDialog/ExportPreview.vue';
import ExportProgress from './ExportDialog/ExportProgress.vue';

interface UploadFile {
  file: File;
  name: string;
  size: number;
  type: string;
}

interface UploadResult {
  success: boolean;
  message: string;
  url?: string;
}

interface Props {
  modelValue?: boolean;
  title?: string;
  uploadUrl?: string;
  acceptedTypes?: string;
  maxFileSize?: number;
  enableChunkedUpload?: boolean;
  chunkSize?: number;
  showExtraParams?: boolean;
  showPreview?: boolean;
  previewColumns?: Array<{ name: string; label: string; field: string }>;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  title: '导入数据',
  uploadUrl: '/files/upload',
  acceptedTypes: '.xlsx,.xls,.csv',
  maxFileSize: 100 * 1024 * 1024,
  enableChunkedUpload: true,
  chunkSize: 5 * 1024 * 1024,
  showExtraParams: true,
  showPreview: false,
  previewColumns: () => [],
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'upload', file: File, params: Record<string, string>): void;
  (e: 'success', result: UploadResult): void;
  (e: 'error', error: string): void;
}>();

const $q = useQuasar();

const isDragging = ref(false);
const fileToUpload = ref<UploadFile | null>(null);
const isUploading = ref(false);
const uploadProgress = ref(0);
const uploadSpeed = ref(0);
const uploadSuccess = ref(false);
const uploadError = ref('');
const useChunkedUpload = ref(true);
const previewData = ref<Record<string, unknown>[]>([]);
const totalRows = ref(0);

const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const fileInfo = computed(() => fileToUpload.value ? { name: fileToUpload.value.name, size: fileToUpload.value.size } : undefined);

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k,i)).toFixed(2))} ${sizes[i]}`;
}

function formatSpeed(bytesPerSecond: number): string {
  return formatFileSize(bytesPerSecond) + '/s';
}

function onDragOver() { isDragging.value = true; }
function onDragLeave() { isDragging.value = false; }

function onDrop(event: DragEvent) {
  isDragging.value = false;
  const files = event.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    if (file) void handleFileSelect(file);
  }
}

function selectFile() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = props.acceptedTypes;
  input.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (file) void handleFileSelect(file);
  };
  input.click();
}

async function handleFileSelect(file: File) {
  if (file.size > props.maxFileSize) {
    $q.notify({ type: 'negative', message: `文件大小超过限制（${formatFileSize(props.maxFileSize)}）` });
    return;
  }
  const ext = '.' + file.name.split('.').pop()?.toLowerCase();
  const accepted = props.acceptedTypes.split(',').map((t) => t.trim().toLowerCase());
  if (!accepted.includes(ext)) {
    $q.notify({ type: 'negative', message: `不支持的文件类型（${ext}）` });
    return;
  }
  fileToUpload.value = { file, name: file.name, size: file.size, type: file.type };
  if (props.showPreview && (ext.includes('xlsx') || ext.includes('csv') || ext.includes('xls'))) {
    await loadPreview(file);
  }
}

async function loadPreview(file: File) {
  try {
    const workbook = new ExcelJS.Workbook();
    const arrayBuffer = await file.arrayBuffer();
    await workbook.xlsx.load(arrayBuffer);
    const worksheet = workbook.getWorksheet(1);
    const rows: Record<string, unknown>[] = [];
    let count = 0;
    worksheet?.eachRow((row) => {
      if (count < 5) {
        const rowData: Record<string, unknown> = {};
        row.eachCell?.((cell, colNumber) => {
          rowData[`col${colNumber}`] = cell.value;
        });
        rows.push(rowData);
      }
      count++;
    });
    previewData.value = rows;
    totalRows.value = worksheet?.rowCount || count;
  } catch (error) {
    console.error('Preview error:', error);
  }
}

function clearFile() {
  fileToUpload.value = null;
  previewData.value = [];
  totalRows.value = 0;
}

async function startUpload() {
  if (!fileToUpload.value) return;
  isUploading.value = true;
  uploadProgress.value = 0;
  uploadError.value = '';
  uploadSuccess.value = false;
  try {
    if (useChunkedUpload.value && fileToUpload.value.size > props.chunkSize) {
      await uploadInChunks();
    } else {
      await uploadFile();
    }
    uploadSuccess.value = true;
    emit('success', { success: true, message: '上传成功', url: '' });
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : '上传失败';
    uploadError.value = message;
    emit('error', uploadError.value);
  } finally {
    isUploading.value = false;
  }
}

async function uploadInChunks() {
  const file = fileToUpload.value;
  if (!file) throw new Error('No file to upload');
  const totalChunks = Math.ceil(file.size / props.chunkSize);
  let uploadedChunks = 0;
  let lastTime = Date.now();
  let lastBytes = 0;
  for (let i = 0; i < totalChunks; i++) {
    const start = i * props.chunkSize;
    const end = Math.min(start + props.chunkSize, file.size);
    const chunk = file.file.slice(start, end);
    const formData = new FormData();
    formData.append('file', chunk);
    formData.append('filename', file.name);
    formData.append('chunkIndex', i.toString());
    formData.append('totalChunks', totalChunks.toString());
    await fetch(props.uploadUrl, {
      method: 'POST',
      body: formData,
      headers: { Authorization: `Bearer ${useAuthStore().token || ''}` },
    }).then((res) => {
      if (!res.ok) throw new Error(`分片上传失败: HTTP ${res.status}`);
    });
    uploadedChunks++;
    uploadProgress.value = Math.round((uploadedChunks / totalChunks) * 100);
    const now = Date.now();
    const elapsed = (now - lastTime) / 1000;
    if (elapsed >= 1) {
      uploadSpeed.value = (uploadedChunks * props.chunkSize - lastBytes) / elapsed;
      lastTime = now;
      lastBytes = uploadedChunks * props.chunkSize;
    }
  }
}

async function uploadFile() {
  const file = fileToUpload.value;
  if (!file) throw new Error('No file to upload');
  const formData = new FormData();
  formData.append('file', file.file);
  const start_time = Date.now();
  const response = await fetch(props.uploadUrl, {
    method: 'POST',
    body: formData,
    headers: { Authorization: `Bearer ${useAuthStore().token || ''}` },
  });
  if (!response.ok) throw new Error(`上传失败: ${response.statusText}`);
  const elapsed = (Date.now() - start_time) / 1000;
  uploadSpeed.value = file.size / elapsed;
}

function retryUpload() {
  uploadError.value = '';
  void startUpload();
}

function handleClose() {
  isOpen.value = false;
  clearFile();
  uploadProgress.value = 0;
  uploadSpeed.value = 0;
  uploadSuccess.value = false;
  uploadError.value = '';
}

watch(isOpen, (newVal) => {
  if (!newVal) {
    clearFile();
    uploadProgress.value = 0;
    uploadSpeed.value = 0;
    uploadSuccess.value = false;
    uploadError.value = '';
  }
});
</script>

<style scoped>
.drop-zone {
  border: 2px dashed rgba(0, 0, 0, 0.12);
  border-radius: 8px;
  padding: 48px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s;
}
.drop-zone:hover {
  border-color: var(--q-primary);
  background-color: rgba(0, 0, 0, 0.02);
}
.drop-zone-active {
  border-color: var(--q-primary);
  background-color: rgba(0, 0, 0, 0.05);
}
</style>
