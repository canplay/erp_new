<template>
  <div class="rule-list">
    <div v-for="(rule, index) in rules" :key="index" class="rule-item">
      <div class="rule-header">
        <span class="text-subtitle2">{{ $t('permission.condition') }} {{ index + 1 }}</span>
        <div class="row q-gutter-sm">
          <q-btn
            flat
            dense
            icon="edit"
            color="primary"
            @click="$emit('edit', index)"
          />
          <q-btn
            v-if="rules.length > 1"
            flat
            dense
            icon="delete"
            color="negative"
            @click="$emit('remove', index)"
          />
        </div>
      </div>

      <div class="rule-condition">
        <div class="row q-col-gutter-sm">
          <div class="col-4">
            <q-select
              v-model="rule.conditionField"
              :options="conditionOptions"
              emit-value
              map-options
              dense
              outlined
              :label="$t('permission.field')"
              @update:model-value="(val: string) => $emit('fieldChange', { index, value: val })"
            />
          </div>
          <div class="col-4">
            <q-select
              v-model="rule.conditionOperator"
              :options="operatorOptions"
              emit-value
              map-options
              dense
              outlined
              :label="$t('permission.operator')"
              @update:model-value="(val: string) => $emit('operatorChange', { index, value: val })"
            />
          </div>
          <div class="col-4">
            <q-input
              v-model="rule.conditionValue"
              dense
              outlined
              :label="$t('permission.value')"
              @input="(val: string) => $emit('valueChange', { index, value: val })"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()

interface Rule {
  conditionField: string
  conditionOperator: string
  conditionValue: string
}

interface Props {
  rules: Rule[]
  conditionOptions: Array<{ label: string; value: string }>
  operatorOptions: Array<{ label: string; value: string }>
}

defineProps<Props>()

defineEmits<{
  (e: 'edit', index: number): void
  (e: 'remove', index: number): void
  (e: 'fieldChange', payload: { index: number; value: string }): void
  (e: 'operatorChange', payload: { index: number; value: string }): void
  (e: 'valueChange', payload: { index: number; value: string }): void
}>()
</script>

<style scoped>
.rule-item {
  padding: 16px;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 8px;
  background: white;
}

.rule-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.rule-condition {
  margin-top: 8px;
}

.body--dark .rule-item {
  background: #1e1e1e;
  border-color: #3d3d3d;
}
</style>
