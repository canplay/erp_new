/**
 * @file useNotificationList.ts
 * @description 通知列表页面业务逻辑 composable
 */

import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useNotificationStore } from '@/stores/notification';
import type { Notification, NotificationType } from '@/types/notification';
import type { AdvancedFilters } from '@/types/advanced-search';
import { useExport } from '@/composables/useExport';

/** 删除对话框最小接口 */
interface DeleteDialog {
  open: () => void;
}

export function useNotificationList() {
  const $q = useQuasar();
  const { t: $t } = useI18n();
  const notificationStore = useNotificationStore();
  const { exportToCSV } = useExport();
const exportData = exportToCSV;

  // ============ 状态 ============

  const selectedNotifications = ref<Notification[]>([]);

  const filters = ref<AdvancedFilters>({
    keyword: '',
    start_date: '',
    end_date: '',
    status: null,
    type: null,
  });

  const showDetailDialog = ref(false);
  const currentNotification = ref<Notification | null>(null);
  const deleteDialogRef = ref<DeleteDialog | null>(null);
  const pendingDeleteNotification = ref<Notification | null>(null);

  const pagination = ref({
    page: 1,
    rowsPerPage: 20,
    rowsNumber: 0,
  });

  // ============ 计算属性 ============

  const typeOptionsForSearch = computed(() => [
    { label: $t('notification.typeOptions.all'), value: '' },
    { label: $t('notification.typeEnum.system'), value: 'system' },
    { label: $t('notification.typeEnum.operation'), value: 'operation' },
    { label: $t('notification.typeEnum.approval'), value: 'approval' },
  ]);

  const columns = computed(() => [
    { name: 'type', label: $t('notification.type'), field: 'type', align: 'center' as const },
    { name: 'priority', label: $t('notification.priority.label'), field: 'priority', align: 'center' as const },
    { name: 'title', label: $t('notification.title_field'), field: 'title', align: 'left' as const },
    { name: 'is_read', label: $t('notification.status'), field: 'is_read', align: 'center' as const },
    { name: 'created_at', label: $t('notification.created_at'), field: 'created_at', align: 'left' as const },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  // ============ 方法 ============

  async function loadNotifications() {
    const statusVal = filters.value.status;
    let is_read_val: boolean | null = null;
    if (statusVal !== null && statusVal !== '' && statusVal !== undefined) {
      const strVal = String(statusVal);
      is_read_val = strVal === '1' || strVal === 'true' || statusVal === 1;
    }

    const storeFilters = {
      type: (filters.value.type as NotificationType) || null,
      is_read: is_read_val,
    };

    notificationStore.setFilters(storeFilters);
    await notificationStore.fetchNotifications();
    pagination.value.rowsNumber = notificationStore.pagination.total;
  }

  function onTableRequest(props: {
    pagination: { page: number; rowsPerPage: number; rowsNumber?: number };
  }) {
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
    void loadNotifications();
  }

  function handleSearch() {
    pagination.value.page = 1;
    void loadNotifications();
  }

  function handleReset() {
    filters.value = {
      keyword: '',
      start_date: '',
      end_date: '',
      status: null,
      type: null,
    };
    pagination.value.page = 1;
    void loadNotifications();
  }

  function getTypeLabel(type: NotificationType): string {
    return $t(`notification.typeEnum.${type}`);
  }

  function getTypeColor(type: NotificationType): string {
    const colors: Record<NotificationType, string> = {
      system: 'blue',
      operation: 'green',
      approval: 'orange',
    };
    return colors[type] || 'grey';
  }

  function getPriorityColor(priority: string): string {
    const colors: Record<string, string> = {
      low: 'grey',
      normal: 'blue',
      high: 'orange',
      urgent: 'red',
    };
    return colors[priority] || 'grey';
  }

  function formatTime(timeStr: string): string {
    if (!timeStr) return '-';
    const date = new Date(timeStr);
    return date.toLocaleString('zh-CN');
  }

  function viewNotification(notification: Notification) {
    currentNotification.value = notification;
    showDetailDialog.value = true;

    if (!notification.is_read) {
      void notificationStore.markAsRead(notification.id);
    }
  }

  async function handleMarkRead(notification: Notification) {
    try {
      await notificationStore.markAsRead(notification.id);
      notification.is_read = true;
      notification.read_at = new Date().toISOString();
      $q.notify({ type: 'positive', message: $t('notification.markReadSuccess') });
      showDetailDialog.value = false;
    } catch {
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  async function handleMarkAllRead() {
    try {
      await notificationStore.markAllAsRead();
      $q.notify({ type: 'positive', message: $t('notification.markAllReadSuccess') });
      void loadNotifications();
    } catch {
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  function handleDelete(notification: Notification) {
    pendingDeleteNotification.value = notification;
    deleteDialogRef.value?.open();
  }

  async function doDeleteNotification() {
    if (!pendingDeleteNotification.value) return;

    try {
      await notificationStore.deleteNotification(pendingDeleteNotification.value.id);
      $q.notify({ type: 'positive', message: $t('common.success') });
      pendingDeleteNotification.value = null;
      void loadNotifications();
    } catch {
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  function handleBatchDelete(items: unknown[]) {
    const notifications = items as Notification[];
    const ids = notifications.map((n) => n.id);
    void notificationStore.batchDelete(ids).then(() => {
      $q.notify({ type: 'positive', message: $t('batchActions.deleteSuccess', { count: ids.length }) });
      selectedNotifications.value = [];
      void loadNotifications();
    });
  }

  function handleBatchExport(items: unknown[]) {
    const notifications = items as Notification[];
    void exportData(notifications, [], 'notifications.csv');
  }

  // ============ 初始化 ============

  async function initData() {
    await loadNotifications();
    await notificationStore.fetchUnreadCount();
  }

  return {
    // 状态
    selectedNotifications,
    filters,
    showDetailDialog,
    currentNotification,
    deleteDialogRef,
    pagination,
    // 计算属性
    typeOptionsForSearch,
    columns,
    // store（模板直接访问 notifications / isLoading / isAllRead）
    notificationStore,
    // 方法
    onTableRequest,
    handleSearch,
    handleReset,
    getTypeLabel,
    getTypeColor,
    getPriorityColor,
    formatTime,
    viewNotification,
    handleMarkRead,
    handleMarkAllRead,
    handleDelete,
    doDeleteNotification,
    handleBatchDelete,
    handleBatchExport,
    // 初始化
    initData,
  };
}
