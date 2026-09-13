/**
 * @file useUserDetail.ts
 * @description 用户详情页面业务逻辑 composable
 */

import { ref, reactive, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useRouter, useRoute } from 'vue-router';
import {
  getUser,
  updateUser,
  updateUserStatus,
  updateUserRole,
  resetUserPassword,
  deleteUser,
} from '@/api/user';
import type { User } from '@/types/user';
import { logger } from '@/utils/logger';

export function useUserDetail() {
  const $q = useQuasar();
  const { t: $t } = useI18n();
  const router = useRouter();
  const route = useRoute();

  // ============ 状态 ============
  const loading = ref(false);
  const user = ref<User | null>(null);

  const showEditDialog = ref(false);
  const showResetPasswordDialog = ref(false);

  const editForm = reactive({
    username: '',
    nickname: '',
    email: '',
    phone: '',
    role: 'user',
    status: 1,
  });

  const new_password = ref('');

  // ============ 计算属性 ============
  const roleOptions = computed(() => [
    { label: $t('user.admin'), value: 'admin' },
    { label: $t('user.normalUser'), value: 'user' },
    { label: $t('user.vip'), value: 'vip' },
  ]);

  const statusOptions = computed(() => [
    { label: $t('user.normal'), value: 1 },
    { label: $t('user.disabled'), value: 0 },
    { label: $t('user.locked'), value: 2 },
  ]);

  // ============ 辅助方法 ============

  function getStatusLabel(status?: number): string {
    const labels: Record<number, string> = {
      0: $t('user.disabled'),
      1: $t('user.normal'),
      2: $t('user.locked'),
    };
    return labels[status || 0] || $t('common.unknown');
  }

  function getStatusColor(status?: number): string {
    const colors: Record<number, string> = {
      0: 'negative',
      1: 'positive',
      2: 'warning',
    };
    return colors[status || 0] || 'grey';
  }

  function getStatusIcon(status?: number): string {
    const icons: Record<number, string> = {
      0: 'block',
      1: 'check_circle',
      2: 'lock',
    };
    return icons[status || 0] || '';
  }

  function getRoleLabel(role?: string): string {
    const labels: Record<string, string> = {
      admin: $t('user.admin'),
      vip: $t('user.vip'),
      user: $t('user.normalUser'),
    };
    return labels[role || 'user'] || $t('user.normalUser');
  }

  function getRoleChipColor(role?: string): string {
    const colors: Record<string, string> = {
      admin: 'primary',
      vip: 'amber',
      user: 'grey',
    };
    return colors[role || 'user'] || 'grey';
  }

  function getGenderLabel(gender?: number): string {
    const labels: Record<number, string> = {
      0: $t('user.unknown'),
      1: $t('user.male'),
      2: $t('user.female'),
    };
    return labels[gender || 0] || $t('user.unknown');
  }

  function formatDateTime(timestamp?: number | string): string {
    if (!timestamp) return '-';
    const date = new Date(typeof timestamp === 'string' ? timestamp : timestamp);
    return date.toLocaleString();
  }

  // ============ 数据操作 ============

  async function loadUserDetail() {
    const user_id = Number(route.params.id);
    if (!user_id) {
      user.value = null;
      return;
    }

    loading.value = true;
    try {
      const response = await getUser(user_id);
      user.value = (response as { data?: User }).data ?? null;
      if (user.value) {
        editForm.username = user.value.username;
        editForm.nickname = user.value.nickname || '';
        editForm.email = user.value.email || '';
        editForm.phone = user.value.phone || '';
        editForm.role = user.value.role || 'user';
        editForm.status = user.value.status || 1;
      }
    } catch (error) {
      logger.error('【加载用户详情失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
      user.value = null;
    } finally {
      loading.value = false;
    }
  }

  function goBack() {
    router.back();
  }

  async function saveUser() {
    if (!user.value) return;

    try {
      await updateUser(user.value.id, {
        email: editForm.email,
        phone: editForm.phone,
      });

      if (editForm.role !== user.value.role) {
        await updateUserRole(user.value.id, editForm.role);
      }

      if (editForm.status !== user.value.status) {
        await updateUserStatus(user.value.id, editForm.status);
      }

      $q.notify({ type: 'positive', message: $t('common.success') });
      showEditDialog.value = false;
      void loadUserDetail();
    } catch (error) {
      logger.error('【保存失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  async function toggleUserStatus() {
    if (!user.value) return;

    const newStatus = user.value.status === 1 ? 0 : 1;
    try {
      await updateUserStatus(user.value.id, newStatus);
      $q.notify({ type: 'positive', message: $t('common.success') });
      void loadUserDetail();
    } catch (error) {
      logger.error('【更新状态失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  async function resetPassword() {
    if (!user.value || !new_password.value) return;

    try {
      await resetUserPassword(user.value.id, new_password.value);
      $q.notify({ type: 'positive', message: $t('common.success') });
      showResetPasswordDialog.value = false;
      new_password.value = '';
    } catch (error) {
      logger.error('【重置密码失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  async function doDeleteUser(): Promise<void> {
    if (!user.value) return;

    try {
      await deleteUser(user.value.id);
      $q.notify({ type: 'positive', message: $t('common.success') });
      await router.push('/users');
    } catch (error) {
      logger.error('【删除失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
      throw error;
    }
  }

  onMounted(() => {
    void loadUserDetail();
  });

  return {
    loading,
    user,
    showEditDialog,
    showResetPasswordDialog,
    editForm,
    new_password,
    roleOptions,
    statusOptions,
    getStatusLabel,
    getStatusColor,
    getStatusIcon,
    getRoleLabel,
    getRoleChipColor,
    getGenderLabel,
    formatDateTime,
    goBack,
    saveUser,
    toggleUserStatus,
    resetPassword,
    doDeleteUser,
  };
}
