/**
 * @file AdvancedSearch/SearchForm.vue
 * @description 高级搜索 - 搜索条件表单（字段/操作符/值输入）
 * @date 2026-08-22
 */

<template>
  <div class="search-form">
    <!-- 搜索条件 -->
    <div class="search-conditions">
      <div
        v-for="(condition, index) in conditions"
        :key="index"
        class="condition-row"
      >
        <!-- 字段选择 -->
        <q-select
          v-model="condition.field"
          :options="fieldOptions"
          dense
          outlined
          emit-value
          map-options
          class="field-select"
          :placeholder="'选择字段'"
          @update:model-value="onFieldChange(condition)"
        />

        <!-- 操作符选择 -->
        <q-select
          v-model="condition.operator"
          :options="getOperatorsForField(condition.field)"
          dense
          outlined
          emit-value
          map-options
          class="operator-select"
        />

        <!-- 值输入 -->
        <component
          :is="getInputComponent(condition)"
          v-model="condition.value"
          :type="getInputType(condition)"
          :options="getOptionsForField(condition.field)"
          dense
          outlined
          class="value-input"
          :placeholder="placeholder"
        />

        <!-- 删除按钮 -->
        <q-btn
          flat
          round
          dense
          icon="close"
          color="grey"
          @click="removeCondition(index)"
        />
      </div>

      <!-- 添加条件按钮 -->
      <q-btn
        flat
        dense
        icon="add"
        :label="addConditionLabel"
        color="primary"
        class="add-condition-btn"
        @click="addCondition"
      />
    </div>

    <!-- 搜索操作 -->
    <div class="search-actions q-mt-md">
      <q-btn
        color="primary"
        icon="search"
        :label="searchLabel"
        @click="handleSearch"
      />
      <q-btn
        flat
        color="grey"
        :label="resetLabel"
        @click="handleReset"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useQuasar } from 'quasar';

interface FieldOption {
  label: string;
  value: string;
  type: string;
  options?: Array<{ label: string; value: string }>;
}

interface SearchCondition {
  field: string;
  operator: string;
  value: unknown;
}

interface Props {
  fieldOptions: FieldOption[];
  initialConditions?: SearchCondition[];
  placeholder?: string;
  addConditionLabel?: string;
  searchLabel?: string;
  resetLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  fieldOptions: () => [],
  initialConditions: () => [],
  placeholder: '输入值',
  addConditionLabel: '添加条件',
  searchLabel: '搜索',
  resetLabel: '重置',
});

const emit = defineEmits<{
  search: [conditions: SearchCondition[]];
  reset: [];
}>();

const $q = useQuasar();

const conditions = ref<SearchCondition[]>(
  props.initialConditions?.length ? [...props.initialConditions] : []
);

const operatorMap: Record<string, Array<{ label: string; value: string }>> = {
  string: [
    { label: '等于', value: 'eq' },
    { label: '包含', value: 'contains' },
    { label: '开头是', value: 'starts_with' },
    { label: '结尾是', value: 'ends_with' },
    { label: '不等于', value: 'ne' },
    { label: '为空', value: 'is_null' },
    { label: '不为空', value: 'is_not_null' }
  ],
  number: [
    { label: '等于', value: 'eq' },
    { label: '不等于', value: 'ne' },
    { label: '大于', value: 'gt' },
    { label: '大于等于', value: 'gte' },
    { label: '小于', value: 'lt' },
    { label: '小于等于', value: 'lte' },
    { label: '区间', value: 'between' }
  ],
  date: [
    { label: '等于', value: 'eq' },
    { label: '早于', value: 'lt' },
    { label: '晚于', value: 'gt' },
    { label: '区间', value: 'between' }
  ],
  select: [
    { label: '等于', value: 'eq' },
    { label: '不等于', value: 'ne' },
    { label: '包含', value: 'in' },
    { label: '为空', value: 'is_null' }
  ],
  boolean: [
    { label: '是', value: 'eq' },
    { label: '否', value: 'ne' }
  ]
};

function getOperatorsForField(field: string): Array<{ label: string; value: string }> {
  const fieldInfo = props.fieldOptions.find(f => f.value === field);
  const type = fieldInfo?.type || 'string';
  return (operatorMap[type] || operatorMap.string) ?? [];
}

function getInputComponent(condition: SearchCondition): string {
  const fieldInfo = props.fieldOptions.find(f => f.value === condition.field);
  const type = fieldInfo?.type || 'string';

  if (type === 'select' && fieldInfo?.options) {
    return 'q-select';
  }

  if (type === 'boolean') {
    return 'q-select';
  }

  return 'q-input';
}

function getInputType(condition: SearchCondition): string {
  const fieldInfo = props.fieldOptions.find(f => f.value === condition.field);
  const type = fieldInfo?.type || 'string';

  if (type === 'number') return 'number';
  if (type === 'date') return 'date';

  return 'text';
}

function getOptionsForField(field: string): Array<{ label: string; value: string }> | undefined {
  const fieldInfo = props.fieldOptions.find(f => f.value === field);
  return fieldInfo?.options;
}

function onFieldChange(condition: SearchCondition) {
  const operators = getOperatorsForField(condition.field);
  if (operators.length > 0 && !operators.find(o => o.value === condition.operator)) {
    condition.operator = operators[0]!.value;
  }
  condition.value = '';
}

function addCondition() {
  const firstField = props.fieldOptions[0];
  if (firstField) {
    const operators = getOperatorsForField(firstField.value);
    conditions.value.push({
      field: firstField.value,
      operator: operators[0]?.value || 'eq',
      value: ''
    });
  }
}

function removeCondition(index: number) {
  conditions.value.splice(index, 1);
}

function handleSearch() {
  emit('search', conditions.value);
}

function handleReset() {
  conditions.value = [];
  emit('reset');
}
</script>

<style scoped>
.search-conditions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.condition-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.field-select {
  min-width: 150px;
  flex: 0 0 150px;
}

.operator-select {
  min-width: 120px;
  flex: 0 0 120px;
}

.value-input {
  flex: 1;
}

.add-condition-btn {
  margin-top: 8px;
}
</style>
