<template>
  <div class="image-uploader">
    <div class="text-h6 q-mb-md">{{ $t('image.upload') }}</div>

    <div class="upload-area" @click="triggerFileInput">
      <q-icon name="cloud_upload" size="48px" color="primary" />
      <div class="text-subtitle1 q-mt-sm">{{ $t('image.dragOrClick') }}</div>
      <div class="text-caption text-grey-6">{{ $t('image.supportedFormats') }}</div>
    </div>

    <input
      ref="fileInput"
      type="file"
      accept="image/*"
      multiple
      class="file-input"
      @change="handleFileSelect"
    />

    <div v-if="isUploading" class="upload-progress q-mt-md">
      <q-linear-progress :value="uploadProgress" color="primary" />
      <div class="text-caption q-mt-xs">{{ $t('image.uploading') }} {{ uploadProgress }}%</div>
    </div>

    <div v-if="images.length > 0" class="image-list q-mt-md">
      <div v-for="(image, index) in images" :key="index" class="image-item">
        <q-img :src="image.url" class="image-item__thumbnail" @click="openPreview(index)" />
        <div class="image-item__info">
          <div class="text-subtitle2">{{ image.name }}</div>
          <div class="text-caption text-grey-6">{{ formatFileSize(image.size) }}</div>
        </div>
        <q-btn flat round dense icon="delete" color="negative" @click="removeImage(index)" />
      </div>
    </div>

    <div class="row q-mt-md q-gutter-sm">
      <q-btn color="primary" :label="$t('image.upload')" :disable="images.length === 0 || isUploading" @click="uploadImages" />
      <q-btn flat color="grey" :label="$t('image.clear')" @click="clearAll" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useImageUploader } from './useImageUploader'

const { t: $t } = useI18n()

interface ImageItem {
  file: File
  url: string
  name: string
  size: number
}

const props = defineProps<{
  maxSize?: number
  maxCount?: number
}>()

const emit = defineEmits<{
  (e: 'upload', images: ImageItem[]): void
  (e: 'update:images', images: ImageItem[]): void
  (e: 'error', message: string): void
}>()

const {
  fileInput,
  images,
  isUploading,
  uploadProgress,
  triggerFileInput,
  handleFileSelect,
  formatFileSize,
  removeImage,
  uploadImages,
  clearAll,
  openPreview
} = useImageUploader({
  maxSize: props.maxSize,
  maxCount: props.maxCount,
  onUpload: emit,
  onError: emit
})

const imageList = computed(() => images.value)
</script>

<style scoped>
.image-uploader {
  max-width: 600px;
}

.upload-area {
  border: 2px dashed #ccc;
  border-radius: 8px;
  padding: 32px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.3s;
}

.upload-area:hover {
  border-color: #1976d2;
}

.file-input {
  display: none;
}

.upload-progress {
  margin-top: 16px;
}

.image-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.image-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 8px;
}

.image-item__thumbnail {
  width: 60px;
  height: 60px;
  border-radius: 4px;
  cursor: pointer;
}

.image-item__info {
  flex: 1;
}
</style>
