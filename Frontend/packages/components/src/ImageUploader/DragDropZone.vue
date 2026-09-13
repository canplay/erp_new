/**
 * @file ImageUploader/DragDropZone.vue
 * @description 拖拽上传区域子组件
 */

<template>
  <div
    class="upload-zone"
    :class="{ 'drag-over': isDragOver, 'has-image': previewUrl }"
    @click="$emit('triggerUpload')"
    @dragover.prevent="$emit('update:isDragOver', true)"
    @dragleave.prevent="$emit('update:isDragOver', false)"
    @drop.prevent="$emit('drop', $event)"
  >
    <!-- 预览图 -->
    <img v-if="previewUrl" :src="previewUrl" class="preview-image" />

    <!-- 上传提示 -->
    <div v-else class="upload-hint">
      <q-icon name="add_photo_alternate" size="48px" color="grey-5" />
      <div class="text-grey-6 q-mt-sm">{{ clickOrDragText }}</div>
      <div class="text-caption text-grey-5 q-mt-xs">{{ supportedFormatsText }}</div>
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
      @click.stop="$emit('remove')"
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
      @click.stop="$emit('openCrop')"
    >
      <q-tooltip>{{ $t('uploader.editImage') }}</q-tooltip>
    </q-btn>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

interface Props {
  previewUrl: string;
  isDragOver: boolean;
  showEdit?: boolean;
  clickOrDragText: string;
  supportedFormatsText: string;
}

withDefaults(defineProps<Props>(), {
  showEdit: true,
});

defineEmits<{
  (e: 'triggerUpload'): void;
  (e: 'update:isDragOver', value: boolean): void;
  (e: 'drop', event: DragEvent): void;
  (e: 'remove'): void;
  (e: 'openCrop'): void;
}>();

// Expose tooltip function for parent
defineExpose({ t });
</script>

<style scoped>
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
</style>
