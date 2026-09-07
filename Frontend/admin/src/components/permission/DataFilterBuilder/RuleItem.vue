<template>
  <div class="rule-item">
    <div class="rule-item__header">
      <span class="text-subtitle2">{{ $t('filter.rule') }} {{ index + 1 }}</span>
      <q-btn
        flat
        dense
        icon="close"
        color="negative"
        @click="$emit('remove', index)"
      />
    </div>

    <div class="rule-item__content row q-col-gutter-sm">
      <div class="col-4">
        <q-select
          v-model="localRule.field"
          :options="fieldOptions"
          emit-value
          map-options
          dense
          outlined
          :label="$t('filter.field')"
          @update:model-value="(val: string) => $emit('update:rule', { ...localRule, field: val })"
        />
      </div>

      <div class="col-4">
        <q-select
          v-model="localRule.operator"
          :options="getOperatorOptions(localRule.field)"
          emit-value
          map-options
          dense
          outlined
          :label="$t('filter.operator')"
          @update:model-value="(val: string) => $emit('update:rule', { ...localRule, operator: val })"
        />
      </div>

      <div class="col-4">
        <q-input
          v-model="localRule.value"
          dense
          outlined
          :label="$t('filter.value')"
          @input="(val: string) => $emit('update:rule', { ...localRule, value: val })"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Rule {
  field: string
  operator: string
  value: string
}

interface Props {
  rule: Rule
  index: number
  fieldOptions: Array<{ label: string; value: string; type: string }>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:rule', rule: Rule): void
  (e: 'remove', index: number): void
}>()

const localRule = computed({
  get: () => ({ ...props.rule }),
  set: (val) => emit('update:rule', val)
})

function getOperatorOptions(field: string): Array<{ label: string; value: string }> {
  const fieldInfo = props.fieldOptions.find(f => f.value === field)
  const type = fieldInfo?.type || 'string'

  const operators: Record<string, Array<{ label: string; value: string }>> = {
    string: [
      { label: '等于', value: 'eq' },
      { label: '包含', value: 'contains' },
      { label: '开头是', value: 'starts_with' }
    ],
    number: [
      { label: '等于', value: 'eq' },
      { label: '大于', value: 'gt' },
      { label: '小于', value: 'lt' }
    ],
    date: [
      { label: '等于', value: 'eq' },
      { label: '早于', value: 'lt' },
      { label: '晚于', value: 'gt' }
    ]
  }

  return (operators[type] ?? operators.string) as Array<{ label: string; value: string }>
}
</script>

<style scoped>
.rule-item {
  padding: 12px;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 8px;
  background: white;
}

.rule-item__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.rule-item__content {
  margin-top: 8px;
}

.body--dark .rule-item {
  background: #1e1e1e;
  border-color: #3d3d3d;
}
</style>
