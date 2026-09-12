
/**
 * @file LoginPage.vue
 * @description 登录页面 - 主文件（拆分后 行）
 * @date 2026-04-03
 */

<template>
  <div class="login-page flex flex-center">
    <div class="floating-shapes">
      <div class="shape shape-1"></div>
      <div class="shape shape-2"></div>
      <div class="shape shape-3"></div>
      <div class="shape shape-4"></div>
    </div>

    <q-card class="login-card" bordered>
      <q-card-section class="text-center login-header">
        <div class="logo-container">
          <q-avatar size="72px" class="logo-avatar">
            <q-icon name="smart_toy" size="40px" />
          </q-avatar>
        </div>
        <div class="text-h5 q-mt-md text-weight-bold login-title">{{ $t('login.title') }}</div>
        <div class="text-grey-6 login-subtitle">{{ $t('login.subtitle') }}</div>
      </q-card-section>

      <q-card-section>
        <LoginForm
          :show-register-mode="showRegisterMode"
          :is-loading="isLoading"
          :error-message="errorMessage"
          :remaining-attempts="remainingAttempts"
          :can-submit="canSubmit"
          :on-login="handleLogin"
          :on-forgot-password="handleForgotPassword"
          :on-toggle-register="handleToggleRegister"
          :on-cancel-register="handleCancelRegister"
          @submit="handleSubmit"
          @register="handleToggleRegister"
          @cancel="handleCancelRegister"
        />
      </q-card-section>
    </q-card>

    <div class="text-caption text-grey-5 q-mt-lg copyright">
      {{ $t('loginPage.copyright') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from '@/stores/auth';
import LoginForm from './LoginForm.vue';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const $q = useQuasar();
const authStore = useAuthStore();

const isLoading = ref(false);

const showRegisterMode = ref(false);

const errorMessage = ref('');

const remainingAttempts = ref(0);

const formData = reactive({
  username: '',
  password: '',
  remember_me: false,
});

const canSubmit = computed(() => {
  if (showRegisterMode.value) {
    return formData.username.length > 0 && formData.password.length >= 8;
  }
  return formData.username.length > 0 && formData.password.length > 0;
});

async function handleLogin(username: string, password: string, rememberMe: boolean) {
  isLoading.value = true;
  try {
    await authStore.login({ username, password, remember_me: rememberMe });
    $q.notify({ type: 'positive', message: t('login.loginSuccess'), position: 'top' });
    const redirect = (route.query.redirect as string) || '/';
    await router.push(redirect);
  } catch (error: unknown) {
    errorMessage.value = error instanceof Error ? error.message : t('login.loginFailed');
    $q.notify({ type: 'negative', message: errorMessage.value, position: 'top' });
  } finally {
    isLoading.value = false;
  }
}

function handleForgotPassword() {
  $q.notify({ type: 'info', message: t('login.forgotPasswordHint'), position: 'top' });
}

function handleToggleRegister() {
  showRegisterMode.value = !showRegisterMode.value;
}

function handleCancelRegister() {
  showRegisterMode.value = false;
}

function handleSubmit() {
  if (showRegisterMode.value) {
    handleRegister();
  } else {
    handleLogin();
  }
}

function handleRegister(username: string, password: string) {
  if (password.length < 8) {
    errorMessage.value = t('login.passwordMinLength');
    $q.notify({ type: 'warning', message: errorMessage.value, position: 'top' });
    return;
  }
  $q.notify({ type: 'info', message: t('login.registerDemoHint'), position: 'top' });
}

function handleCancel() {
  showRegisterMode.value = false;
}
</script>

<style scoped>
.login-page {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  min-height: 100vh;
  position: relative;
  overflow: hidden;
}
.floating-shapes {
  position: fixed; top: 0; left: 0; width: 100%; height: 100%;
  pointer-events: none; z-index: 0;
}
.shape {
  position: absolute; border-radius: 50%; opacity: 0.1;
  animation: float 20s ease-in-out infinite;
}
.shape-1 { width: 300px; height: 300px; background: #fff; top: -100px; left: -100px; animation-delay: 0s; }
.shape-2 { width: 200px; height: 200px; background: #fff; top: 50%; right: -50px; animation-delay: -5s; }
.shape-3 { width: 150px; height: 150px; background: #fff; bottom: -50px; left: 20%; animation-delay: -10s; }
.shape-4 { width: 100px; height: 100px; background: #fff; top: 30%; left: 10%; animation-delay: -15s; }
@keyframes float {
  0%, 100% { transform: translate(0, 0) rotate(0deg); }
  25% { transform: translate(20px, 20px) rotate(5deg); }
  50% { transform: translate(0, 40px) rotate(0deg); }
  75% { transform: translate(-20px, 20px) rotate(-5deg); }
}
.login-card {
  width: 100%; max-width: 420px; border-radius: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative; z-index: 2; animation: slideUp 0.5s ease forwards;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}
@keyframes slideUp {
  from { opacity: 0; transform: translateY(30px); }
  to { opacity: 1; transform: translateY(0); }
}
.login-card:hover { transform: translateY(-4px); box-shadow: 0 25px 70px rgba(0, 0, 0, 0.35); }
.login-header { padding-bottom: 0; }
.logo-container { display: flex; justify-content: center; margin-bottom: 8px; }
.logo-avatar {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white; box-shadow: 0 8px 24px rgba(102, 126, 234, 0.4);
  animation: pulse 2s ease-in-out infinite;
}
@keyframes pulse {
  0%, 100% { box-shadow: 0 8px 24px rgba(102, 126, 234, 0.4); }
  50% { box-shadow: 0 12px 32px rgba(102, 126, 234, 0.6); }
}
.login-title {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;
}
.login-subtitle { font-size: 14px; margin-top: 4px; }
.copyright { font-size: 12px; }
</style>
