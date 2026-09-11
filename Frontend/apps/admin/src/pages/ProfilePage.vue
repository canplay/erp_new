<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('profile.title') }}</div>

    <div class="row q-col-gutter-md">
      <!-- 基本信息 -->
      <div class="col-12 col-md-4">
        <q-card bordered>
          <q-card-section class="text-center">
            <q-avatar size="100px" color="primary" text-color="white" class="q-mb-md">
              {{ userInfo?.username?.substring(0, 1).toUpperCase() }}
            </q-avatar>
            <div class="text-h6">{{ userInfo?.username }}</div>
            <div class="text-caption text-grey-6">{{ userInfo?.email }}</div>
            <!-- 头像上传按钮 -->
            <q-btn flat color="primary" :label="$t('profile.changeAvatar')" class="q-mt-sm" @click="changeAvatar" />
          </q-card-section>
          <q-separator />
          <q-card-section>
            <q-list dense>
              <q-item>
                <q-item-section avatar><q-icon name="badge" /></q-item-section>
                <q-item-section>{{ $t('profile.role') }}：{{ userInfo?.role === 'admin' ? $t('user.admin') : $t('user.normalUser') }}</q-item-section>
              </q-item>
              <q-item>
                <q-item-section avatar><q-icon name="calendar_today" /></q-item-section>
                <q-item-section>{{ $t('profile.registered') }}：{{ formatDate(userInfo?.created_at) }}</q-item-section>
              </q-item>
              <q-item>
                <q-item-section avatar><q-icon name="schedule" /></q-item-section>
                <q-item-section>{{ $t('profile.lastLogin') }}：{{ formatDate(userInfo?.last_login_at) }}</q-item-section>
              </q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>

      <!-- 编辑表单 -->
      <div class="col-12 col-md-8">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('profile.basicInfo') }}</div>
            <q-form @submit="saveProfile" class="q-gutter-md">
              <q-input v-model="form.username" :label="$t('profile.username')" outlined disable />
              <q-input v-model="form.email" :label="$t('profile.email')" outlined type="email" />
              <q-input v-model="form.phone" :label="$t('profile.phone')" outlined type="tel" />
              <div>
                <q-btn type="submit" color="primary" :label="$t('profile.saveChanges')" :loading="saving" />
              </div>
            </q-form>
          </q-card-section>
        </q-card>

        <q-card bordered class="q-mt-md">
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('profile.changePassword') }}</div>
            <q-form @submit="changePwd" class="q-gutter-md">
              <q-input v-model="pwdForm.old_password" :label="$t('profile.old_password')" outlined type="password" />
              <q-input v-model="pwdForm.new_password" :label="$t('profile.new_password')" outlined type="password" />
              <!-- 密码强度提示 -->
              <PasswordStrength v-if="pwdForm.new_password" :password="pwdForm.new_password" />
              <q-input v-model="pwdForm.confirmPassword" :label="$t('profile.confirmPassword')" outlined type="password" />
              <div>
                <q-btn type="submit" color="negative" :label="$t('profile.updatePassword')" :loading="changingPwd" />
              </div>
            </q-form>
          </q-card-section>
        </q-card>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from '@/stores/auth';
import PasswordStrength from '@/components/PasswordStrength.vue';

const { t } = useI18n();
const $q = useQuasar();

const authStore = useAuthStore();

const saving = ref(false);
const changingPwd = ref(false);

const userInfo = ref(authStore.userInfo);

const form = reactive({
  username: '',
  email: '',
  phone: '',
});

const pwdForm = reactive({
  old_password: '',
  new_password: '',
  confirmPassword: '',
});

/**
 * @brief 格式化日期（后端返回 epoch 秒，需转毫秒）
 */
function formatDate(dateStr?: string | number): string {
  if (!dateStr) return '-';
  const num = typeof dateStr === 'string' ? Number(dateStr) : dateStr;
  if (!num) return '-';
  const date = new Date(num < 1e12 ? num * 1000 : num);
  return date.toLocaleDateString('zh-CN');
}

/**
 * @brief 更换头像
 */
function changeAvatar() {
  $q.notify({
    type: 'info',
    message: t('profile.avatarUploadDemo'),
    position: 'top',
  });
}

async function saveProfile() {
  saving.value = true;
  try {
    await authStore.updateProfile(form);
    $q.notify({ type: 'positive', message: t('profile.profileSaved') });
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  } finally {
    saving.value = false;
  }
}

async function changePwd() {
  if (pwdForm.new_password !== pwdForm.confirmPassword) {
    $q.notify({ type: 'warning', message: t('profile.passwordMismatch') });
    return;
  }
  changingPwd.value = true;
  try {
    await authStore.changePassword(pwdForm.old_password, pwdForm.new_password);
    $q.notify({ type: 'positive', message: t('profile.passwordChanged') });
    pwdForm.old_password = '';
    pwdForm.new_password = '';
    pwdForm.confirmPassword = '';
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  } finally {
    changingPwd.value = false;
  }
}

onMounted(() => {
  if (authStore.userInfo) {
    form.username = authStore.userInfo.username || '';
    form.email = authStore.userInfo.email || '';
    form.phone = authStore.userInfo.phone || '';
  }
});
</script>

<style scoped>
/* 暗色主题适配 */
.body--dark .text-h5 {
  color: #ffffff;
}

.body--dark .q-card {
  background: #1e1e1e;
  border-color: #2d2d2d;
}

.body--dark .q-item {
  color: #b0b0b0;
}

.body--dark .q-input :deep(.q-field__control) {
  background: #2d2d2d;
}

.body--dark .q-input :deep(.q-field__native) {
  color: #ffffff;
}

.body--dark .q-input :deep(.q-field__label) {
  color: #b0b0b0;
}
</style>
