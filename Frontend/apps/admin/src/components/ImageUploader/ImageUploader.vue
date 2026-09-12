/**
 * @file ImageUploader.vue
 * @description 图片上传组件 - 支持压缩、裁剪、预览
 * @date 2026-04-03
 * @refactored 2026-09-12 - Split into composables and sub-components
 */

<template>
  <div class="image-uploader">
    <!-- Upload Zone -->
    <UploadZone
      :preview-url="previewUrl"
      :show-edit="showEdit"
      @click="triggerUpload"
      @drop="handleDrop"
      @remove="removeImage"
      @edit="openCropDialog"
      @preview="showPreviewDialog = true"
    />

    <!-- Hidden file input -->
    <input
      ref="fileInputRef"
      type="file"
      :accept="acceptTypes"
      class="hidden-input"
      @change="handleFileChange"
    />

    <!-- Image info -->
    <div v-if="currentFile" class="image-info q-mt-sm">
      <div class="text-caption text-grey-6">
        {{ currentFile.name }} · {{ formatFileSize(currentFile.size) }}
        <span v-if="compressedSize" class="text-positive q-ml-sm">
          ({{ $t('uploader.compressed') }}: {{ formatFileSize(compressedSize) }})
        </span>
      </div>
    </div>

    <!-- Upload progress -->
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

    <!-- Crop dialog -->
    <ImageCropDialog
      v-model="showCropDialog"
      :source-url="cropSourceUrl"
      :scale="cropScale"
      :rotate="cropRotate"
      @update:scale="cropScale = $event"
      @update:rotate="cropRotate = $event"
      @apply="applyCrop"
    />

    <!-- Preview dialog -->
    <ImagePreviewDialog
      v-model="showPreviewDialog"
      :image-url="previewUrl"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import ImageCropDialog from './ImageCropDialog.vue';
import ImagePreviewDialog from './ImagePreviewDialog.vue';
import UploadZone from './UploadZone.vue';
import { useImageUpload } from './composables/useImageUpload';

const { t } = useI18n();
const $q = useQuasar();

// ============ Props & Emits ============

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

// ============ Composable ============

const {
  uploading,
  uploadProgress,
  compressedSize,
  formatFileSize,
  uploadImage,
  validateFile,
  notifyUploadResult,
} = useImageUpload({
  uploadUrl: props.uploadUrl,
  maxSize: props.maxSize,
  maxWidth: props.maxWidth,
  maxHeight: props.maxHeight,
  enableCompress: props.enableCompress,
  compressQuality: props.compressQuality,
});

// ============ State ============

const fileInputRef = ref<HTMLInputElement | null>(null);
const currentFile = ref<File | null>(null);
const previewUrl = ref(props.modelValue || '');

// Crop state
const showCropDialog = ref(false);
const showPreviewDialog = ref(false);
const cropSourceUrl = ref('');
const cropScale = ref(1);
const cropRotate = ref(0);

const acceptTypes = 'image/jpeg,image/png,image/webp,image/gif';

// ============ Methods ============

function triggerUpload() {
  if (!previewUrl.value) {
    fileInputRef.value?.click();
  } else if (props.showEdit) {
    openCropDialog();
  }
}

function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (file) {
    void processFile(file);
  }
  target.value = '';
}

function handleDrop(file: File) {
  if (file.type.startsWith('image/')) {
    void processFile(file);
  } else {
    $q.notify({
      type: 'warning',
      message: t('uploader.imageOnly'),
    });
  }
}

async function processFile(file: File) {
  const validation = validateFile(file);
  if (!validation.valid) {
    $q.notify({
      type: 'negative',
      message: validation.message,
    });
    return;
  }

  currentFile.value = file;
  previewUrl.value = URL.createObjectURL(file);

  try {
    const uploadedUrl = await uploadImage(file, (progress) => {
      emit('progress', progress);
    });
    if (uploadedUrl) {
      previewUrl.value = uploadedUrl;
      emit('update:modelValue', uploadedUrl);
      emit('upload', file);
      emit('change', file);
      notifyUploadResult(true);
    }
  } catch {
    notifyUploadResult(false);
  }
}

function openCropDialog() {
  cropSourceUrl.value = previewUrl.value;
  cropScale.value = 1;
  cropRotate.value = 0;
  showCropDialog.value = true;
}

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
    props.compressQuality
  );
}

function removeImage() {
  previewUrl.value = '';
  currentFile.value = null;
  compressedSize.value = null;
  emit('update:modelValue', '');
}
</script>

<style scoped>
.image-uploader {
  width: 100%;
}

.hidden-input {
  display: none;
}

.image-info {
  word-break: break-all;
}
</style>
