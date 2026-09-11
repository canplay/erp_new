/**
 * @file RowConditionEditor.vue
 * @description 行条件编辑器 - 规则添加/删除/编辑
 * @date 2026-04-04
 */

<template>
  <div class="condition-editor">
    <div class="text-subtitle2 q-mb-sm">过滤条件</div>
    <div v-for="(rule, idx) in localRules" :key="idx" class="row q-mb-sm items-center q-gutter-x-sm">
      <q-select v-model="rule.field" :options="availableFields.map(f => ({label: f.label, value: f.field}))" dense outlined
        style="min-width: 150px" emit-value map-options />
      <q-select v-model="rule.operator" :options="OPERATOR_OPTIONS" dense outlined
        style="min-width: 130px" emit-value map-options />
      <q-input v-if="rule.operator === 'between'" v-model.number="(rule.value2 as number | undefined)" dense outlined
        placeholder="至" style="min-width: 100px" />
      <q-input v-else v-model="String(rule.value)" dense outlined style="min-width: 150px" />
      <q-btn flat dense color="negative" icon="delete" @click="removeRule(idx)" />
    </div>
    <q-btn flat dense color="primary" icon="add" label="添加条件" @click="addRule" />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { FilterRule, FilterGroup } from '@/types/rowPermission';
import { FilterOperator, LogicalOperator, OPERATOR_OPTIONS } from '@/types/rowPermission';

interface Props {
  modelValue: FilterGroup;
  availableFields: Array<{ field: string; label: string; type: string; enumOptions?: Array<{ label: string; value: unknown }> }>;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'update:modelValue', value: FilterGroup): void;
  (e: 'saved', filterGroup: FilterGroup): void;
}>();

const localRules = ref<FilterRule[]>(props.modelValue.rules || []);
const localLogic = ref<LogicalOperator>(props.modelValue.logic || LogicalOperator.AND);

watch(localRules, () => emit('update:modelValue', { rules: [...localRules.value], logic: localLogic.value }), { deep: true });
watch(localLogic, () => emit('update:modelValue', { rules: [...localRules.value], logic: localLogic.value }));

function removeRule(index: number) {
  localRules.value.splice(index, 1);
}

function addRule() {
  localRules.value.push({ field: '', operator: FilterOperator.EQ, value: '', logic: LogicalOperator.AND });
}
</script>

<style scoped>
.condition-editor { width: 100%; }
</style>
