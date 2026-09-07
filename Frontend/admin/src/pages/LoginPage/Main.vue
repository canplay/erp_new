// @ts-nocheck
<template>
  <div class="login-page">
    <div class="login-page__header">
      <HeaderLogo />
    </div>

    <div class="login-page__content">
      <div class="login-page__form-section">
        <LoginForm
          :show-register-mode="showRegisterMode as any"
          :is-loading="isLoading as any"
          :error-message="error_message as any"
          :remaining-attempts="(remainingAttempts || 0) as any"
          :can-submit="canSubmit as any"
          :on-login="(handleLogin as any)"
          :on-forgot-password="(handleForgotPassword as any)"
          :on-toggle-register="(handleRegister as any)"
          :on-cancel-register="(cancelRegister as any)"
          @submit="(handleLogin as any)"
        />
      </div>

      <div class="login-page__info-section">
        <div class="text-h5 q-mb-md">{{ $t('login.welcome') }}</div>
        <div class="text-body1">{{ $t('login.description') }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import HeaderLogo from './HeaderLogo.vue'
import LoginForm from './LoginForm.vue'
import { useLogin } from './useLogin'

const { t: $t } = useI18n()

const { handleLogin } = useLogin()
</script>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.login-page__header {
  padding: 16px;
}

.login-page__content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}

.login-page__form-section {
  background: white;
  border-radius: 12px;
  padding: 32px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  max-width: 400px;
  width: 100%;
}

.login-page__info-section {
  color: white;
  max-width: 400px;
  margin-left: 32px;
}

@media (max-width: 768px) {
  .login-page__content {
    flex-direction: column;
  }

  .login-page__info-section {
    margin-left: 0;
    margin-top: 24px;
  }
}
</style>
