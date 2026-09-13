/**
 * @file LoginHeader.vue
 * @description 登录页头部 - Logo + 标题
 * @date 2026-08-22
 */

<template>
  <q-card-section class="text-center login-header">
    <div class="logo-container">
      <q-avatar size="72px" class="logo-avatar">
        <q-icon name="smart_toy" size="40px" />
      </q-avatar>
    </div>
    <div class="text-h5 q-mt-md text-weight-bold login-title">{{ $t('login.title') }}</div>
    <div class="text-grey-6 login-subtitle">{{ $t('login.subtitle') }}</div>
  </q-card-section>
</template>

<style scoped>
.login-header {
  padding-bottom: 0;
}

.logo-container {
  display: flex;
  justify-content: center;
  margin-bottom: 8px;
}

.logo-avatar {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  box-shadow: 0 8px 24px rgba(102, 126, 234, 0.4);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 8px 24px rgba(102, 126, 234, 0.4);
  }
  50% {
    box-shadow: 0 12px 32px rgba(102, 126, 234, 0.6);
  }
}

.login-title {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.login-subtitle {
  font-size: 14px;
  margin-top: 4px;
}
</style>
