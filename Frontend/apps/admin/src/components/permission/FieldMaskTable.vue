// @ts-nocheck
/**
 * @file FieldMaskTable.vue
 * @description 字段掩码表格
 * @date 2026-08-22
 */

<template>
  <q-table
    :rows="filteredFieldMasks"
    :columns="columns"
    row-key="fieldName"
    dense
    flat
    bordered
  >
    <template v-slot:body-cell-preview="props">
      <q-td :props="props">
        <q-btn
          flat
          dense
          size="xs"
          icon="visibility"
          color="primary"
          @click="onPreview as any"
        />
      </q-td>
    </template>
  </q-table>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { MaskStrategy } from '@/types/fieldMask';

export interface LocalFieldMaskItem {
  fieldName?: string;
  strategy?: MaskStrategy;
  sampleValue?: string;
}

interface Props {
  fieldMasks: LocalFieldMaskItem[];
}

const props = withDefaults(defineProps<Props>(), {
  fieldMasks: () => [],
});

const emit = defineEmits<{
  onPreview?: (row: LocalFieldMaskItem) => void;
  onStrategyChange?: () => void;
  onRuleChange?: () => void;
}>();

const filteredFieldMasks = computed(() =>
  props.fieldMasks.map((item) => ({
    fieldName: item.fieldName,
    strategy: item.strategy,
    sampleValue: item.sampleValue,
  }))
);

const columns: { name: string; label: string; field: string; align?: string }[] = [
  { name: 'fieldName', label: '字段名', field: 'fieldName', align: 'left' },
  { name: 'strategy', label: '掩码策略', field: 'strategy', align: 'left' },
  { name: 'preview', label: '预览', field: 'preview', align: 'center' },
];

function onPreview(row: LocalFieldMaskItem) {
  emit('onPreview', row);
}
</script>

<style scoped>
/* No custom styles needed */
</style>
