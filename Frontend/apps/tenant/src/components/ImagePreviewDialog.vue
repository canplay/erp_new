/**
 * @file ImagePreviewDialog.vue
 * @description 图片全屏预览弹窗组件
 * @date 2026-08-22
 */

<template>
  <q-dialog v-model="props.modelValue">
    <q-card class="preview-card">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ $t('uploader.preview') }}</div>
        <q-space />
        <q-btn flat round icon="close" @click="close" />
      </q-card-section>
      <q-card-section class="q-pa-none">
        <img :src="imageUrl" class="full-preview" />
      </q-card-section>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

interface Props {
  /** 是否显示 */
  modelValue: boolean;
  /** 图片 URL */
  imageUrl: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

function close() {
  emit('update:modelValue', false);
}
</script>

<style scoped>
.preview-card {
  max-width: 90vw;
}

.full-preview {
  max-width: 100%;
  max-height: 70vh;
  object-fit: contain;
}
</style>
