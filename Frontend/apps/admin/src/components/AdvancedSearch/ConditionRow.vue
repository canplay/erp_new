<template>
  <div class="condition-row">
    <q-select v-model="localCondition.field" :options="fieldOptions" dense outlined emit-value map-options class="field-select" :placeholder="'选择字段'" @update:model-value="(val: string) => $emit('update:condition', { ...localCondition, field: val })" />
    <q-select v-model="localCondition.operator" :options="getOperatorOptions(localCondition.field)" dense outlined emit-value map-options class="operator-select" @update:model-value="(val: string) => $emit('update:condition', { ...localCondition, operator: val })" />
    <component :is="getInputComponent(localCondition.field)" v-model="localCondition.value" :type="getInputType(localCondition.field)" :options="getOptionsForField(localCondition.field)" dense outlined class="value-input" :placeholder="$t('common.enterValue')" @input="(val: string) => $emit('update:condition', { ...localCondition, value: val })" />
    <q-btn flat round dense icon="close" color="grey" @click="$emit('remove', index)" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()

interface Condition {
  field: string
  operator: string
  value: unknown
}

interface Props {
  condition: Condition
  index: number
  fieldOptions: Array<{ label: string; value: string; type: string; options?: Array<{ label: string; value: string }> }>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:condition', condition: Condition): void
  (e: 'remove', index: number): void
}>()

const localCondition = computed({
  get: () => ({ ...props.condition }),
  set: (val) => emit('update:condition', val)
})

const operatorMap: Record<string, Array<{ label: string; value: string }>> = {
  string: [{ label: '等于', value: 'eq' }, { label: '包含', value: 'contains' }, { label: '开头是', value: 'starts_with' }],
  number: [{ label: '等于', value: 'eq' }, { label: '大于', value: 'gt' }, { label: '小于', value: 'lt' }],
  date: [{ label: '等于', value: 'eq' }, { label: '早于', value: 'lt' }, { label: '晚于', value: 'gt' }],
  select: [{ label: '等于', value: 'eq' }, { label: '不等于', value: 'ne' }],
  boolean: [{ label: '是', value: 'eq' }, { label: '否', value: 'ne' }]
}

function getOperatorOptions(field: string): Array<{ label: string; value: string }> {
  const fieldInfo = props.fieldOptions.find(f => f.value === field)
  const type = fieldInfo?.type || 'string'
  const ops = operatorMap[type]
  return (ops ?? operatorMap.string) as Array<{ label: string; value: string }>
}

function getInputComponent(field: string): string {
  const fieldInfo = props.fieldOptions.find(f => f.value === field)
  const type = fieldInfo?.type || 'string'
  if (type === 'select' && fieldInfo?.options) return 'q-select'
  if (type === 'boolean') return 'q-select'
  return 'q-input'
}

function getInputType(field: string): string {
  const fieldInfo = props.fieldOptions.find(f => f.value === field)
  const type = fieldInfo?.type || 'string'
  if (type === 'number') return 'number'
  if (type === 'date') return 'date'
  return 'text'
}

function getOptionsForField(field: string): Array<{ label: string; value: string }> {
  const fieldInfo = props.fieldOptions.find(f => f.value === field)
  return fieldInfo?.options ?? []
}
</script>

<style scoped>
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
</style>
