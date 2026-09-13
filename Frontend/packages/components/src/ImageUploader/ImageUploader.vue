/**
 * @file ImageUploader/ImageUploader.vue
 * @description 图片上传组件 - 主入口，保持原有公共 API
 * @date 2026-04-03
 */

<template>
  <div class="image-uploader">
    <!-- 拖拽上传区域 -->
    <DragDropZone
      :preview-url="previewUrl"
      :is-drag-over="isDragOver"
      :show-edit="showEdit"
      :click-or-click-drag="clickOrDragText"
      :supported-formats-text="supportedFormatsText"
      @trigger-upload="triggerUpload"
      @update:is-drag-over="isDragOver = $event"
      @drop="handleDrop"
      @remove="removeImage"
      @open-crop="openCropDialog"
    />

    <!-- 隐藏的文件输入框 -->
    <input
      ref="fileInputRef"
      type="file"
      :accept="acceptTypes"
      class="hidden-input"
      @change="handleFileChange"
    />

    <!-- 文件信息预览 -->
    <FilePreview
      v-if="currentFile"
      :file="currentFile"
      :compressed-size="compressedSize"
      :compressed-label="compressedText"
    />

    <!-- 上传进度 -->
    <ProgressBar
      v-if="uploading"
      :progress="uploadProgress"
      :uploading-text="uploadingText"
    />

    <!-- 裁剪弹窗 -->
    <ImageCropDialog
      v-model="showCropDialog"
      :source-url="cropSourceUrl"
      :scale="cropScale"
      :rotate="cropRotate"
      @update:scale="cropScale = $event"
      @update:rotate="cropRotate = $event"
      @apply="applyCrop"
    />

    <!-- 图片预览弹窗 -->
    <ImagePreviewDialog
      v-model="showPreviewDialog"
      :image-url="previewUrl"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * @file ImageUploader/ImageUploader.vue
 * @description 图片上传组件 - 主入口，保持原有公共 API
 * @date 2026-04-03
 */

import { ref } from 'vue';
import { logger } from '@/utils/logger';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { httpClient } from '@/utils/alova';
import ImageCropDialog from '../ImageCropDialog.vue';
import ImagePreviewDialog from '../ImagePreviewDialog.vue';
import DragDropZone from './DragDropZone.vue';
import FilePreview from './FilePreview.vue';
import ProgressBar from './ProgressBar.vue';

const { t } = useI18n();
const $q = useQuasar();

// ============ Props & Emits ============

interface Props {
  /** v-model 绑定值 */
  modelValue?: string;
  /** 是否多选 */
  multiple?: boolean;
  /** 最大文件大小（MB） */
  maxSize?: number;
  /** 最大宽度 */
  maxWidth?: number;
  /** 最大高度 */
  maxHeight?: number;
  /** 是否显示编辑按钮 */
  showEdit?: boolean;
  /** 上传 API 地址 */
  uploadUrl?: string;
  /** 是否启用压缩 */
  enableCompress?: boolean;
  /** 压缩质量 (0-1) */
  compressQuality?: number;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  multiple: false,
  maxSize: 5,
  maxWidth: 1920,
  maxHeight: 1920,
  showEdit: true,
  uploadUrl: '/files/upload',
  enableCompress: true,
  compressQuality: 0.8,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'upload', file: File): void;
  (e: 'change', file: File): void;
  (e: 'progress', progress: number): void;
}>();

// ============ 状态 ============
const fileInputRef = ref<HTMLInputElement | null>(null);
const isDragOver = ref(false);
const uploading = ref(false);
const uploadProgress = ref(0);
const currentFile = ref<File | null>(null);
const previewUrl = ref(props.modelValue || '');
const compressedSize = ref<number | null>(null);

// 裁剪相关状态
const showCropDialog = ref(false);
const showPreviewDialog = ref(false);
const cropSourceUrl = ref('');
const cropScale = ref(1);
const cropRotate = ref(0);

// 支持的图片类型
const acceptTypes = 'image/jpeg,image/png,image/webp,image/gif';

// 子组件文本
const clickOrDragText = t('uploader.clickOrDrag');
const supportedFormatsText = t('uploader.supportedFormats');
const compressedText = t('uploader.compressed');
const uploadingText = t('uploader.uploading');

// ============ 方法 ============

/** 触发上传 */
function triggerUpload() {
  if (!previewUrl.value) {
    fileInputRef.value?.click();
  } else if (props.showEdit) {
    openCropDialog();
  }
}

/** 处理文件选择 */
function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (file) {
    void processFile(file);
  }
  target.value = '';
}

/** 处理拖拽文件 */
function handleDrop(event: DragEvent) {
  isDragOver.value = false;
  const file = event.dataTransfer?.files?.[0];
  if (file && file.type.startsWith('image/')) {
    void processFile(file);
  } else {
    $q.notify({
      type: 'warning',
      message: t('uploader.imageOnly'),
    });
  }
}

/** 处理文件 */
async function processFile(file: File) {
  if (!file.type.startsWith('image/')) {
    $q.notify({
      type: 'warning',
      message: t('uploader.imageOnly') || '请上传图片文件',
    });
    return;
  }
  const maxSizeBytes = props.maxSize * 1024 * 1024;
  if (file.size > maxSizeBytes) {
    $q.notify({
      type: 'negative',
      message: `${file.name}: ${t('uploader.fileTooLarge')} (${props.maxSize}MB)`,
    });
    return;
  }
  currentFile.value = file;
  previewUrl.value = URL.createObjectURL(file);
  await uploadImage(file);
}

/** 压缩并上传图片 */
async function uploadImage(file: File) {
  uploading.value = true;
  uploadProgress.value = 0;
  try {
    let fileToUpload = file;
    if (props.enableCompress) {
      fileToUpload = await compressImage(file);
      if (fileToUpload !== file) {
        compressedSize.value = fileToUpload.size;
      }
    }
    const formData = new FormData();
    formData.append('file', fileToUpload);
    const response = await httpClient.post(props.uploadUrl, formData, {
      onUploadProgress: (progressEvent: { loaded: number; total?: number }) => {
        if (progressEvent.total) {
          uploadProgress.value = Math.round((progressEvent.loaded / progressEvent.total) * 100);
          emit('progress', uploadProgress.value);
        }
      },
    }) as { data?: { url?: string; data?: { url?: string } } };
    const uploadedUrl = response.data?.data?.url || response.data?.url || previewUrl.value;
    previewUrl.value = uploadedUrl;
    emit('update:modelValue', uploadedUrl);
    emit('upload', file);
    emit('change', file);
    $q.notify({
      type: 'positive',
      message: t('uploader.uploadSuccess'),
    });
  } catch (error) {
    logger.error('【图片上传失败】', error);
    $q.notify({
      type: 'negative',
      message: t('uploader.uploadFailed'),
    });
  } finally {
    uploading.value = false;
  }
}

/** 压缩图片 */
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
      canvas.toBlob(
        (blob) => {
          if (blob) {
            const compressedFile = new File([blob], file.name, { type: 'image/jpeg' });
            logger.info(`【图片压缩】${file.name}: ${formatFileSize(file.size)} -> ${formatFileSize(blob.size)}`);
            URL.revokeObjectURL(img.src);
            resolve(compressedFile);
          } else {
            resolve(file);
          }
        },
        'image/jpeg',
        props.compressQuality,
      );
    };
    img.onerror = () => {
      URL.revokeObjectURL(img.src);
      resolve(file);
    };
    img.src = URL.createObjectURL(file);
  });
}

/** 打开裁剪弹窗 */
function openCropDialog() {
  cropSourceUrl.value = previewUrl.value;
  cropScale.value = 1;
  cropRotate.value = 0;
  showCropDialog.value = true;
}

/** 应用裁剪 */
function applyCrop() {
  if (!currentFile.value) {
    showCropDialog.value = false;
    return;
  }
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');
  canvas.width = 1000;
  canvas.height = 1000;
  const img = new Image();
  img.src = cropSourceUrl.value;
  let resolved = false;
  const done = () => { if (!resolved) resolved = true; };
  img.onload = () => {
    ctx?.save();
    ctx?.translate(canvas.width / 2, canvas.height / 2);
    ctx?.rotate((cropRotate.value * Math.PI) / 180);
    ctx?.scale(cropScale.value, cropScale.value);
    ctx?.drawImage(img, -1000 / 2, -1000 / 2, 1000, 1000);
    ctx?.restore();
    done();
  };
  img.onerror = () => done();
  canvas.toBlob(
    (blob) => {
      if (blob) {
        const croppedFile = new File([blob], currentFile.value?.name ?? 'cropped.jpg', {
          type: 'image/jpeg',
        });
        const newUrl = URL.createObjectURL(blob);
        previewUrl.value = newUrl;
        void uploadImage(croppedFile);
      }
      showCropDialog.value = false;
    },
    'image/jpeg',
    props.compressQuality,
  );
}

/** 移除图片 */
function removeImage() {
  previewUrl.value = '';
  currentFile.value = null;
  compressedSize.value = null;
  emit('update:modelValue', '');
}

/** 格式化文件大小 */
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}
</script>

<style scoped>
.image-uploader {
  width: 100%;
}

.hidden-input {
  display: none;
}
</style>
