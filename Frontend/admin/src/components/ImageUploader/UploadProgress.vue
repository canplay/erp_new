/**
 * @file UploadProgress.vue
 * @description 上传进度指示器
 * @date 2026-04-03
 */

<template>
  <div class="upload-progress">
    <q-linear-progress
      :value="progress / 100"
      color="primary"
      track-color="grey-3"
    />
    <div class="text-caption text-grey-6 q-mt-xs">
      {{ $t('uploader.uploading') }} {{ progress }}%
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  progress: number;
}

defineProps<Props>();
</script>
