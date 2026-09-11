/**
 * @file FieldMaskPreviewDialog.vue
 * @description 字段掩码预览对话框
 * @date 2026-08-22
 */

<template>
  <q-dialog v-model="visible">
    <q-card style="min-width: 500px">
      <q-card-section>
        <div class="text-h6">字段掩码预览</div>
      </q-card-section>

      <q-card-section class="q-pt-none">
        <div v-if="previewRule" class="q-mb-md">
          <div class="text-subtitle2 q-mb-xs">字段名: {{ previewRule.fieldName }}</div>
          <div class="text-subtitle2 q-mb-xs">策略: {{ previewRule.strategy }}</div>
          <div class="text-subtitle">原始值: {{ previewRule.sampleValue }}</div>
        </div>

        <div v-if="previewRule" class="q-mt-md">
          <div class="text-subtitle2 q-mb-xs">掩码后值:</div>
          <div class="text-h6">{{ maskedValue }}</div>
        </div>
      </q-card-section>

      <q-card-actions align="right">
        <q-btn flat label="关闭" color="primary" v-close-popup />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { MaskStrategy } from '@/types/fieldMask';

export interface LocalFieldMaskItem {
  fieldName?: string;
  strategy?: MaskStrategy;
  sampleValue?: string;
}

interface Props {
  modelValue: boolean;
  previewRule: LocalFieldMaskItem | null;
  strategy: MaskStrategy;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  previewRule: null,
  strategy: MaskStrategy.HIDE,
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'close': [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val),
});

const maskedValue = computed(() => {
  if (!props.previewRule || !props.previewRule.sampleValue) {
    return '';
  }
  const value = props.previewRule.sampleValue;
  switch (props.previewRule.strategy) {
    case MaskStrategy.HIDE:
      return '****';
    case MaskStrategy.REVEAL:
      return value;
    case MaskStrategy.HASH:
      return '***HASH***';
    default:
      return value;
  }
});
</script>

<style scoped>
/* No custom styles needed */
</style>
