/**
 * @file message.ts
 * @description 站内信 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

/**
 * @brief 消息类型
 */
export type MessageType = 'system' | 'user' | 'announcement';

/**
 * @brief 消息优先级
 */
export type MessagePriority = 0 | 1 | 2;

/**
 * @brief 发送目标类型
 */
export type MessageTargetType = 'all' | 'dept' | 'role' | 'user';

/**
 * @brief 消息接口
 */
export interface Message {
  id: number;
  type: MessageType;
  title: string;
  content: string;
  sender_id?: number;
  sender_name?: string;
  receiver_id?: number;
  receiver_name?: string;
  priority: MessagePriority;
  attachmentUrls?: string[];
  targetType: MessageTargetType;
  target_ids?: number[];
  expireTime?: string;
  created_at: string;
  updated_at: string;
  /** 是否已读 */
  is_read?: boolean;
  /** 是否星标 */
  is_starred?: boolean;
}

/**
 * @brief 用户消息关联接口
 */
export interface MessageUser {
  id: number;
  messageId: number;
  user_id: number;
  is_read: boolean;
  readTime?: string;
  is_starred: boolean;
  isDeleted: boolean;
  isArchived: boolean;
  created_at: string;
}

/**
 * @brief 消息查询参数
 */
export interface MessageQueryParams {
  type?: MessageType;
  priority?: MessagePriority;
  is_read?: boolean;
  is_starred?: boolean;
  keyword?: string;
  start_date?: string;
  end_date?: string;
  page?: number;
  page_size?: number;
}

/**
 * @brief 消息创建参数
 */
export interface MessageCreateParams {
  type: MessageType;
  title: string;
  content: string;
  priority?: MessagePriority;
  attachmentUrls?: string[];
  targetType: MessageTargetType;
  target_ids?: number[];
  expireTime?: string;
}

/**
 * @brief 获取消息列表
 */
export async function getMessageList(params?: MessageQueryParams) {
  try {
    return await httpClient.get('/messages', { params });
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 获取收件箱消息
 */
export async function getInboxMessages(params?: Omit<MessageQueryParams, 'type'>) {
  try {
    return await httpClient.get('/messages/inbox', { params });
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 获取发件箱消息
 */
export async function getOutboxMessages(params?: Omit<MessageQueryParams, 'type'>) {
  try {
    return await httpClient.get('/messages/outbox', { params });
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 获取系统公告
 */
export async function getAnnouncements(params?: Omit<MessageQueryParams, 'type'>) {
  try {
    return await httpClient.get('/messages/announcements', { params });
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 获取消息详情
 */
export async function getMessageDetail(id: number) {
  try {
    return await httpClient.get(`/messages/${id}`);
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 发送消息
 */
export async function sendMessage(data: MessageCreateParams) {
  try {
    return await httpClient.post('/messages', data);
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 批量发送消息
 */
export async function batchSendMessages(messageIds: number[], user_ids: number[]) {
  try {
    return await httpClient.post('/messages/batch-send', { messageIds, user_ids });
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 标记消息已读
 */
export async function markAsRead(messageId: number) {
  try {
    return await httpClient.put(`/messages/${messageId}/read`);
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 批量标记已读
 */
export async function batchMarkAsRead(messageIds: number[]) {
  try {
    return await httpClient.put('/messages/read/batch', { messageIds });
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 标记所有消息已读
 */
export async function markAllAsRead() {
  try {
    return await httpClient.put('/messages/read/all');
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 标记星标
 */
export async function markAsStarred(messageId: number, starred: boolean) {
  try {
    return await httpClient.put(`/messages/${messageId}/star`, { starred });
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 删除消息（软删除）
 */
export async function deleteMessage(messageId: number) {
  try {
    return await httpClient.delete(`/messages/${messageId}`);
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 批量删除消息
 */
export async function batchDeleteMessages(messageIds: number[]) {
  try {
    return await httpClient.delete('/messages/batch', { data: { messageIds } });
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 获取未读消息数量
 */
export async function getUnreadCount() {
  try {
    return await httpClient.get('/messages/unread-count');
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}

/**
 * @brief 获取未读公告数量
 */
export async function getUnreadAnnouncementCount() {
  try {
    return await httpClient.get('/messages/announcements/unread-count');
  } catch (error) {
    handleApiError(error, '站内信');
    throw error;
  }
}