/**
 * @file notification.ts
 * @description 通知和公告 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';

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
  return httpClient.get('/admin/notification-templates', { params });
}

/**
 * @brief 获取通知模板详情
 */
export function getNotificationTemplate(id: number) {
  return httpClient.get(`/admin/notification-templates/${id}`);
}

/**
 * @brief 创建通知模板
 */
export function createNotificationTemplate(data: Partial<NotificationTemplate>) {
  return httpClient.post('/admin/notification-templates', data);
}

/**
 * @brief 更新通知模板
 */
export function updateNotificationTemplate(id: number, data: Partial<NotificationTemplate>) {
  return httpClient.put(`/admin/notification-templates/${id}`, data);
}

/**
 * @brief 删除通知模板
 */
export function deleteNotificationTemplate(id: number) {
  return httpClient.delete(`/admin/notification-templates/${id}`);
}

/**
 * @brief 切换模板启用状态
 */
export function toggleNotificationTemplate(id: number, enabled: boolean) {
  return httpClient.put(`/admin/notification-templates/${id}/toggle`, { enabled });
}

/**
 * @brief 发送测试通知
 */
export function sendTestNotification(id: number, user_id?: number) {
  return httpClient.post(`/admin/notification-templates/${id}/test`, { user_id: user_id });
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
  return httpClient.get('/admin/notifications', { params });
}

/**
 * @brief 获取通知记录详情
 */
export function getNotificationRecord(id: number) {
  return httpClient.get(`/admin/notifications/${id}`);
}

/**
 * @brief 标记通知为已读
 */
export function markNotificationAsRead(id: number) {
  return httpClient.put(`/admin/notifications/${id}/read`);
}

/**
 * @brief 标记所有通知为已读
 */
export function markAllNotificationsAsRead() {
  return httpClient.put('/admin/notifications/read-all');
}

/**
 * @brief 删除通知记录
 */
export function deleteNotification(id: number) {
  return httpClient.delete(`/admin/notifications/${id}`);
}

/**
 * @brief 批量删除通知
 */
export function deleteNotifications(ids: number[]) {
  return httpClient.post('/admin/notifications/batch-delete', { ids });
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
  return httpClient.post('/admin/notifications/send', data);
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
  return httpClient.get('/admin/announcements', { params });
}

/**
 * @brief 获取公告详情
 */
export function getAnnouncement(id: number) {
  return httpClient.get(`/admin/announcements/${id}`);
}

/**
 * @brief 创建公告
 */
export function createAnnouncement(data: Partial<Announcement> & {
  title: string;
  content: string;
  type: 'normal' | 'important' | 'urgent';
}) {
  return httpClient.post('/admin/announcements', data);
}

/**
 * @brief 更新公告
 */
export function updateAnnouncement(id: number, data: Partial<Announcement>) {
  return httpClient.put(`/admin/announcements/${id}`, data);
}

/**
 * @brief 删除公告
 */
export function deleteAnnouncement(id: number) {
  return httpClient.delete(`/admin/announcements/${id}`);
}

/**
 * @brief 切换公告置顶状态
 */
export function toggleAnnouncementPin(id: number, isPinned: boolean) {
  return httpClient.put(`/admin/announcements/${id}/pin`, { is_pinned: isPinned });
}

/**
 * @brief 切换公告激活状态
 */
export function toggleAnnouncementActive(id: number, isActive: boolean) {
  return httpClient.put(`/admin/announcements/${id}/active`, { is_active: isActive });
}

/**
 * @brief 获取当前生效的公告
 */
export function getActiveAnnouncements() {
  return httpClient.get('/announcements/active');
}