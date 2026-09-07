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
      <!-- 预览图 -->
      <img v-if="previewUrl" :src="previewUrl" class="preview-image" />

      <!-- 上传提示 -->
      <div v-else class="upload-hint">
        <q-icon name="add_photo_alternate" size="48px" color="grey-5" />
        <div class="text-grey-6 q-mt-sm">{{ $t('uploader.clickOrDrag') }}</div>
        <div class="text-caption text-grey-5 q-mt-xs">
          {{ $t('uploader.supportedFormats') }}
        </div>
      </div>

      <!-- 删除按钮 -->
      <q-btn
        v-if="previewUrl"
        class="delete-btn"
        round
        dense
        color="negative"
        icon="close"
        size="sm"
        @click.stop="removeImage"
      >
        <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
      </q-btn>

      <!-- 编辑按钮 -->
      <q-btn
        v-if="previewUrl && showEdit"
        class="edit-btn"
        round
        dense
        color="primary"
        icon="edit"
        size="sm"
        @click.stop="openCropDialog"
      >
        <q-tooltip>{{ $t('uploader.editImage') }}</q-tooltip>
      </q-btn>
    </div>

    <!-- 隐藏的文件输入框 -->
    <input
      ref="fileInputRef"
      type="file"
      :accept="acceptTypes"
      class="hidden-input"
      @change="handleFileChange"
    />

    <!-- 图片信息 -->
    <div v-if="currentFile" class="image-info q-mt-sm">
      <div class="text-caption text-grey-6">
        {{ currentFile.name }} · {{ formatFileSize(currentFile.size) }}
        <span v-if="compressedSize" class="text-positive q-ml-sm">
          ({{ $t('uploader.compressed') }}: {{ formatFileSize(compressedSize) }})
        </span>
      </div>
    </div>

    <!-- 上传进度 -->
    <div v-if="uploading" class="q-mt-sm">
      <q-linear-progress
        :value="uploadProgress / 100"
        color="primary"
        track-color="grey-3"
      />
      <div class="text-caption text-grey-6 q-mt-xs">
        {{ $t('uploader.uploading') }} {{ uploadProgress }}%
      </div>
    </div>

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
 * @file ImageUploader.vue
 * @description 图片上传组件 - 支持压缩、裁剪、预览
 * @date 2026-04-03
 */

import { ref } from 'vue';
import { logger } from '@/utils/logger';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { httpClient } from '@/utils/alova';
import ImageCropDialog from './ImageCropDialog.vue';
import ImagePreviewDialog from './ImagePreviewDialog.vue';

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

// ============ 方法 ============

/**
 * @brief 触发上传
 */
function triggerUpload() {
  if (!previewUrl.value) {
    fileInputRef.value?.click();
  } else if (props.showEdit) {
    openCropDialog();
  }
}

/**
 * @brief 处理文件选择
 */
function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];

  if (file) {
    void processFile(file);
  }

  target.value = '';
}

/**
 * @brief 处理拖拽文件
 */
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

/**
 * @brief 处理文件
 */
async function processFile(file: File) {
  // 验证文件类型
  if (!file.type.startsWith('image/')) {
    $q.notify({
      type: 'warning',
      message: t('uploader.imageOnly') || '请上传图片文件',
    });
    return;
  }

  // 验证文件大小
  const maxSizeBytes = props.maxSize * 1024 * 1024;
  if (file.size > maxSizeBytes) {
    $q.notify({
      type: 'negative',
      message: `${file.name}: ${t('uploader.fileTooLarge')} (${props.maxSize}MB)`,
    });
    return;
  }

  currentFile.value = file;

  // 生成预览
  previewUrl.value = URL.createObjectURL(file);

  // 自动上传
  await uploadImage(file);
}

/**
 * @brief 压缩并上传图片
 */
async function uploadImage(file: File) {
  uploading.value = true;
  uploadProgress.value = 0;

  try {
    // 1. 压缩图片
    let fileToUpload = file;
    if (props.enableCompress) {
      fileToUpload = await compressImage(file);
      if (fileToUpload !== file) {
        compressedSize.value = fileToUpload.size;
      }
    }

    // 2. 上传
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

    // 3. 更新状态
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

/**
 * @brief 压缩图片
 */
async function compressImage(file: File): Promise<File> {
  return new Promise((resolve) => {
    const img = new Image();
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');

    img.onload = () => {
      // 计算压缩后的尺寸
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
        props.compressQuality
      );
    };

    img.onerror = () => {
      URL.revokeObjectURL(img.src);
      resolve(file);
    };

    img.src = URL.createObjectURL(file);
  });
}

/**
 * @brief 打开裁剪弹窗
 */
function openCropDialog() {
  cropSourceUrl.value = previewUrl.value;
  cropScale.value = 1;
  cropRotate.value = 0;
  showCropDialog.value = true;
}

/**
 * @brief 应用裁剪
 */
function applyCrop() {
  if (!currentFile.value) {
    showCropDialog.value = false;
    return;
  }

  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');

  // 设置 canvas 尺寸（这里简单处理，实际可使用 cropperjs）
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

  // 转换为 File
  canvas.toBlob(
    (blob) => {
      if (blob) {
        const croppedFile = new File([blob], currentFile.value?.name ?? 'cropped.jpg', {
          type: 'image/jpeg',
        });

        // 更新预览
        const newUrl = URL.createObjectURL(blob);
        previewUrl.value = newUrl;

        // 重新上传
        void uploadImage(croppedFile);
      }
      showCropDialog.value = false;
    },
    'image/jpeg',
    props.compressQuality
  );
}

/**
 * @brief 移除图片
 */
function removeImage() {
  previewUrl.value = '';
  currentFile.value = null;
  compressedSize.value = null;
  emit('update:modelValue', '');
}

/**
 * @brief 格式化文件大小
 */
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

.upload-zone {
  position: relative;
  width: 100%;
  min-height: 150px;
  border: 2px dashed #ddd;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s ease;
  background-color: #fafafa;
}

.upload-zone:hover {
  border-color: #1976d2;
}

.upload-zone.drag-over {
  border-color: #1976d2;
  background-color: #e3f2fd;
}

.upload-zone.has-image {
  border-style: solid;
  border-color: #ddd;
  background-color: transparent;
}

.preview-image {
  max-width: 100%;
  max-height: 200px;
  object-fit: contain;
}

.upload-hint {
  text-align: center;
  color: #757575;
}

.delete-btn,
.edit-btn {
  position: absolute;
  top: 8px;
  z-index: 10;
}

.delete-btn {
  right: 8px;
}

.edit-btn {
  right: 40px;
}

.image-info {
  word-break: break-all;
}
</style>
