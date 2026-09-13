/**
 * @file form/StepContent.vue
 * @description 步骤表单 - 步骤内容区域（过渡动画）
 * @date 2026-08-22
 */

<template>
  <div class="step-content-wrapper">
    <transition :name="transitionName" mode="out-in">
      <div :key="currentStep" class="step-panel">
        <slot :name="'step-' + currentStep" :step="currentStep" :data="formData">
          <div class="step-default-content">
            <p>步骤 {{ currentStep + 1 }} 内容</p>
          </div>
        </slot>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

interface Props {
  currentStep: number;
  formData: Record<string, unknown>;
  transitionName: string;
}

const props = defineProps<Props>();
</script>

<style scoped>
.step-content-wrapper {
  min-height: 300px;
  padding: 20px;
  background: #fafafa;
  border-radius: 8px;
  margin-bottom: 20px;
}

.step-panel {
  width: 100%;
}

.step-default-content {
  text-align: center;
  color: #999;
  padding: 40px 0;
}

.slide-left-enter-active,
.slide-left-leave-active,
.slide-right-enter-active,
.slide-right-leave-active {
  transition: all 0.3s ease;
}

.slide-left-enter-from {
  opacity: 0;
  transform: translateX(30px);
}

.slide-left-leave-to {
  opacity: 0;
  transform: translateX(-30px);
}

.slide-right-enter-from {
  opacity: 0;
  transform: translateX(-30px);
}

.slide-right-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
