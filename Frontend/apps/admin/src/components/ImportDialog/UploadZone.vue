/**
 * @file UploadZone.vue
 * @description 文件上传区域 - 拖拽/点击上传
 * @date 2026-04-04
 */

<template>
  <q-card flat bordered class="upload-zone" @click="$emit('fileSelected')" @dragover.prevent @dragleave="onDragLeave" @drop.prevent="onDrop">
    <q-card-section class="text-center q-py-xl">
      <q-icon name="cloud_upload" size="64px" color="grey-5" />
      <div class="text-h6 q-mt-md">{{ $t('importMod.dragOrClick') }}</div>
      <div class="text-caption text-grey">
        {{ $t('importMod.supportedFormats') }}: .xlsx, .xls, .csv
      </div>
      <div class="text-caption text-grey">
        {{ $t('importMod.maxFileSize') }}: {{ maxFileSize }}MB
      </div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
const emit = defineEmits<{
  (e: 'fileSelected'): void;
  (e: 'fileDrop', file: File): void;
}>();

const maxFileSize = 10;

function onDragLeave(e: DragEvent) {
  (e.currentTarget as HTMLElement).classList.remove('drag-over');
}

function onDrop(e: DragEvent) {
  (e.currentTarget as HTMLElement).classList.remove('drag-over');
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    emit('fileDrop', files[0]!);
  }
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
</style>
