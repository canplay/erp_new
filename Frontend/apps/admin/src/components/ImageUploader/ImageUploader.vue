/**
 * @file ImageUploader.vue
 * @description 图片上传组件 - 支持压缩、裁剪、预览
 * @date 2026-04-03
 */

<template>
  <div class="image-uploader">
    <!-- 上传区域 -->
    <div
      class="upload-zone"
      :class="{ 'drag-over': isDragOver, 'has-image': previewUrl }"
      @click="triggerUpload"
      @dragover.prevent="isDragOver = true"
      @dragleave.prevent="isDragOver = false"
      @drop.prevent="handleDrop"
    >
      <img v-if="previewUrl" :src="previewUrl" class="preview-image" />
      <div v-else class="upload-hint">
        <q-icon name="add_photo_alternate" size="48px" color="grey-5" />
        <div class="text-grey-6 q-mt-sm">{{ $t('uploader.clickOrDrag') }}</div>
        <div class="text-caption text-grey-5 q-mt-xs">{{ $t('uploader.supportedFormats') }}</div>
      </div>
      <q-btn v-if="previewUrl" class="delete-btn" round dense color="negative" icon="close" size="sm" @click.stop="removeImage">
        <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
      </q-btn>
      <q-btn v-if="previewUrl && showEdit" class="edit-btn" round dense color="primary" icon="edit" size="sm" @click.stop="openCrop">
        <q-tooltip>{{ $t('uploader.editImage') }}</q-tooltip>
      </q-btn>
    </div>
    <input ref="fileInputRef" type="file" :accept="acceptTypes" class="hidden-input" @change="handleFileChange" />
    <div v-if="currentFile" class="image-info q-mt-sm">
      <div class="text-caption text-grey-6">
        {{ currentFile.name }} · {{ formatFileSize(currentFile.size) }}
        <span v-if="compressedSize" class="text-positive q-ml-sm">({{ $t('uploader.compressed') }}: {{ formatFileSize(compressedSize) }})</span>
      </div>
    </div>
    <UploadProgress v-if="uploading" :progress="uploadProgress" />
    <CropDialog v-model:visible="showCrop" :source-url="cropSourceUrl" :quality="compressQuality" @apply="onCropApply" />
    <PreviewDialog v-model:visible="showPreview" :image-url="previewUrl" />
  </div>
</template>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue';
import { logger } from '@/utils/logger';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { httpClient } from '@/utils/alova';
import CropDialog from './CropDialog.vue';
import PreviewDialog from './PreviewDialog.vue';
import UploadProgress from './UploadProgress.vue';

const { t } = useI18n();
const $q = useQuasar();

interface Props {
  modelValue?: string;
  multiple?: boolean;
  maxSize?: number;
  maxWidth?: number;
  maxHeight?: number;
  showEdit?: boolean;
  uploadUrl?: string;
  enableCompress?: boolean;
  compressQuality?: number;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '', multiple: false, maxSize: 5, maxWidth: 1920, maxHeight: 1920,
  showEdit: true, uploadUrl: '/files/upload', enableCompress: true, compressQuality: 0.8,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'upload', file: File): void;
  (e: 'change', file: File): void;
  (e: 'progress', progress: number): void;
}>();

const fileInputRef = ref<HTMLInputElement | null>(null);
const isDragOver = ref(false);
const uploading = ref(false);
const uploadProgress = ref(0);
const currentFile = ref<File | null>(null);
const previewUrl = ref(props.modelValue || '');
const compressedSize = ref<number | null>(null);
const showCrop = ref(false);
const showPreview = ref(false);
const cropSourceUrl = ref('');
const acceptTypes = 'image/jpeg,image/png,image/webp,image/gif';

function triggerUpload() {
  if (!previewUrl.value) fileInputRef.value?.click();
  else if (props.showEdit) openCrop();
}

function handleFileChange(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (file) void processFile(file);
  (event.target as HTMLInputElement).value = '';
}

function handleDrop(event: DragEvent) {
  isDragOver.value = false;
  const file = event.dataTransfer?.files?.[0];
  if (file && file.type.startsWith('image/')) void processFile(file);
  else $q.notify({ type: 'warning', message: t('uploader.imageOnly') });
}

async function processFile(file: File) {
  if (!file.type.startsWith('image/')) {
    $q.notify({ type: 'warning', message: t('uploader.imageOnly') || '请上传图片文件' });
    return;
  }
  const maxSizeBytes = props.maxSize * 1024 * 1024;
  if (file.size > maxSizeBytes) {
    $q.notify({ type: 'negative', message: `${file.name}: ${t('uploader.fileTooLarge')} (${props.maxSize}MB)` });
    return;
  }
  currentFile.value = file;
  previewUrl.value = URL.createObjectURL(file);
  await uploadImage(file);
}

async function uploadImage(file: File) {
  uploading.value = true;
  uploadProgress.value = 0;
  try {
    let fileToUpload = file;
    if (props.enableCompress) {
      fileToUpload = await compressImage(file);
      if (fileToUpload !== file) compressedSize.value = fileToUpload.size;
    }
    const formData = new FormData();
    formData.append('file', fileToUpload);
    const response = await httpClient.post(props.uploadUrl, formData, {
      onUploadProgress: (e: ProgressEvent) => {
        if (e.total) {
          uploadProgress.value = Math.round((e.loaded / e.total) * 100);
          emit('progress', uploadProgress.value);
        }
      },
    }) as { data?: { url?: string; data?: { url?: string } } };
    const url = response.data?.data?.url || response.data?.url || previewUrl.value;
    previewUrl.value = url;
    emit('update:modelValue', url);
    emit('upload', file);
    emit('change', file);
    $q.notify({ type: 'positive', message: t('uploader.uploadSuccess') });
  } catch (error) {
    logger.error('【图片上传失败】', error);
    $q.notify({ type: 'negative', message: t('uploader.uploadFailed') });
  } finally {
    uploading.value = false;
  }
}

async function compressImage(file: File): Promise<File> {
  return new Promise((resolve) => {
    const img = new Image();
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    img.onload = () => {
      let { width, height } = img;
      if (width > props.maxWidth || height > props.maxHeight) {
        const ratio = Math.min(props.maxWidth / width, props.maxHeight / height);
        width = Math.round(width * ratio);
        height = Math.round(height * ratio);
      }
      canvas.width = width;
      canvas.height = height;
      ctx?.drawImage(img, 0, 0, width, height);
      canvas.toBlob((blob) => {
        if (blob) {
          const cf = new File([blob], file.name, { type: 'image/jpeg' });
          logger.info(`【图片压缩】${file.name}: ${formatFileSize(file.size)} -> ${formatFileSize(blob.size)}`);
          URL.revokeObjectURL(img.src);
          resolve(cf);
        } else resolve(file);
      }, 'image/jpeg', props.compressQuality);
    };
    img.onerror = () => { URL.revokeObjectURL(img.src); resolve(file); };
    img.src = URL.createObjectURL(file);
  });
}

function openCrop() {
  cropSourceUrl.value = previewUrl.value;
  showCrop.value = true;
}

function onCropApply(blob: Blob, _fileName: string) {
  const url = URL.createObjectURL(blob);
  previewUrl.value = url;
  const f = new File([blob], currentFile.value?.name || 'cropped.jpg', { type: 'image/jpeg' });
  void uploadImage(f);
}

function removeImage() {
  previewUrl.value = '';
  currentFile.value = null;
  compressedSize.value = null;
  uploadProgress.value = 0;
  emit('update:modelValue', '');
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024, sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}
</script>

<style scoped>
.image-uploader { width: 100%; }
.upload-zone {
  position: relative; width: 200px; height: 200px; border: 2px dashed #ccc;
  border-radius: 8px; display: flex; align-items: center; justify-content: center;
  cursor: pointer; transition: all 0.3s ease; overflow: hidden; background: #f5f5f5;
}
.upload-zone:hover { border-color: #1976d2; background: rgba(25, 118, 210, 0.05); }
.upload-zone.drag-over { border-color: #1976d2; background: rgba(25, 118, 210, 0.1); }
.upload-zone.has-image { border-style: solid; }
.preview-image { width: 100%; height: 100%; object-fit: cover; }
.upload-hint { text-align: center; padding: 16px; }
.delete-btn { position: absolute; top: 8px; right: 8px; }
.edit-btn { position: absolute; top: 8px; right: 48px; }
.hidden-input { display: none; }
.image-info { padding: 8px 12px; background: #f5f5f5; border-radius: 4px; }

/* Dark theme */
.body--dark .upload-zone { background: #252525; border-color: #404040; }
.body--dark .upload-zone:hover { border-color: #1976d2; background: rgba(25, 118, 210, 0.1); }
.body--dark .upload-hint { color: #b0b0b0; }
.body--dark .image-info { background: #2d2d2d; }
.body--dark .q-linear-progress { background: #2d2d2d; }
</style>
