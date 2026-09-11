/**
 * @file useUserColumns.ts
 * @description User list table column definitions and display helpers
 */

import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { User } from '@/api/user';

export function useUserColumns() {
  const { t: $t } = useI18n();

  const defaultVisibleColumns = ['avatar', 'username', 'nickname', 'email', 'role', 'status', 'created_at', 'actions'];
  const visibleColumns = ref<string[]>([...defaultVisibleColumns]);

  const columnDefinitions = computed(() => [
    { name: 'id', label: $t('user.user_id'), field: 'id', align: 'left' as const, sortable: true },
    { name: 'avatar', label: $t('user.avatar'), field: 'avatar', align: 'center' as const, sortable: false },
    { name: 'username', label: $t('user.username'), field: 'username', align: 'left' as const, sortable: true },
    { name: 'nickname', label: $t('user.nickname'), field: 'nickname', align: 'left' as const, sortable: true },
    { name: 'email', label: $t('user.email'), field: 'email', align: 'left' as const, sortable: true },
    { name: 'phone', label: $t('user.phone'), field: 'phone', align: 'left' as const, sortable: false },
    { name: 'role', label: $t('user.role'), field: 'role', align: 'center' as const, sortable: true },
    { name: 'status', label: $t('user.status'), field: 'status', align: 'center' as const, sortable: true },
    { name: 'created_at', label: $t('user.created_at'), field: 'created_at', align: 'left' as const, sortable: true },
    { name: 'last_login_at', label: $t('user.last_login_at'), field: 'last_login_at', align: 'left' as const, sortable: true },
    { name: 'actions', label: $t('user.actions'), field: 'actions', align: 'center' as const, sortable: false },
  ]);

  const displayedColumns = computed(() =>
    columnDefinitions.value.filter((col) => visibleColumns.value.includes(col.name)),
  );

  const statusOptionsForSearch = [
    { label: $t('user.active'), value: 1 },
    { label: $t('user.inactive'), value: 0 },
    { label: $t('user.locked'), value: 2 },
  ] as const;

  const roleOptionsForSearch = [
    { label: $t('user.admin'), value: 'admin' },
    { label: $t('user.vip'), value: 'vip' },
    { label: $t('user.normalUser'), value: 'user' },
  ] as const;

  const statusOptionsForDialog = [
    { label: $t('user.active'), value: 1 },
    { label: $t('user.inactive'), value: 0 },
    { label: $t('user.locked'), value: 2 },
  ] as const;

  const roleOptionsForDialog = [
    { label: $t('user.admin'), value: 'admin' },
    { label: $t('user.vip'), value: 'vip' },
    { label: $t('user.normalUser'), value: 'user' },
  ] as const;

  function getStatusLabel(status?: number): string {
    const labels: Record<number, string> = {
      0: $t('user.inactive'),
      1: $t('user.active'),
      2: $t('user.locked'),
    };
    return labels[status || 0] || $t('common.unknown') || 'Unknown';
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

  const importPreviewColumns = computed(() => [
    { name: 'rowIndex', label: '#', field: 'rowIndex', align: 'center' as const },
    { name: 'username', label: $t('user.username'), field: 'username', align: 'left' as const },
    { name: 'nickname', label: $t('user.nickname'), field: 'nickname', align: 'left' as const },
    { name: 'email', label: $t('user.email'), field: 'email', align: 'left' as const },
    { name: 'phone', label: $t('user.phone'), field: 'phone', align: 'left' as const },
    { name: 'role', label: $t('user.role'), field: 'role', align: 'center' as const },
    { name: 'errors', label: $t('common.status'), field: 'errors', align: 'center' as const },
  ]);

  return {
    columnDefinitions,
    displayedColumns,
    defaultVisibleColumns,
    visibleColumns,
    statusOptionsForSearch,
    roleOptionsForSearch,
    statusOptionsForDialog,
    roleOptionsForDialog,
    importPreviewColumns,
    getStatusLabel,
    getStatusColor,
    getStatusIcon,
    getRoleLabel,
    getRoleChipColor,
  };
}
