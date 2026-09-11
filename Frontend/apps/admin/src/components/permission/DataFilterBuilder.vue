/**
 * @file DataFilterBuilder.vue
 * @description 可视化数据过滤条件构建器 - 组合 FilterGroup + 权限预览
 * @date 2026-04-04
 */

<template>
  <div class="data-filter-builder">
    <q-card flat bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-sm">
          <q-icon name="filter_alt" class="q-mr-sm" color="primary" />
          {{ $t('permission.rowPermission') || '行级权限' }}
        </div>
        <div class="text-caption text-grey q-mb-md">
          {{ $t('permission.dataFilterDesc') }}
        </div>

        <!-- 条件组 -->
        <FilterGroup
          :rules="rules"
          :current-logic="currentLogic"
          :available-fields="availableFieldsConfig"
          :logic-options="logicOptions"
          :field-options="fieldOptions"
          :operator-options="operatorOptions"
          @update:rules="rules = $event"
          @update:current-logic="currentLogic = $event"
          @remove-group="handleRemoveGroup"
          @add-rule="addRule"
          @add-nested-group="addNestedGroup"
          @update-rule="updateRule"
          @field-change="handleFieldChange"
          @remove-rule="removeRule"
        />

        <!-- 权限预览 -->
        <q-expansion-item
          class="q-mt-md"
          icon="preview"
          :label="$t('common.permissionPreview')"
          caption="查看此配置对应的数据访问范围"
        >
          <q-card flat bordered class="q-pa-md">
            <div class="text-body2">
              <div v-for="(desc, idx) in permissionDescriptions" :key="idx" class="q-mb-xs">
                <q-icon name="check_circle" color="positive" size="16px" class="q-mr-xs" />
                {{ desc }}
              </div>
            </div>
          </q-card>
        </q-expansion-item>
      </q-card-section>

      <!-- 操作按钮 -->
      <q-separator />
      <q-card-actions align="right">
        <q-btn flat color="grey" :label="$t('common.reset')" @click="handleReset" />
        <q-btn color="primary" :label="$t('common.save')" @click="handleSave" />
      </q-card-actions>
    </q-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import { FilterOperator, LogicalOperator, OPERATOR_OPTIONS, createEmptyFilterRule, filterGroupToApiFormat, type FilterRule } from '@/types/rowPermission';
import FilterGroup from './DataFilterBuilder/FilterGroup.vue';
import type { FilterGroup as FilterGroupType } from '@/types/rowPermission';

const { t } = useI18n();
const $q = useQuasar();

const props = defineProps<{
  modelValue?: FilterGroupType;
  availableFields?: Array<{
    field: string;
    label: string;
    type: 'string' | 'number' | 'date' | 'boolean' | 'select';
    enumOptions?: Array<{ label: string; value: unknown }>;
  }>;
  entity_type?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: FilterGroupType];
  saved: [value: FilterGroupType];
}>();

// 常量
const logicOptions = [
  { label: t('permission.logicAnd'), value: LogicalOperator.AND },
  { label: t('permission.logicOr'), value: LogicalOperator.OR },
];

const operatorOptions = computed(() => OPERATOR_OPTIONS);

const fieldOptions = computed(() => {
  const fields = props.availableFields || defaultFields;
  return fields.map((f) => ({ label: f.label, value: f.field }));
});

const defaultFields = [
  { field: 'department_id', label: t('permission.fieldDepartmentId'), type: 'number' as const },
  { field: 'department_name', label: t('permission.fieldDepartmentName'), type: 'string' as const },
  { field: 'creatorId', label: t('permission.fieldCreatorId'), type: 'number' as const },
  { field: 'creatorName', label: t('permission.fieldCreatorName'), type: 'string' as const },
  { field: 'status', label: t('permission.fieldStatus'), type: 'select' as const, enumOptions: [
    { label: t('permission.statusEnabled'), value: 1 },
    { label: t('permission.statusDisabled'), value: 0 },
  ]},
  { field: 'created_at', label: t('permission.fieldCreatedAt'), type: 'date' as const },
  { field: 'updated_at', label: t('permission.fieldUpdatedAt'), type: 'date' as const },
];

// 状态
const rules = ref<FilterRule[]>([]);
const currentLogic = ref<LogicalOperator>(LogicalOperator.AND);

// 计算属性
const availableFieldsConfig = computed(() => props.availableFields || defaultFields);

const permissionDescriptions = computed(() => {
  if (rules.value.length === 0) return [t('permission.permissionAllData')];
  return rules.value.map((rule) => {
    const fieldConfig = (props.availableFields || defaultFields).find((f) => f.field === rule.field);
    const field = fieldConfig?.label || rule.field;
    const operator = OPERATOR_OPTIONS.find((o) => o.value === rule.operator);
    const operatorLabel = operator?.label || rule.operator;

    let value = '';
    if (Array.isArray(rule.value)) {
      value = `(${(rule.value as unknown[]).join(', ')})`;
    } else if (rule.value == null) {
      value = '';
    } else {
      value = String(rule.value);
    }

    if (rule.operator === FilterOperator.IS_NULL) return `${field} ${t('permission.permissionIsNull')}`;
    if (rule.operator === FilterOperator.IS_NOT_NULL) return `${field} ${t('permission.permissionIsNotNull')}`;
    return `${field} ${operatorLabel} ${value}`;
  });
});

// 方法
function addRule() {
  const newRule = createEmptyFilterRule();
  if (rules.value.length > 0) {
    newRule.logic = LogicalOperator.AND;
  }
  rules.value.push(newRule);
  emitUpdate();
}

function addNestedGroup() {
  $q.notify({ type: 'info', message: t('permission.nestedGroupDeveloping') });
}

function removeRule(index: number) {
  rules.value.splice(index, 1);
  emitUpdate();
}

function handleRemoveGroup() {
  emit('update:modelValue', { rules: [], logic: LogicalOperator.AND });
}

function updateRule(index: number, rule: FilterRule) {
  rules.value[index] = rule;
  emitUpdate();
}

function handleFieldChange(index: number) {
  const rule = rules.value[index]!;
  rule.value = '';
  rule.value2 = undefined;
  rules.value[index] = rule;
  emitUpdate();
}

function handleReset() {
  rules.value = [];
  currentLogic.value = LogicalOperator.AND;
  emitUpdate();
}

function handleSave() {
  const filterGroup: FilterGroupType = {
    rules: rules.value,
    logic: currentLogic.value,
  };
  const apiFormat = filterGroupToApiFormat(filterGroup);
  logger.info('【过滤条件】保存的数据：', apiFormat);
  emit('saved', filterGroup);
  $q.notify({ type: 'positive', message: t('permission.filterSaveSuccess') });
}

function emitUpdate() {
  emit('update:modelValue', { rules: rules.value, logic: currentLogic.value });
}
</script>
