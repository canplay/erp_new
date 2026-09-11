/**
 * @file LoginPage.vue
 * @description 登录页面 - 组合 Header + Form + 背景装饰
 * @date 2026-08-22
 */

<template>
  <div class="login-page flex flex-center">
    <!-- 装饰元素 - 浮动圆圈 -->
    <div class="floating-shapes">
      <div class="shape shape-1"></div>
      <div class="shape shape-2"></div>
      <div class="shape shape-3"></div>
      <div class="shape shape-4"></div>
    </div>

    <q-card class="login-card" bordered>
      <!-- Logo 区域 -->
      <LoginHeader />

      <!-- 表单区域 -->
      <LoginForm
        @login="handleLogin"
        @register="handleRegister"
        @forgot-password="handleForgotPassword"
      />
    </q-card>

    <!-- 版权信息 -->
    <div class="text-caption text-grey-5 q-mt-lg copyright">
      {{ $t('loginPage.copyright') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from '@/stores/auth';
import LoginHeader from './LoginHeader.vue';
import LoginForm from './LoginForm.vue';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const $q = useQuasar();
const authStore = useAuthStore();

function handleLogin(data: { username: string; password: string; remember_me: boolean }) {
  // Handled by LoginForm internally
}

function handleRegister(data: { username: string; password: string }) {
  $q.notify({
    type: 'info',
    message: t('login.registerDemoHint'),
    position: 'top',
  });
}

function handleForgotPassword() {
  $q.dialog({
    title: t('login.forgotPasswordTitle'),
    message: t('login.forgotPasswordMessage'),
    ok: {
      label: t('login.forgotPasswordConfirm'),
      color: 'primary',
    },
    cancel: false,
  });
}
</script>

<style scoped>
.login-page {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  min-height: 100vh;
  position: relative;
  overflow: hidden;
}

/* 浮动装饰形状 */
.floating-shapes {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 0;
}

.shape {
  position: absolute;
  border-radius: 50%;
  opacity: 0.1;
  animation: float 20s ease-in-out infinite;
}

.shape-1 {
  width: 300px;
  height: 300px;
  background: #ffffff;
  top: -100px;
  left: -100px;
  animation-delay: 0s;
}

.shape-2 {
  width: 200px;
  height: 200px;
  background: #ffffff;
  top: 50%;
  right: -50px;
  animation-delay: -5s;
}

.shape-3 {
  width: 150px;
  height: 150px;
  background: #ffffff;
  bottom: -50px;
  left: 20%;
  animation-delay: -10s;
}

.shape-4 {
  width: 100px;
  height: 100px;
  background: #ffffff;
  top: 30%;
  left: 10%;
  animation-delay: -15s;
}

@keyframes float {
  0%, 100% {
    transform: translate(0, 0) rotate(0deg);
  }
  25% {
    transform: translate(20px, 20px) rotate(5deg);
  }
  50% {
    transform: translate(0, 40px) rotate(0deg);
  }
  75% {
    transform: translate(-20px, 20px) rotate(-5deg);
  }
}

/* 登录卡片 */
.login-card {
  width: 100%;
  max-width: 420px;
  border-radius: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  z-index: 2;
  animation: slideUp 0.5s ease forwards;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.login-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 25px 70px rgba(0, 0, 0, 0.35);
}

/* 版权信息 */
.copyright {
  position: absolute;
  bottom: 18px;
  left: 0;
  right: 0;
  text-align: center;
  z-index: 1;
  animation: fadeIn 1s ease 0.5s forwards;
  opacity: 0;
}

@keyframes fadeIn {
  to {
    opacity: 1;
  }
}

/* 暗色主题适配 */
.body--dark .login-page {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
}

.body--dark .login-card {
  background: #1e1e1e;
  border-color: #2d2d2d;
}

.body--dark .login-card :deep(.q-field__control) {
  background: #2d2d2d;
  border-color: #3d3d3d;
}

.body--dark .login-card :deep(.q-field__native) {
  color: #ffffff;
}

.body--dark .login-card :deep(.q-field__label) {
  color: #b0b0b0;
}

.body--dark .login-card :deep(.login-title) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.body--dark .login-card :deep(.input-icon) {
  color: #667eea;
}

.body--dark .shape {
  opacity: 0.05;
}

/* 响应式适配 */
@media (max-width: 599px) {
  .login-card {
    max-width: 95vw;
    margin: 16px;
  }

  .shape {
    display: none;
  }
}
</style>
