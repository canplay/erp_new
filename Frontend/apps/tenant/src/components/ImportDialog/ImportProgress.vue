/**
 * @file ImportProgress.vue
 * @description 导入进度展示组件
 * @date 2026-04-04
 */

<template>
  <div class="import-progress q-pa-lg">
    <div class="text-subtitle1 q-mb-md">{{ title }}</div>
    <q-linear-progress :value="progress" color="primary" class="q-mb-sm" />
    <div class="text-caption text-grey">{{ statusMessage }}</div>
    <div v-if="stats" class="text-caption q-mt-sm">
      已处理: {{ stats.processed }}/{{ stats.total }} |
      成功: {{ stats.success }} |
      失败: {{ stats.failed }}
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title?: string;
  progress: number;
  statusMessage: string;
  stats?: { processed: number; total: number; success: number; failed: number };
}

defineProps<Props>();
</script>

<style scoped>
.import-progress { width: 100%; }
</style>
