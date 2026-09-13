/**
 * @file PreviewDialog.vue
 * @description 图片预览对话框
 * @date 2026-04-03
 */

<template>
  <q-dialog v-model="visible">
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
interface Props {
  visible: boolean;
  imageUrl: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
}>();

function close() {
  emit('update:visible', false);
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
