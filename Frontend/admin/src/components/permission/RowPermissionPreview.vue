/**
 * @file RowPermissionPreview.vue
 * @description 行级权限过滤条件预览组件
 * @date 2026-04-04
 */

<template>
  <div class="filter-preview">
    <template v-for="(rule, idx) in rules" :key="idx">
      <q-chip
        v-if="idx > 0"
        dense
        :color="logic === 'and' ? 'primary' : 'secondary'"
        text-color="white"
        size="sm"
      >
        {{ logic === 'and' ? '且' : '或' }}
      </q-chip>
      <q-badge color="grey-5" text-color="dark" class="q-pa-xs">
        {{ getFieldLabel(rule.field) }} {{ getOperatorLabel(rule.operator) }} {{ formatValue(rule) }}
      </q-badge>
    </template>
    <span v-if="!rules?.length" class="text-grey">无过滤条件</span>
  </div>
</template>

<script setup lang="ts">
import { FilterOperator, OPERATOR_OPTIONS, type FilterRule } from '@/types/rowPermission';

interface Props {
  rules: FilterRule[];
  logic: string;
  availableFields: Array<{ field: string; label: string }>;
}

const props = defineProps<Props>();

function getFieldLabel(field: string): string {
  const config = props.availableFields.find(f => f.field === field);
  return config?.label || field;
}

function getOperatorLabel(operator: string): string {
  const opt = OPERATOR_OPTIONS.find(o => o.value === operator);
  return opt?.label || operator;
}

function formatValue(rule: FilterRule): string {
  const val = rule.value;
  if (Array.isArray(val)) return `(${val.join(', ')})`;
  if (val === null || val === undefined) return '';
  if (rule.operator === FilterOperator.BETWEEN) {
    return `${val ?? ''} ~ ${rule.value2 ?? ''}`;
  }
  return String(val);
}
</script>

<style scoped>
.filter-preview {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}
</style>
