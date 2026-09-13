/**
 * @file form/StepHeader.vue
 * @description 步骤表单 - 步骤指示器（头部导航）
 * @date 2026-08-22
 */

<template>
  <div class="step-header">
    <div
      v-for="(step, index) in steps"
      :key="index"
      class="step-item"
      :class="{
        'is-active': currentStep === index,
        'is-completed': currentStep > index,
        'is-clickable': clickable && currentStep > index
      }"
      @click="handleStepClick(index)"
    >
      <div class="step-number">
        <q-icon v-if="currentStep > index" name="check" size="16px" />
        <span v-else>{{ index + 1 }}</span>
      </div>
      <div class="step-info">
        <div class="step-title">{{ step.title }}</div>
        <div v-if="step.description" class="step-description">
          {{ step.description }}
        </div>
      </div>
      <div v-if="index < steps.length - 1" class="step-connector">
        <div
          class="connector-line"
          :class="{ 'is-completed': currentStep > index }"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface StepConfig {
  title: string;
  description?: string;
  validate?: () => Promise<boolean> | boolean;
  onEnter?: () => Promise<void> | void;
  onLeave?: () => Promise<void> | void;
}

interface Props {
  steps: StepConfig[];
  currentStep: number;
  clickable?: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'step-click': [index: number];
}>();

function handleStepClick(index: number) {
  if (!props.clickable) return;
  if (index < props.currentStep) {
    emit('step-click', index);
  }
}
</script>

<style scoped>
.step-header {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 20px 0;
  margin-bottom: 24px;
}

.step-item {
  display: flex;
  align-items: flex-start;
  flex: 1;
  max-width: 280px;
  position: relative;
  cursor: default;
}

.step-item.is-clickable {
  cursor: pointer;
}

.step-item.is-clickable:hover .step-number {
  transform: scale(1.1);
}

.step-item.is-active .step-number {
  background: var(--q-primary);
  color: white;
  border-color: var(--q-primary);
}

.step-item.is-active .step-title {
  color: var(--q-primary);
  font-weight: 600;
}

.step-item.is-completed .step-number {
  background: var(--q-positive);
  border-color: var(--q-positive);
  color: white;
}

.step-item.is-completed .step-title {
  color: var(--q-positive);
}

.step-number {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: 2px solid #d0d5dd;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
  color: #666;
  background: white;
  transition: all 0.3s ease;
  flex-shrink: 0;
  z-index: 1;
}

.step-info {
  margin-left: 12px;
  padding-right: 20px;
}

.step-title {
  font-size: 14px;
  font-weight: 500;
  color: #333;
  line-height: 1.4;
  transition: color 0.3s ease;
}

.step-description {
  font-size: 12px;
  color: #999;
  margin-top: 4px;
  line-height: 1.4;
}

.step-connector {
  position: absolute;
  top: 16px;
  left: calc(50% + 20px);
  width: calc(100% - 60px);
  height: 2px;
  background: #e0e0e0;
}

.connector-line {
  height: 100%;
  background: #e0e0e0;
  transition: background 0.3s ease;
}

.connector-line.is-completed {
  background: var(--q-positive);
}
</style>
