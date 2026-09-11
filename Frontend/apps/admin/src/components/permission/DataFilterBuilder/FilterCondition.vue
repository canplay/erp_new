/**
 * @file DataFilterBuilder/FilterCondition.vue
 * @description 过滤条件 - 单个条件编辑
 * @date 2026-04-04
 */

<template>
  <div class="filter-condition">
    <div class="text-subtitle2 q-mb-sm">条件 {{ index + 1 }}</div>
    <div class="row q-col-gutter-sm">
      <div class="col-4">
        <q-select
          v-model="localRule.field"
          :options="fieldOptions"
          emit-value
          map-options
          dense
          outlined
          :label="$t('permission.field')"
        />
      </div>
      <div class="col-4">
        <q-select
          v-model="localRule.operator"
          :options="operatorOptions"
          emit-value
          map-options
          dense
          outlined
          :label="$t('permission.operator')"
        />
      </div>
      <div class="col-4">
        <q-input
          v-if="requiresValue2(localRule.operator)"
          v-model="String(localRule.value2)"
          dense
          outlined
          :label="$t('permission.value2')"
        />
        <q-input
          v-else
          v-model="String(localRule.value)"
          dense
          outlined
          :label="$t('permission.value')"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { FilterRule } from '@/types/rowPermission';
import { FilterOperator, OPERATOR_OPTIONS } from '@/types/rowPermission';

interface Props {
  index: number;
  rule: FilterRule;
  fieldOptions: { label: string; value: string }[];
  operatorOptions: { label: string; value: string }[];
}

const props = withDefaults(defineProps<Props>(), {
  index: 0,
  rule: () => ({ field: '', operator: FilterOperator.EQ, value: '' }),
  fieldOptions: () => [],
  operatorOptions: () => [],
});

const emit = defineEmits<{
  'update:modelValue': [value: FilterRule];
  'fieldChange': [];
  'remove': [index: number];
}>();

const localRule = ref<FilterRule>({ ...props.rule });

watch(localRule, () => emit('update:modelValue', { ...localRule.value }), { deep: true });

function requiresValue(op: string): boolean {
  return ![FilterOperator.IS_NULL, FilterOperator.IS_NOT_NULL].includes(op as FilterOperator);
}

function requiresValue2(op: string): boolean {
  return op === 'between';
}
</script>

<style scoped>
.filter-condition {
  margin-bottom: 8px;
}
</style>
