/** * @file LoginForm.vue * @description 登录表单组件 - 输入框、按钮、状态管理 * @date 2026-08-22 */

<template>
  <q-card-section>
    <q-form @submit="handleLogin" class="q-gutter-y-md" @keydown.enter="handleEnterKey">
      <!-- 用户名 -->
      <q-input
        ref="usernameInputRef"
        v-model="formData.username"
        :label="$t('login.username')"
        outlined
        autofocus
        class="modern-input"
        :rules="[(val) => !!val || $t('login.validation.usernameRequired')]"
        @keydown.enter="passwordInputRef?.focus()"
      >
        <template v-slot:prepend>
          <q-icon name="person" class="input-icon" />
        </template>
      </q-input>

      <!-- 密码 -->
      <q-input
        ref="passwordInputRef"
        v-model="formData.password"
        :type="showPassword ? 'text' : 'password'"
        :label="$t('login.password')"
        outlined
        class="modern-input"
        :rules="[(val) => !!val || $t('login.validation.passwordRequired')]"
        @keyup.enter="handleLogin"
      >
        <template v-slot:prepend>
          <q-icon name="lock" class="input-icon" />
        </template>
        <template v-slot:append>
          <q-icon
            :name="showPassword ? 'visibility_off' : 'visibility'"
            class="cursor-pointer input-icon"
            @click="showPassword = !showPassword"
          />
        </template>
      </q-input>

      <!-- 密码强度提示（注册时显示） -->
      <PasswordStrength v-if="showRegisterMode" :password="formData.password" />

      <!-- 记住登录 -->
      <div class="row justify-between items-center">
        <q-checkbox
          v-model="formData.remember_me"
          :label="$t('login.remember_me')"
          class="remember-checkbox"
        />
        <div class="row items-center">
          <q-btn
            v-if="!showRegisterMode"
            flat
            dense
            color="primary"
            :label="$t('login.register')"
            class="action-btn"
            @click="showRegisterMode = true"
          />
          <q-btn
            v-if="showRegisterMode"
            flat
            dense
            color="grey"
            :label="$t('login.backToLogin')"
            class="action-btn"
            @click="cancelRegister"
          />
          <q-btn
            flat
            dense
            color="grey"
            :label="$t('login.forgotPassword')"
            @click="$emit('forgotPassword')"
            class="action-btn q-ml-sm"
          />
        </div>
      </div>

      <!-- 错误提示 -->
      <q-banner v-if="error_message" class="error-banner" rounded>
        <template v-slot:avatar>
          <q-icon name="error" />
        </template>
        {{ error_message }}
      </q-banner>

      <!-- 登录失败次数提示 -->
      <q-banner
        v-if="remainingAttempts > 0 && remainingAttempts < 5"
        class="warning-banner"
        rounded
      >
        <template v-slot:avatar>
          <q-icon name="warning" />
        </template>
        {{ $t('login.loginAttemptsLeft', { count: remainingAttempts }) }}
      </q-banner>

      <!-- 登录按钮 -->
      <q-btn
        type="submit"
        class="login-btn full-width"
        size="large"
        :loading="isLoading"
        :disable="isLoading || !canSubmit"
      >
        <q-icon
          v-if="!isLoading"
          :name="showRegisterMode ? 'person_add' : 'login'"
          class="q-mr-sm"
        />
        {{ showRegisterMode ? $t('login.registerButton') : $t('login.loginButton') }}
        <template v-slot:loading>
          <q-spinner-hourglass class="on-left" />
          {{ showRegisterMode ? $t('login.registering') : $t('login.loggingIn') }}
        </template>
      </q-btn>
    </q-form>
  </q-card-section>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from '@/stores/auth';
import PasswordStrength from '@/components/PasswordStrength.vue';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const $q = useQuasar();
const authStore = useAuthStore();

// ============ Refs ============
const usernameInputRef = ref<HTMLElement | null>(null);
const passwordInputRef = ref<HTMLElement | null>(null);

// ============ 表单数据 ============
const formData = reactive({
  username: '',
  password: '',
  remember_me: false,
});

// ============ UI 状态 ============
const showPassword = ref(false);
const isLoading = ref(false);
const error_message = ref('');
const showRegisterMode = ref(false);
const remainingAttempts = ref(5); // 剩余尝试次数

// ============ 防抖控制 ============
let loginDebounceTimer: ReturnType<typeof setTimeout> | null = null;
const isSubmitting = ref(false);

/**
 * @brief 检查是否可以提交
 */
const canSubmit = computed(() => {
  if (showRegisterMode.value) {
    return formData.username.length > 0 && formData.password.length >= 8;
  }
  return formData.username.length > 0 && formData.password.length > 0;
});

/**
 * @brief 处理回车键
 */
function handleEnterKey(event: KeyboardEvent) {
  if (event.key === 'Enter' && canSubmit.value && !isSubmitting.value) {
    void handleLogin();
  }
}

const emit = defineEmits<{
  (e: 'login', data: { username: string; password: string; remember_me: boolean }): void;
  (e: 'register', data: { username: string; password: string }): void;
  (e: 'forgotPassword'): void;
  (e: 'update:remainingAttempts', count: number): void;
}>();

/**
 * @brief 处理登录（带防抖）
 */
async function handleLogin() {
  if (isSubmitting.value) return;
  if (loginDebounceTimer) {
    clearTimeout(loginDebounceTimer);
  }

  error_message.value = '';

  isSubmitting.value = true;
  loginDebounceTimer = setTimeout(() => {
    isSubmitting.value = false;
  }, 300);

  if (showRegisterMode.value) {
    if (formData.password.length < 8) {
      error_message.value = t('login.passwordMinLength');
      return;
    }
    emit('register', { username: formData.username, password: formData.password });
    $q.notify({ type: 'info', message: t('login.registerDemoHint'), position: 'top' });
    showRegisterMode.value = false;
    return;
  }

  isLoading.value = true;

  try {
    await authStore.login({
      username: formData.username,
      password: formData.password,
      remember_me: formData.remember_me,
    });

    $q.notify({
      type: 'positive',
      message: t('login.loginSuccess'),
      position: 'top',
    });

    const redirect = (route.query.redirect as string) || '/';
    await router.push(redirect);
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : t('login.loginFailed');
    error_message.value = message;

    if (remainingAttempts.value > 0) {
      remainingAttempts.value--;
      emit('update:remainingAttempts', remainingAttempts.value);
    }

    if (remainingAttempts.value === 0) {
      $q.notify({
        type: 'negative',
        message: t('login.loginAttemptsExhausted'),
        position: 'top',
        timeout: 5000,
      });
    }
  } finally {
    isLoading.value = false;
  }
}

/**
 * @brief 取消注册
 */
function cancelRegister() {
  showRegisterMode.value = false;
  formData.password = '';
}
</script>

<style scoped>
.modern-input {
  transition: all 0.25s ease;
}

.modern-input :deep(.q-field__control) {
  border-radius: 8px;
  transition: all 0.25s ease;
}

.modern-input :deep(.q-field__control:hover) {
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.2);
}

.modern-input :deep(.q-field--focused .q-field__control) {
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.3);
}

.input-icon {
  color: #667eea;
  transition: color 0.25s ease;
}

.modern-input :deep(.q-field--focused) .input-icon {
  color: #764ba2;
}

.remember-checkbox {
  font-size: 13px;
}

.remember-checkbox :deep(.q-checkbox__inner {
  color: #667eea;
}

.action-btn {
  font-size: 13px;
  transition: all 0.2s ease;
}

.action-btn:hover {
  color: #667eea !important;
}

.error-banner {
  background: linear-gradient(135deg, #ff6b6b 0%, #f03e3e 100%) !important;
  color: white !important;
  border-radius: 8px;
  animation: shake 0.5s ease;
}

@keyframes shake {
  0%, 100% {
    transform: translateX(0);
  }
  20%, 60% {
    transform: translateX(-5px);
  }
  40%, 80% {
    transform: translateX(5px);
  }
}

.error-banner :deep(.q-banner__avatar) {
  color: white;
}

.warning-banner {
  background: linear-gradient(135deg, #ffc107 0%, #fab005 100%) !important;
  color: #333 !important;
  border-radius: 8px;
}

.warning-banner :deep(.q-banner__avatar) {
  color: #333;
}

.login-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%) !important;
  color: white !important;
  border-radius: 8px;
  font-weight: 600;
  letter-spacing: 0.5px;
  transition: all 0.25s ease;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
  height: 48px;
}

.login-btn:hover {
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.5);
  transform: translateY(-2px);
}

.login-btn:active {
  transform: translateY(0);
}

.login-btn:disabled {
  opacity: 0.7;
}
</style>
