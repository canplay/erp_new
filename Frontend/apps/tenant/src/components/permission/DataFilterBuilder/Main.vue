<template>
  <div class="data-filter-builder">
    <div class="text-h6 q-mb-md">{{ $t('filter.title') }}</div>

    <div class="filter-group">
      <RuleItem
        v-for="(ruleItem, index) in rules"
        :key="index"
        :rule="ruleItem"
        :index="index"
        :field-options="fieldOptions"
        :operator-options="operatorOptions"
        @update:rule="(val: { field: string; operator: string; value: string }) => updateRule(index, val)"
        @remove="removeRule(index)"
      />
    </div>

    <q-btn
      flat
      dense
      icon="add"
      :label="$t('filter.addRule')"
      color="primary"
      class="q-mt-md"
      @click="addRule"
    />

    <div class="row q-mt-md q-gutter-sm">
      <q-btn color="primary" :label="$t('filter.apply')" @click="applyFilters" />
      <q-btn flat color="grey" :label="$t('filter.reset')" @click="resetFilters" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useDataFilterBuilder } from './useDataFilterBuilder'
import RuleItem from './RuleItem.vue'

interface Props {
  fieldOptions: Array<{ label: string; value: string; type: string }>
  operatorOptions?: Array<{ label: string; value: string }>
  initialFilters?: Record<string, unknown>
}

const props = withDefaults(defineProps<Props>(), {
  fieldOptions: () => [],
  operatorOptions: () => [],
  initialFilters: () => ({})
})

const emit = defineEmits<{
  (e: 'change', filters: Record<string, unknown>): void
  (e: 'reset'): void
}>()

const {
  rules,
  addRule,
  removeRule,
  updateRule,
  applyFilters,
  resetFilters
} = useDataFilterBuilder({
  fieldOptions: props.fieldOptions,
  initialFilters: props.initialFilters,
  onChange: emit,
  onReset: emit
})
</script>

<style scoped>
.data-filter-builder {
  max-width: 800px;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
</style>
