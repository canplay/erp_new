/**
 * @file DataFilterBuilder/FilterGroup.vue
 * @description 过滤组 - 规则组管理
 * @date 2026-04-04
 */

<template>
  <div class="filter-group">
    <div class="text-subtitle2 q-mb-sm">过滤组</div>
    <div class="q-mb-sm">
      <q-btn-toggle
        v-model="localLogic"
        :options="logicOptions"
        toggle-color="primary"
        no-caps
      />
    </div>
    <div v-for="(rule, idx) in rules" :key="idx" class="q-mb-sm">
      <FilterCondition
        :index="idx"
        :rule="rule"
        :field-options="fieldOptions"
        :operator-options="operatorOptions"
        @update:model-value="(val: FilterRule) => updateRule(idx, val)"
        @remove="removeRule(idx)"
      />
    </div>
    <q-btn
      flat
      dense
      color="primary"
      icon="add"
      :label="$t('permission.addCondition')"
      @click="addRule"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { FilterRule, FilterGroup } from '@/types/rowPermission';
import { FilterOperator, LogicalOperator, OPERATOR_OPTIONS } from '@/types/rowPermission';
import FilterCondition from './FilterCondition.vue';

const { t } = useI18n();

const props = withDefaults(defineProps<{
  rules: FilterRule[];
  currentLogic: LogicalOperator;
  availableFields?: Array<{
    field: string;
    label: string;
    type: 'string' | 'number' | 'date' | 'boolean' | 'select';
    enumOptions?: Array<{ label: string; value: unknown }>;
  }>;
  logicOptions: { label: string; value: string }[];
  fieldOptions: { label: string; value: string }[];
  operatorOptions: { label: string; value: string }[];
}>(), {
  rules: () => [],
  currentLogic: LogicalOperator.AND,
  logicOptions: () => [],
  fieldOptions: () => [],
  operatorOptions: () => [],
});

const emit = defineEmits<{
  'update:modelValue': [value: FilterGroup];
  saved: [value: FilterGroup];
}>();

const localRules = computed({
  get: () => props.rules,
  set: (val: FilterRule[]) => emit('update:modelValue', { rules: val, logic: props.currentLogic }),
});

const localLogic = computed({
  get: () => props.currentLogic,
  set: (val: LogicalOperator) => emit('update:modelValue', { rules: props.rules, logic: val }),
});

function updateRule(index: number, rule: FilterRule) {
  const newRules = [...props.rules];
  newRules[index] = rule;
  emit('update:modelValue', { rules: newRules, logic: props.currentLogic });
}

function removeRule(index: number) {
  const newRules = props.rules.filter((_, i) => i !== index);
  emit('update:modelValue', { rules: newRules, logic: props.currentLogic });
}

function addRule() {
  const newRule: FilterRule = {
    field: '',
    operator: FilterOperator.EQ,
    value: '',
  };
  emit('update:modelValue', { rules: [...props.rules, newRule], logic: props.currentLogic });
}
</script>

<style scoped>
.filter-group {
  margin-bottom: 16px;
}
</style>
