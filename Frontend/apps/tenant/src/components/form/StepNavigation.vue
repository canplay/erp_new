/**
 * @file form/StepNavigation.vue
 * @description 步骤表单 - 导航按钮（上一步/下一步/提交）
 * @date 2026-08-22
 */

<template>
  <div class="step-navigation">
    <div class="action-left">
      <slot name="extra-action" :step="currentStep" />
    </div>
    <div class="action-right">
      <q-btn
        v-if="showPrev && currentStep > 0"
        flat
        :label="t('common.previous')"
        :disable="loading"
        @click="$emit('prev')"
      />
      <q-btn
        v-if="showNext && currentStep < props.stepsLength - 1"
        color="primary"
        :label="t('common.next')"
        :loading="loading"
        @click="$emit('next')"
      >
        <template #loading>
          <q-spinner-dots />
        </template>
      </q-btn>
      <q-btn
        v-if="showSubmit && currentStep === props.stepsLength - 1"
        color="primary"
        :label="submitLabel || t('common.submit')"
        :loading="loading"
        @click="$emit('submit')"
      >
        <template #loading>
          <q-spinner-dots />
        </template>
      </q-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

interface Props {
  currentStep: number;
  stepsLength: number;
  loading: boolean;
  showPrev?: boolean;
  showNext?: boolean;
  showSubmit?: boolean;
  submitLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  showPrev: true,
  showNext: true,
  showSubmit: true,
  submitLabel: ''
});

const { t } = useI18n();

const emit = defineEmits<{
  'prev': [];
  'next': [];
  'submit': [];
}>();
</script>

<style scoped>
.step-navigation {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
}

.action-left,
.action-right {
  display: flex;
  gap: 12px;
}
</style>
