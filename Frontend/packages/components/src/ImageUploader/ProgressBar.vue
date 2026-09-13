/**
 * @file ImageUploader/ProgressBar.vue
 * @description 上传进度条子组件
 */

<template>
  <div class="q-mt-sm">
    <q-linear-progress
      :value="progress / 100"
      color="primary"
      track-color="grey-3"
    />
    <div class="text-caption text-grey-6 q-mt-xs">
      {{ uploadingText }} {{ progress }}%
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  progress: number;
  uploadingText?: string;
}

withDefaults(defineProps<Props>(), {
  uploadingText: '上传中',
});
</script>
