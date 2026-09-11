/**
 * @file MemoryWidget.vue
 * @description 记忆验证码组件 - 基于用户记忆信息的验证
 * @date 2026-04-03
 */

<template>
  <div class="memory-captcha">
    <div class="text-subtitle2 q-mb-sm">{{ $t('login.memoryCaptchaTitle') }}</div>
    <div class="text-body2 q-mb-md">
      {{ $t('login.memoryCaptchaQuestion') }}
    </div>
    <q-input
      v-model="answer"
      outlined
      dense
      :label="$t('login.memoryCaptchaAnswer')"
      :placeholder="$t('login.memoryCaptchaPlaceholder')"
      @keyup.enter="submitAnswer"
    />
    <q-btn
      color="primary"
      :label="$t('login.submit')"
      class="q-mt-md full-width"
      :disable="!answer.trim()"
      @click="submitAnswer"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const emit = defineEmits<{
  (e: 'verified'): void;
}>();

const answer = ref('');

function submitAnswer() {
  if (answer.value.trim()) {
    // 模拟验证通过
    emit('verified');
  }
}
</script>

<style scoped>
.memory-captcha { margin-bottom: 16px; }
</style>
