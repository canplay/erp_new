<template>
  <div class="step-form">
    <div class="text-h6 q-mb-md">{{ title }}</div>

    <StepperIndicator :currentStep="currentStep" :totalSteps="totalSteps" />

    <q-separator />

    <div class="step-form__content">
      <slot :name="`step-${currentStep}`" />
    </div>

    <q-card-actions align="right" class="step-form__actions">
      <q-btn
        v-if="currentStep > 0"
        flat
        color="grey"
        :label="$t('common.prevStep')"
        @click="prevStep"
      />
      <q-btn
        v-if="currentStep < totalSteps - 1"
        color="primary"
        :label="$t('common.nextStep')"
        @click="nextStep"
      />
      <q-btn
        v-if="currentStep === totalSteps - 1"
        color="positive"
        :label="$t('common.submit')"
        @click="submit"
      />
    </q-card-actions>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import StepperIndicator from './StepperIndicator.vue'
import { useStepForm } from './useStepForm'

interface Props {
  title?: string
  totalSteps: number
}

const props = withDefaults(defineProps<Props>(), {
  title: '分步表单'
})

const emit = defineEmits<{
  (e: 'next'): void
  (e: 'prev'): void
  (e: 'submit'): void
}>()

const { currentStep, nextStep, prevStep, submit } = useStepForm({
  totalSteps: props.totalSteps,
  onNext: emit,
  onPrev: emit,
  onSubmit: emit
})

const { t: $t } = useI18n()
</script>

<style scoped>
.step-form {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
}

.step-form__content {
  min-height: 300px;
  padding: 16px 0;
}

.step-form__actions {
  border-top: 1px solid rgba(0, 0, 0, 0.12);
  padding-top: 16px;
}
</style>
