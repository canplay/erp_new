/**
 * @file ImageUploader/FilePreview.vue
 * @description 文件信息预览子组件
 */

<template>
  <div class="image-info q-mt-sm">
    <div class="text-caption text-grey-6">
      {{ file.name }} · {{ formatFileSize(file.size) }}
      <span v-if="compressedSize" class="text-positive q-ml-sm">
        ({{ compressedLabel }}: {{ formatFileSize(compressedSize) }})
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  file: File;
  compressedSize?: number | null;
  compressedLabel?: string;
}

withDefaults(defineProps<Props>(), {
  compressedSize: null,
  compressedLabel: '压缩后',
});

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}
</script>

<style scoped>
.image-info {
  word-break: break-all;
}
</style>
