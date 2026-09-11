/**
 * @file socket.ts
 * @description WebSocket 类型定义
 * @date 2026-04-04
 */

/**
 * @brief Socket.io 客户端事件
 */
export interface ClientToServerEvents {
  /** 加入房间 */
  'join-room': (room: string) => void;
  /** 离开房间 */
  'leave-room': (room: string) => void;
  /** 订阅通知 */
  'subscribe:notification': () => void;
  /** 取消订阅通知 */
  'unsubscribe:notification': () => void;
  /** 订阅监控 */
  'subscribe:monitor': () => void;
  /** 取消订阅监控 */
  'unsubscribe:monitor': () => void;
  /** 订阅公告 */
  'subscribe:announcement': () => void;
  /** 取消订阅公告 */
  'unsubscribe:announcement': () => void;
  /** 订阅在线用户 */
  'subscribe:online-users': () => void;
  /** 取消订阅在线用户 */
  'unsubscribe:online-users': () => void;
  /** 发送消息给指定用户 */
  'message:to-user': (user_id: number, message: string) => void;
}

/**
 * @brief Socket.io 服务端事件
 */
export interface ServerToClientEvents {
  /** 新通知 */
  'notification:new': (data: NotificationPayload) => void;
  /** 通知已读 */
  'notification:read': (data: { id: number }) => void;
  /** 新公告 */
  'announcement:new': (data: AnnouncementPayload) => void;
  /** 公告更新 */
  'announcement:update': (data: AnnouncementPayload) => void;
  /** 监控数据更新 */
  'monitor:stats': (data: MonitorStats) => void;
  /** 监控告警 */
  'monitor:alert': (data: MonitorAlert) => void;
  /** 用户上线 */
  'online:user-joined': (data: OnlineUser) => void;
  /** 用户下线 */
  'online:user-left': (data: OnlineUser) => void;
  /** 在线人数更新 */
  'online:count-update': (data: { count: number }) => void;
  /** 系统消息 */
  'system:message': (data: SystemMessage) => void;
  /** 错误消息 */
  'error': (data: { message: string; code?: string }) => void;
}

/**
 * @brief 通知载荷
 */
export interface NotificationPayload {
  id: number;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  content: string;
  from?: {
    id: number;
    username: string;
    avatar?: string;
  };
  link?: string;
  created_at: string;
  read?: boolean;
}

/**
 * @brief 公告载荷
 */
export interface AnnouncementPayload {
  id: number;
  title: string;
  content: string;
  type: 'normal' | 'important' | 'urgent';
  level: number;
  publishedAt: string;
  expires_at?: string;
  author: {
    id: number;
    username: string;
  };
  read?: boolean;
}

/**
 * @brief 监控数据
 */
export interface MonitorStats {
  cpu: number;
  memory: number;
  disk: number;
  network: {
    inbound: number;
    outbound: number;
  };
  response_time: number;
  requestCount: number;
  errorRate: number;
  timestamp: string;
}

/**
 * @brief 监控告警
 */
export interface MonitorAlert {
  id: string;
  level: 'info' | 'warning' | 'error' | 'critical';
  metric: string;
  message: string;
  value: number;
  threshold: number;
  timestamp: string;
}

/**
 * @brief 在线用户信息
 */
export interface OnlineUser {
  id: number;
  username: string;
  avatar?: string;
  role: string;
  loginAt: string;
  lastActivity: string;
  ip?: string;
  device?: string;
}

/**
 * @brief 系统消息
 */
export interface SystemMessage {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  content: string;
  timestamp: string;
}

/**
 * @brief Socket 连接状态
 */
export type SocketStatus = 'disconnected' | 'connecting' | 'connected' | 'reconnecting' | 'error';

/**
 * @brief Socket 选项
 */
export interface SocketOptions {
  /** 服务器地址 */
  url?: string;
  /** 自动连接 */
  autoConnect?: boolean;
  /** 自动重连 */
  autoReconnect?: boolean;
  /** 重连间隔（毫秒） */
  reconnectInterval?: number;
  /** 最大重连次数 */
  maxReconnectAttempts?: number;
  /** 认证 token */
  token?: string;
}

/**
 * @brief 房间类型
 */
export type SocketRoom = 'notification' | 'monitor' | 'announcement' | 'online-users';
