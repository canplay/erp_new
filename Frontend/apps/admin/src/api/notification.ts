/**
 * @file notification.ts
 * @description 通知和公告 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

/**
 * @brief 通知模板接口
 */
export interface NotificationTemplate {
  id: number;
  name: string;
  code: string;
  title: string;
  content: string;
  type: 'system' | 'operation' | 'approval' | 'custom';
  channels: ('in_app' | 'email' | 'sms')[];
  variables: string[];
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 通知记录接口
 */
export interface NotificationRecord {
  id: number;
  user_id: number;
  username: string;
  templateId?: number;
  templateCode?: string;
  title: string;
  content: string;
  type: string;
  channel: 'in_app' | 'email' | 'sms';
  status: 'pending' | 'sent' | 'read' | 'failed';
  read_at?: string;
  created_at: string;
}

/**
 * @brief 公告接口
 */
export interface Announcement {
  id: number;
  title: string;
  content: string;
  type: 'normal' | 'important' | 'urgent';
  level: number;
  isPinned: boolean;
  isActive: boolean;
  start_time?: string;
  end_time?: string;
  created_by: {
    id: number;
    username: string;
  };
  created_at: string;
  updated_at: string;
}

// ============ 通知模板 API ============

/**
 * @brief 获取通知模板列表
 */
export function listNotificationTemplates(params?: {
  page?: number;
  page_size?: number;
  keyword?: string;
  type?: string;
}) {
  try {
    return await httpClient.get('/admin/notification-templates', { params });
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 获取通知模板详情
 */
export function getNotificationTemplate(id: number) {
  try {
    return await httpClient.get(`/admin/notification-templates/${id}`);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 创建通知模板
 */
export function createNotificationTemplate(data: Partial<NotificationTemplate>) {
  try {
    return await httpClient.post('/admin/notification-templates', data);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 更新通知模板
 */
export function updateNotificationTemplate(id: number, data: Partial<NotificationTemplate>) {
  try {
    return await httpClient.put(`/admin/notification-templates/${id}`, data);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 删除通知模板
 */
export function deleteNotificationTemplate(id: number) {
  try {
    return await httpClient.delete(`/admin/notification-templates/${id}`);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 切换模板启用状态
 */
export function toggleNotificationTemplate(id: number, enabled: boolean) {
  try {
    return await httpClient.put(`/admin/notification-templates/${id}/toggle`, { enabled });
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 发送测试通知
 */
export function sendTestNotification(id: number, user_id?: number) {
  try {
    return await httpClient.post(`/admin/notification-templates/${id}/test`, { user_id: user_id });
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

// ============ 通知记录 API ============

/**
 * @brief 获取通知记录列表
 */
export function listNotificationRecords(params?: {
  page?: number;
  page_size?: number;
  user_id?: number;
  type?: string;
  channel?: string;
  status?: string;
  start_date?: string;
  end_date?: string;
}) {
  try {
    return await httpClient.get('/admin/notifications', { params });
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 获取通知记录详情
 */
export function getNotificationRecord(id: number) {
  try {
    return await httpClient.get(`/admin/notifications/${id}`);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 标记通知为已读
 */
export function markNotificationAsRead(id: number) {
  try {
    return await httpClient.put(`/admin/notifications/${id}/read`);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 标记所有通知为已读
 */
export function markAllNotificationsAsRead() {
  try {
    return await httpClient.put('/admin/notifications/read-all');
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 删除通知记录
 */
export function deleteNotification(id: number) {
  try {
    return await httpClient.delete(`/admin/notifications/${id}`);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 批量删除通知
 */
export function deleteNotifications(ids: number[]) {
  try {
    return await httpClient.post('/admin/notifications/batch-delete', { ids });
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 发送通知
 */
export function sendNotification(data: {
  user_ids?: number[];
  role?: string;
  templateId?: number;
  title: string;
  content: string;
  type: string;
  channel: 'in_app' | 'email' | 'sms';
}) {
  try {
    return await httpClient.post('/admin/notifications/send', data);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

// ============ 公告 API ============

/**
 * @brief 获取公告列表
 */
export function listAnnouncements(params?: {
  page?: number;
  page_size?: number;
  keyword?: string;
  type?: string;
  isActive?: boolean;
}) {
  try {
    return await httpClient.get('/admin/announcements', { params });
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 获取公告详情
 */
export function getAnnouncement(id: number) {
  try {
    return await httpClient.get(`/admin/announcements/${id}`);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 创建公告
 */
export function createAnnouncement(data: Partial<Announcement> & {
  title: string;
  content: string;
  type: 'normal' | 'important' | 'urgent';
}) {
  try {
    return await httpClient.post('/admin/announcements', data);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 更新公告
 */
export function updateAnnouncement(id: number, data: Partial<Announcement>) {
  try {
    return await httpClient.put(`/admin/announcements/${id}`, data);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 删除公告
 */
export function deleteAnnouncement(id: number) {
  try {
    return await httpClient.delete(`/admin/announcements/${id}`);
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 切换公告置顶状态
 */
export function toggleAnnouncementPin(id: number, isPinned: boolean) {
  try {
    return await httpClient.put(`/admin/announcements/${id}/pin`, { is_pinned: isPinned });
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 切换公告激活状态
 */
export function toggleAnnouncementActive(id: number, isActive: boolean) {
  try {
    return await httpClient.put(`/admin/announcements/${id}/active`, { is_active: isActive });
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}

/**
 * @brief 获取当前生效的公告
 */
export function getActiveAnnouncements() {
  try {
    return await httpClient.get('/announcements/active');
  } catch (error) {
    handleApiError(error, '通知公告');
    throw error;
  }
}