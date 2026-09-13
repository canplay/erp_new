/**
 * @file FileUploader.vue
 * @description 文件上传组件 - 支持切片上传、压缩、进度显示
 * @date 2026-04-03
 */

<template>
  <div class="file-uploader">
    <!-- 上传按钮 -->
    <q-btn
      color="primary"
      :icon="icon"
      :label="label"
      :loading="uploading"
      :disable="uploading"
      @click="triggerUpload"
    >
      <template v-slot:loading>
        <q-spinner-hourglass class="on-left" />
        {{ $t('uploader.uploading') }}
      </template>
    </q-btn>

    <!-- 隐藏的文件输入框 -->
    <input
      ref="fileInputRef"
      type="file"
      :accept="accept"
      :multiple="multiple"
      class="hidden-input"
      @change="handleFileChange"
    />

    <!-- 拖拽上传提示 -->
    <q-chip
      v-if="showDragHint"
      color="grey-3"
      text-color="grey-7"
      icon="info"
      class="q-ml-sm"
      size="sm"
    >
      {{ $t('uploader.dragHint') }}
    </q-chip>

    <!-- 已上传文件列表 -->
    <div v-if="files.length > 0" class="file-list q-mt-sm">
      <q-list dense>
        <q-item v-for="(file, index) in files" :key="index" class="file-item">
          <q-item-section avatar>
            <q-icon :name="getFileIcon(file.name)" color="primary" />
          </q-item-section>
          <q-item-section>
            <q-item-label>{{ file.name }}</q-item-label>
            <q-item-label caption>
              {{ formatFileSize(file.size) }}
              <span v-if="file.chunkSize" class="text-info q-ml-sm">
                ({{ $t('uploader.chunked') }}: {{ file.chunkSize }} {{ $t('uploader.chunks') }})
              </span>
            </q-item-label>
            <!-- 单文件进度条 -->
            <q-linear-progress
              :value="file.progress / 100"
              color="primary"
              track-color="grey-3"
              class="q-mt-xs"
              size="8px"
            />
          </q-item-section>
          <q-item-section side>
            <q-icon
              :name="file.status === 'success' ? 'check_circle' : file.status === 'error' ? 'error' : 'hourglass_empty'"
              :color="file.status === 'success' ? 'positive' : file.status === 'error' ? 'negative' : 'grey'"
              size="20px"
            />
            <q-btn flat round dense icon="close" size="sm" @click="removeFile(index)">
              <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
            </q-btn>
          </q-item-section>
        </q-item>
      </q-list>
    </div>

    <!-- 总上传进度 -->
    <div v-if="uploading && files.length > 0" class="q-mt-sm">
      <div class="text-caption text-grey-6">
        {{ $t('uploader.totalProgress') }}: {{ totalProgress }}%
      </div>
      <q-linear-progress
        :value="totalProgress / 100"
        color="primary"
        track-color="grey-3"
        class="q-mt-xs"
        size="12px"
      />
    </div>

    <!-- 上传历史（可下载） -->
    <div v-if="uploadedUrls.length > 0" class="uploaded-list q-mt-md">
      <div class="text-caption text-grey-6 q-mb-xs">
        {{ $t('uploader.uploadedFiles') }}：
      </div>
      <div v-for="(url, index) in uploadedUrls" :key="index" class="uploaded-item">
        <a :href="url" target="_blank" class="text-primary">
          {{ getFileNameFromUrl(url) }}
        </a>
        <q-btn flat round dense icon="content_copy" size="xs" @click="copyUrl(url)">
          <q-tooltip>{{ $t('uploader.copyUrl') }}</q-tooltip>
        </q-btn>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * @file FileUploader.vue
 * @description 文件上传组件 - script 仅做 props/emits 声明 + composable 解构
 * @date 2026-07-08
 */

import { useFileUploader } from '@erp-new-frontend-monorepo/composables/src/useFileUploader';;

// ============ Props & Emits ============

interface Props {
  modelValue?: string | string[];
  accept?: string;
  multiple?: boolean;
  maxSize?: number;
  maxFiles?: number;
  label?: string;
  icon?: string;
  uploadUrl?: string;
  chunkSize?: number;
  enableCompress?: boolean;
  compressMaxSize?: number;
  showDragHint?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  accept: '*/*',
  multiple: false,
  maxSize: 50,
  maxFiles: 10,
  label: '',
  icon: 'attach_file',
  uploadUrl: '/files/upload',
  chunkSize: 5,
  enableCompress: true,
  compressMaxSize: 1,
  showDragHint: true,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | string[]): void;
  (e: 'upload', files: File[]): void;
  (e: 'change', files: File[]): void;
  (e: 'progress', file: File, progress: number): void;
  (e: 'success', file: File, url: string): void;
  (e: 'error', file: File, error: Error): void;
}>();

// ============ Composable 解构 ============
const {
  fileInputRef,
  uploading,
  files,
  uploadedUrls,
  totalProgress,
  triggerUpload,
  handleFileChange,
  removeFile,
  getFileIcon,
  formatFileSize,
  getFileNameFromUrl,
  copyUrl,
} = useFileUploader(props, emit);
</script>

<style scoped>
.hidden-input {
  display: none;
}

.file-list {
  max-height: 300px;
  overflow-y: auto;
}

.file-item {
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  margin-bottom: 8px;
  padding: 8px 12px;
}

.uploaded-list {
  padding: 8px;
  background: #f5f5f5;
  border-radius: 4px;
}

.uploaded-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 0;
}

.uploaded-item a {
  text-decoration: none;
  word-break: break-all;
}

.uploaded-item a:hover {
  text-decoration: underline;
}

/* 暗色主题适配 */
.body--dark .file-item {
  border-color: #2d2d2d;
}

.body--dark .uploaded-list {
  background: #252525;
}

.body--dark .q-linear-progress {
  background: #2d2d2d;
}
</style>
