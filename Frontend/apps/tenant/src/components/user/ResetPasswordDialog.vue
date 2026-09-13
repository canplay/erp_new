<template>
  <q-dialog v-model="showDialog" persistent>
    <q-card style="min-width: 400px">
      <q-card-section class="row items-center">
        <div class="text-h6">{{ $t('user.resetPassword') }}</div>
        <q-space />
        <q-btn flat round icon="close" v-close-popup />
      </q-card-section>

      <q-separator />

      <q-card-section>
        <q-form class="q-gutter-md">
          <!-- 用户信息 -->
          <div class="row items-center q-gutter-sm">
            <q-avatar size="40px" color="primary" text-color="white">
              {{ user?.username?.substring(0, 1).toUpperCase() || 'U' }}
            </q-avatar>
            <div>
              <div class="text-subtitle1">{{ user?.username }}</div>
              <div class="text-caption text-grey">{{ user?.email || user?.phone }}</div>
            </div>
          </div>

          <q-separator />

          <!-- 密码强度提示 -->
          <PasswordStrength v-model:password="new_password" />

          <!-- 新密码 -->
          <q-input
            v-model="new_password"
            :label="$t('user.new_password')"
            outlined
            :type="showPassword ? 'text' : 'password'"
            autocomplete="new-password"
            :rules="[
              (val) => !!val || $t('validation.required', { field: $t('user.new_password') }),
              (val) => val.length >= 8 || $t('validation.minLength', { field: $t('user.password'), length: 8 })
            ]"
          >
            <template #append>
              <q-icon
                :name="showPassword ? 'visibility_off' : 'visibility'"
                class="cursor-pointer"
                @click="showPassword = !showPassword"
              />
            </template>
          </q-input>

          <!-- 确认密码 -->
          <q-input
            v-model="confirmPassword"
            :label="$t('user.confirmPassword')"
            outlined
            :type="showPassword ? 'text' : 'password'"
            autocomplete="new-password"
            :error="!!confirmPassword && new_password !== confirmPassword"
            :error-message="$t('user.passwordMismatch')"
            :rules="[
              (val) => !!val || $t('validation.required', { field: $t('user.confirmPassword') })
            ]"
          >
            <template #append>
              <q-icon
                :name="showPassword ? 'visibility_off' : 'visibility'"
                class="cursor-pointer"
                @click="showPassword = !showPassword"
              />
            </template>
          </q-input>

          <!-- 密码不匹配警告 -->
          <q-banner v-if="new_password && confirmPassword && new_password !== confirmPassword" class="bg-warning text-white" rounded>
            <template #avatar>
              <q-icon name="warning" />
            </template>
            {{ $t('user.passwordMismatch') }}
          </q-banner>
        </q-form>
      </q-card-section>

      <q-separator />

      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" color="grey" v-close-popup />
        <q-btn
          color="primary"
          :label="$t('common.confirm')"
          :loading="loading"
          :disable="!isValid"
          @click="handleReset"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
/**
 * @file ResetPasswordDialog.vue
 * @description 用户密码重置弹窗
 * @date 2026-04-06
 */

import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useQuasar } from 'quasar';
import PasswordStrength from '@erp-new-frontend-monorepo/components/src/PasswordStrength.vue';
import { resetUserPassword } from '@/api/user';
import type { User } from '@/types/user';


const { t } = useI18n();
const $q = useQuasar();

// Props
interface Props {
  user?: User | null;
  modelValue?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  user: null,
  modelValue: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'success': [];
}>();

// 状态
const showDialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const new_password = ref('');
const confirmPassword = ref('');
const showPassword = ref(false);
const loading = ref(false);

// 验证
const isValid = computed(() => {
  return (
    new_password.value.length >= 8 &&
    new_password.value === confirmPassword.value
  );
});

// 重置表单
watch(showDialog, (value) => {
  if (!value) {
    new_password.value = '';
    confirmPassword.value = '';
    showPassword.value = false;
  }
});

// 处理重置
async function handleReset() {
  if (!props.user || !isValid.value) return;

  loading.value = true;
  try {
    await resetUserPassword(props.user.id, new_password.value);
    $q.notify({
      type: 'positive',
      message: t('user.passwordResetSuccess'),
    });
    emit('success');
    showDialog.value = false;
  } catch (error) {
    console.error('【密码重置失败】', error);
    $q.notify({
      type: 'negative',
      message: t('common.error'),
    });
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
/* 暗色主题适配 */
.body--dark .q-card {
  background: #1e1e1e;
}
</style>
