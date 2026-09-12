/**
 * @file CaptchaWidget.vue
 * @description 图形验证码组件
 * @date 2026-04-03
 */

<template>
  <div class="captcha-widget">
    <div class="row items-center q-gutter-md">
      <div class="captcha-image">
        <img :src="captchaSrc" :alt="$t('common.captcha')" @click="refreshCaptcha" />
      </div>
      <q-input
        v-model="captchaCode"
        outlined
        dense
        type="text"
        :label="$t('login.captcha')"
        :placeholder="$t('login.captchaPlaceholder')"
        style="max-width: 160px"
        class="captcha-input"
      />
      <q-btn
        flat
        dense
        color="primary"
        icon="refresh"
        :label="$t('login.refreshCaptcha')"
        @click="refreshCaptcha"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
const emit = defineEmits<{
  (e: 'verified'): void;
}>();

const captchaSrc = ref('');
const captchaCode = ref('');

function refreshCaptcha() {
  // 模拟刷新验证码
  captchaSrc.value = `/api/captcha?${Date.now()}`;
  captchaCode.value = '';
}

onMounted(() => {
  refreshCaptcha();
});
</script>

<style scoped>
.captcha-widget { margin-bottom: 16px; }
.captcha-image img {
  border-radius: 4px;
  cursor: pointer;
  height: 40px;
}
.captcha-input { flex: 1; }
</style>
