/**
 * @file message.ts
 * @description 消息相关类型定义
 * @date 2026-04-06
 */

/**
 * @brief 消息类型
 */
export type MessageType = 'system' | 'user' | 'announcement';

/**
 * @brief 消息目标类型
 */
export type MessageTargetType = 'all' | 'dept' | 'role' | 'user';

/**
 * @brief 消息
 */
export interface Message {
  /** 消息ID */
  id: number;
  /** 消息标题 */
  title: string;
  /** 消息内容 */
  content: string;
  /** 消息类型 */
  type: MessageType;
  /** 发送者ID */
  sender_id?: number;
  /** 发送者名称 */
  sender_name?: string;
  /** 接收者ID */
  receiver_id?: number;
  /** 接收者名称 */
  receiver_name?: string;
  /** 优先级 */
  priority?: number;
  /** 是否已读 */
  is_read?: boolean;
  /** 是否星标 */
  is_starred?: boolean;
  /** 创建时间 */
  created_at?: string;
  /** 更新时间 */
  updated_at?: string;
}

/**
 * @brief 消息查询参数
 */
export interface MessageQueryParams {
  /** 页码 */
  page?: number;
  /** 每页数量 */
  page_size?: number;
  /** 关键词搜索 */
  keyword?: string;
  /** 消息类型 */
  type?: MessageType;
  /** 是否已读 */
  is_read?: boolean;
}

/**
 * @brief 消息创建参数
 */
export interface MessageCreateParams {
  /** 消息类型 */
  type: MessageType;
  /** 目标类型 */
  targetType: MessageTargetType;
  /** 目标ID列表 */
  target_ids?: number[];
  /** 消息标题 */
  title: string;
  /** 消息内容 */
  content: string;
  /** 优先级 */
  priority?: number;
}

/**
 * @brief 消息统计
 */
export interface MessageStatistics {
  /** 未读消息总数 */
  unreadCount: number;
  /** 收件箱总数 */
  inboxCount: number;
  /** 发件箱总数 */
  outboxCount: number;
  /** 公告总数 */
  announcementCount: number;
}
