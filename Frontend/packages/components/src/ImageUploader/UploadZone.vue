/**
 * @file UploadZone.vue
 * @description Upload zone sub-component with drag-and-drop support
 * @date 2026-09-12
 */

<template>
  <div
    class="upload-zone"
    :class="{ 'drag-over': isDragOver, 'has-image': previewUrl }"
    @click="handleClick"
    @dragover.prevent="isDragOver = true"
    @dragleave.prevent="isDragOver = false"
    @drop.prevent="handleDrop"
  >
    <!-- Preview image -->
    <img v-if="previewUrl" :src="previewUrl" class="preview-image" @click.stop="openPreview" />

    <!-- Upload hint -->
    <div v-else class="upload-hint">
      <q-icon name="add_photo_alternate" size="48px" color="grey-5" />
      <div class="text-grey-6 q-mt-sm">{{ $t('uploader.clickOrDrag') }}</div>
      <div class="text-caption text-grey-5 q-mt-xs">
        {{ $t('uploader.supportedFormats') }}
      </div>
    </div>

    <!-- Delete button -->
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

    <!-- Edit button -->
    <q-btn
      v-if="previewUrl && showEdit"
      class="edit-btn"
      round
      dense
      color="primary"
      icon="edit"
      size="sm"
      @click.stop="$emit('edit')"
    >
      <q-tooltip>{{ $t('uploader.editImage') }}</q-tooltip>
    </q-btn>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

defineProps<{
  previewUrl?: string;
  showEdit?: boolean;
}>();

const emit = defineEmits<{
  click: [];
  drop: [file: File];
  remove: [];
  edit: [];
  preview: [];
}>();

const { t } = useI18n();
const isDragOver = ref(false);

function handleClick() {
  emit('click');
}

function handleDrop(event: DragEvent) {
  isDragOver.value = false;
  const file = event.dataTransfer?.files?.[0];
  if (file) {
    emit('drop', file);
  }
}

function openPreview() {
  emit('preview');
}
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
