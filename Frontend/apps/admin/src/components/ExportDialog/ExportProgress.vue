/**
 * @file ExportDialog/ExportProgress.vue
 * @description 导出进度组件 - 上传进度、成功/失败状态
 * @date 2026-08-22
 */

<template>
  <div class="export-progress">
    <!-- 上传进度 -->
    <q-card-section v-if="isUploading">
      <div class="q-mb-sm">
        <span>{{ $t('common.uploading') }}</span>
        <span class="float-right">{{ uploadProgress }}%</span>
      </div>
      <q-linear-progress :value="uploadProgress / 100" size="12px" color="primary" />
      <div v-if="uploadSpeed" class="text-caption text-grey q-mt-xs">
        速度: {{ formatSpeed(uploadSpeed) }}
      </div>
    </q-card-section>

    <!-- 上传成功 -->
    <q-card-section v-if="uploadSuccess">
      <q-banner type="positive" icon="check_circle" class="bg-green-1">
        {{ $t('common.uploadSuccess') }}
        <template #action>
          <q-btn flat color="primary" :label="$t('common.done')" @click="$emit('close')" />
        </template>
      </q-banner>
    </q-card-section>

    <!-- 上传失败 -->
    <q-card-section v-if="uploadError">
      <q-banner type="negative" class="bg-red-1">
        {{ uploadError }}
        <template #action>
          <q-btn flat color="negative" :label="$t('common.retry')" @click="$emit('retry')" />
          <q-btn flat color="grey" :label="$t('common.cancel')" @click="$emit('close')" />
        </template>
      </q-banner>
    </q-card-section>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()

interface Props {
  isUploading?: boolean;
  uploadProgress?: number;
  uploadSpeed?: number;
  uploadSuccess?: boolean;
  uploadError?: string;
}

withDefaults(defineProps<Props>(), {
  isUploading: false,
  uploadProgress: 0,
  uploadSpeed: 0,
  uploadSuccess: false,
  uploadError: '',
});

const emit = defineEmits<{
  close: [];
  retry: [];
}>();

function formatSpeed(bytesPerSecond: number): string {
  if (bytesPerSecond < 1024) return `${Math.round(bytesPerSecond)} B/s`;
  if (bytesPerSecond < 1024 * 1024) return `${(bytesPerSecond / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSecond / (1024 * 1024)).toFixed(1)} MB/s`;
}
</script>
