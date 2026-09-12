/**
 * @file form/StepForm.vue
 * @description 多步骤表单组件 - 组合 StepHeader + StepContent + StepNavigation
 * @date 2026-04-04
 */

<template>
  <div class="step-form">
    <StepHeader
      :steps="steps"
      :current-step="currentStep"
      :clickable="clickable"
      @step-click="setStep"
    />

    <StepContent
      :current-step="currentStep"
      :form-data="formData"
      :transition-name="transitionName"
    >
      <template #[`step-${currentStep}`]>>
        <slot :name="'step-' + currentStep" :step="currentStep" :data="formData" />
      </template>
    </StepContent>

    <StepNavigation
      :current-step="currentStep"
      :steps-length="steps.length"
      :loading="loading"
      :show-prev="showPrev"
      :show-next="showNext"
      :show-submit="showSubmit"
      :submit-label="submitLabel"
      @prev="handlePrev"
      @next="handleNext"
      @submit="handleSubmit"
    >
      <template #extra-action="{ step }">
        <slot name="extra-action" :step="step" />
      </template>
    </StepNavigation>

    <ConfirmDialog
      v-model="confirmDialog.show"
      :title="confirmDialog.title"
      :message="confirmDialog.message"
      :confirm-text="confirmDialog.confirmText"
      :cancel-text="confirmDialog.cancelText"
      :type="confirmDialog.type"
      @confirm="confirmDialog.onConfirm()"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import StepHeader from './StepHeader.vue';
import StepContent from './StepContent.vue';
import StepNavigation from './StepNavigation.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';

interface StepConfig {
  title: string;
  description?: string;
  validate?: () => Promise<boolean> | boolean;
  onEnter?: () => Promise<void> | void;
  onLeave?: () => Promise<void> | void;
}

interface Props {
  steps: StepConfig[];
  modelValue?: Record<string, unknown>;
  loading?: boolean;
  clickable?: boolean;
  direction?: 'horizontal' | 'vertical';
  submitLabel?: string;
  showPrev?: boolean;
  showNext?: boolean;
  showSubmit?: boolean;
  validateOnChange?: boolean;
  confirmBeforeNext?: boolean;
  confirmBeforePrev?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: () => ({}),
  loading: false,
  clickable: true,
  direction: 'horizontal',
  submitLabel: '',
  showPrev: true,
  showNext: true,
  showSubmit: true,
  validateOnChange: true,
  confirmBeforeNext: false,
  confirmBeforePrev: false
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: Record<string, unknown>): void;
  (e: 'step-change', currentStep: number, prevStep: number): void;
  (e: 'finish', data: Record<string, unknown>): void;
  (e: 'next', step: number): void;
  (e: 'prev', step: number): void;
}>();

const currentStep = ref(0);
const formData = ref<Record<string, unknown>>({ ...props.modelValue });
const transitionName = ref('slide-left');
const confirmDialog = ref({
  show: false,
  title: '',
  message: '',
  confirmText: '',
  cancelText: '',
  type: 'warning' as const,
  onConfirm: (() => {})
});

watch(
  () => props.modelValue,
  (newVal) => {
    formData.value = { ...newVal };
  },
  { deep: true }
);

function updateFormData(data: Record<string, unknown>) {
  formData.value = { ...formData.value, ...data };
  emit('update:modelValue', formData.value);
}

async function setStep(step: number) {
  if (step < 0 || step >= props.steps.length) return;
  if (step === currentStep.value) return;

  const prevStep = currentStep.value;
  const isNext = step > prevStep;

  const prevStepConfig = props.steps[prevStep];
  const nextStepConfig = props.steps[step];
  if (!prevStepConfig || !nextStepConfig) return;

  if (isNext && prevStepConfig.validate) {
    const valid = await prevStepConfig.validate();
    if (!valid) return;
  }

  if (prevStepConfig.onLeave) {
    await prevStepConfig.onLeave();
  }

  currentStep.value = step;
  transitionName.value = isNext ? 'slide-left' : 'slide-right';

  if (nextStepConfig.onEnter) {
    await nextStepConfig.onEnter();
  }

  emit('step-change', step, prevStep);
}

async function handleNext() {
  if (props.confirmBeforeNext && currentStep.value < props.steps.length - 1) {
    const { t } = await import('vue-i18n').then(m => m.useI18n());
    confirmDialog.value = {
      show: true,
      title: t('step.confirmNextTitle'),
      message: t('step.confirmNextMessage'),
      confirmText: t('common.confirm'),
      cancelText: t('common.cancel'),
      type: 'warning' as const,
      onConfirm: () => { void setStep(currentStep.value + 1); }
    };
  } else {
    await setStep(currentStep.value + 1);
  }
  emit('next', currentStep.value);
}

async function handlePrev() {
  if (props.confirmBeforePrev && currentStep.value > 0) {
    const { t } = await import('vue-i18n').then(m => m.useI18n());
    confirmDialog.value = {
      show: true,
      title: t('step.confirmPrevTitle'),
      message: t('step.confirmPrevMessage'),
      confirmText: t('common.confirm'),
      cancelText: t('common.cancel'),
      type: 'warning',
      onConfirm: () => { void setStep(currentStep.value - 1); }
    };
  } else {
    await setStep(currentStep.value - 1);
  }
  emit('prev', currentStep.value);
}

async function handleSubmit() {
  const lastStep = props.steps.length - 1;
  const lastStepConfig = props.steps[lastStep];
  if (lastStepConfig?.validate) {
    const valid = await lastStepConfig.validate();
    if (!valid) return;
  }
  emit('finish', formData.value);
}

defineExpose({
  currentStep,
  formData,
  setStep,
  updateFormData,
  validate: async () => {
    const step = props.steps[currentStep.value];
    if (step?.validate) {
      return await step.validate();
    }
    return true;
  },
  reset: () => {
    currentStep.value = 0;
    formData.value = {};
    emit('update:modelValue', {});
  },
  next: handleNext,
  prev: handlePrev,
  submit: handleSubmit
});
</script>

<style scoped lang="scss">
.step-form {
  width: 100%;
}

// 垂直方向样式
.step-form.is-vertical {
  .step-indicator {
    flex-direction: column;
    align-items: flex-start;
    padding: 0 20px;
  }

  .step-item {
    max-width: none;
    margin-bottom: 20px;

    &::after {
      display: none;
    }
  }

  .step-connector {
    position: absolute;
    top: 36px;
    left: 15px;
    width: 2px;
    height: calc(100% - 36px);
    background: #e0e0e0;
  }
}

@media (max-width: 768px) {
  .step-indicator {
    flex-wrap: wrap;
    gap: 16px;
  }
}
</style>
