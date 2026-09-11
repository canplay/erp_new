/**
 * @file notification.ts
 * @description 通知状态管理
 * @date 2026-04-03
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '@/utils/logger';
import {
  listNotificationRecords,
  markNotificationAsRead,
  markAllNotificationsAsRead,
  deleteNotification,
  deleteNotifications,
} from '@/api/notification';
import type { Notification, NotificationType } from '@/types/notification';

export const useNotificationStore = defineStore('notification', () => {
  // ============ 状态定义 ============

  /** 通知列表 */
  const notifications = ref<Notification[]>([]);

  /** 未读数量 */
  const unreadCount = ref(0);

  /** 加载状态 */
  const isLoading = ref(false);

  /** 分页信息 */
  const pagination = ref({
    page: 1,
    page_size: 20,
    total: 0,
  });

  /** 筛选条件 */
  const filters = ref<{
    type: NotificationType | null;
    is_read: boolean | null;
  }>({
    type: null,
    is_read: null,
  });

  // ============ 计算属性 ============

  /** 是否全部已读 */
  const isAllRead = computed(() => unreadCount.value === 0);

  /** 最近通知（最新5条） */
  const recentNotifications = computed(() => notifications.value.slice(0, 5));

  // ============ Actions ============

  /**
   * @brief 获取通知列表
   */
  async function fetchNotifications() {
    isLoading.value = true;
    try {
      const params: {
        page: number;
        page_size: number;
        type?: NotificationType;
        is_read?: boolean;
      } = {
        page: pagination.value.page,
        page_size: pagination.value.page_size,
      };
      if (filters.value.type !== null) {
        params.type = filters.value.type ?? undefined;
      }
      if (filters.value.is_read !== null) {
        params.is_read = filters.value.is_read ?? undefined;
      }
      const response = await listNotificationRecords(params);

      const data = (response as { data?: { list: Notification[]; total: number } }).data;
      if (data) {
        notifications.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取通知列表失败】', error);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取未读数量
   */
  async function fetchUnreadCount() {
    try {
      const response = await listNotificationRecords({ page_size: 1 });
      unreadCount.value = (response as { data?: { total?: number } }).data?.total || 0;
    } catch (error) {
      logger.error('【获取未读数量失败】', error);
    }
  }

  /**
   * @brief 标记单条已读
   * @param id - 通知ID
   */
  async function markAsRead(id: number) {
    try {
      await markNotificationAsRead(id);
      const notification = notifications.value.find((n) => n.id === id);
      if (notification && !notification.is_read) {
        notification.is_read = true;
        notification.read_at = new Date().toISOString();
        unreadCount.value = Math.max(0, unreadCount.value - 1);
      }
    } catch (error) {
      logger.error('【标记已读失败】', error);
      throw error;
    }
  }

  /**
   * @brief 全部标记已读
   */
  async function markAllAsRead() {
    try {
      await markAllNotificationsAsRead();
      notifications.value.forEach((n) => {
        n.is_read = true;
        n.read_at = new Date().toISOString();
      });
      unreadCount.value = 0;
    } catch (error) {
      logger.error('【全部标记已读失败】', error);
      throw error;
    }
  }

  /**
   * @brief 删除通知
   * @param id - 通知ID
   */
  async function deleteNotificationById(id: number) {
    try {
      await deleteNotification(id);
      const index = notifications.value.findIndex((n) => n.id === id);
      if (index !== -1) {
        const notification = notifications.value[index];
        if (notification && !notification.is_read) {
          unreadCount.value = Math.max(0, unreadCount.value - 1);
        }
        notifications.value.splice(index, 1);
        pagination.value.total = Math.max(0, pagination.value.total - 1);
      }
    } catch (error) {
      logger.error('【删除通知失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量删除
   * @param ids - 通知ID数组
   */
  async function batchDelete(ids: number[]) {
    try {
      await deleteNotifications(ids);
      const deletedSet = new Set(ids);
      notifications.value = notifications.value.filter((n) => {
        if (deletedSet.has(n.id) && !n.is_read) {
          unreadCount.value = Math.max(0, unreadCount.value - 1);
          return false;
        }
        return !deletedSet.has(n.id);
      });
      pagination.value.total = Math.max(0, pagination.value.total - ids.length);
    } catch (error) {
      logger.error('【批量删除失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量标记已读
   * @param ids - 通知ID数组
   */
  async function batchMarkRead(ids: number[]) {
    try {
      // 使用 markNotificationAsRead 逐条标记
      await Promise.all(ids.map(id => markNotificationAsRead(id)));
      const idSet = new Set(ids);
      let count = 0;
      notifications.value.forEach((n) => {
        if (idSet.has(n.id) && !n.is_read) {
          n.is_read = true;
          n.read_at = new Date().toISOString();
          count++;
        }
      });
      unreadCount.value = Math.max(0, unreadCount.value - count);
    } catch (error) {
      logger.error('【批量标记已读失败】', error);
      throw error;
    }
  }

  /**
   * @brief 添加新通知（WebSocket 推送时调用）
   * @param notification - 新通知
   */
  function addNotification(notification: Notification) {
    notifications.value.unshift(notification);
    if (!notification.is_read) {
      unreadCount.value++;
    }
    pagination.value.total++;
  }

  /**
   * @brief 设置筛选条件
   */
  function setFilters(newFilters: Partial<typeof filters.value>) {
    filters.value = { ...filters.value, ...newFilters };
    pagination.value.page = 1;
  }

  /**
   * @brief 重置筛选
   */
  function resetFilters() {
    filters.value = { type: null, is_read: null };
    pagination.value.page = 1;
  }

  return {
    // 状态
    notifications,
    unreadCount,
    isLoading,
    pagination,
    filters,

    // 计算属性
    isAllRead,
    recentNotifications,

    // 方法
    fetchNotifications,
    fetchUnreadCount,
    markAsRead,
    markAllAsRead,
    deleteNotification: deleteNotificationById,
    batchDelete,
    batchMarkRead,
    addNotification,
    setFilters,
    resetFilters,
  };
});



