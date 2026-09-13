/**
 * @file notification.ts
 * @description 通知相关类型定义
 * @date 2026-04-03
 */

/**
 * @brief 通知类型
 */
export type NotificationType = 'system' | 'operation' | 'approval';

/**
 * @brief 通知优先级
 */
export type NotificationPriority = 'low' | 'normal' | 'high' | 'urgent';

/**
 * @brief 通知
 */
export interface Notification {
  id: number;
  type: NotificationType;
  title: string;
  content: string;
  priority: NotificationPriority;
  is_read: boolean;
  related_id?: number;
  related_type?: string;
  created_at: string;
  read_at?: string;
}

/**
 * @brief 通知查询参数
 */
export interface NotificationParams {
  page?: number;
  page_size?: number;
  type?: NotificationType;
  is_read?: boolean;
  start_date?: string;
  end_date?: string;
}

/**
 * @brief WebSocket 通知消息
 */
export interface NotificationMessage {
  type: 'notification';
  data: Notification;
}

/**
 * @brief WebSocket 消息类型联合
 */
export type WebSocketMessage = NotificationMessage;
