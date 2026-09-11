<template>
  <div class="stepper-indicator">
    <div class="row items-center justify-center q-pa-md">
      <div
        v-for="step in 3"
        :key="step"
        class="step-item"
        :class="{ 'step-item--active': currentStep >= step, 'step-item--completed': currentStep > step }"
      >
        <div class="step-item__circle">
          <q-icon v-if="currentStep > step" name="check" size="16px" color="white" />
          <span v-else class="step-item__number">{{ step }}</span>
        </div>
        <div class="step-item__label">{{ $t(`import.step${step}`) }}</div>
        <div v-if="step < 3" class="step-item__line" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()

defineProps<{
  currentStep: number
}>()
</script>

<style scoped>
.stepper-indicator {
  padding: 16px 0;
}

.step-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
}

.step-item__circle {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #ccc;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: bold;
}

.step-item--active .step-item__circle {
  background: #1976d2;
}

.step-item--completed .step-item__circle {
  background: #2e7d32;
}

.step-item__label {
  margin-top: 4px;
  font-size: 12px;
}

.step-item__line {
  width: 60px;
  height: 2px;
  background: #ccc;
  position: absolute;
  top: 16px;
  left: 100%;
}

.step-item--active .step-item__line,
.step-item--completed .step-item__line {
  background: #1976d2;
}
</style>
