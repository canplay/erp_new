/**
 * @file ImportResult.vue
 * @description 导入结果展示
 * @date 2026-04-04
 */

<template>
  <div class="text-center q-pa-xl">
    <q-icon name="check_circle" size="64px" color="positive" />
    <div class="text-h5 q-mt-md">{{ $t('importMod.success') }}</div>
    <div class="text-subtitle q-mt-sm">
      {{ $t('importMod.imported') }}: {{ result?.successCount || 0 }} |
      {{ $t('importMod.failed') }}: {{ result?.failCount || 0 }}
    </div>
  </div>
</template>

<script setup lang="ts">
interface ImportResult {
  successCount: number;
  failCount: number;
}

interface Props {
  result: ImportResult | null;
}

defineProps<Props>();
</script>
