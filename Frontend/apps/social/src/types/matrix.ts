/**
 * Matrix 协议核心类型定义
 * 基于 Matrix Client-Server API v1.12
 * NOTE: 此文件为备用类型定义，实际类型以 matrix-js-sdk 为准
 */

import { NotificationCountType, type MatrixEvent, type Room, type RoomMember } from 'matrix-js-sdk';
import type { AuthCredentials as AuthCreds } from '@/services/matrix-types';

/** 认证信息凭证（从 services/matrix 引用） */
export type AuthCredentials = AuthCreds;

/** 登录表单数据 */
export interface LoginParams {
  username: string;
  password: string;
  initialDeviceDisplayName?: string;
}

/** 注册表单数据 */
export interface RegisterParams {
  username: string;
  password: string;
  initialDeviceDisplayName?: string;
}

/** 房间摘要信息（用于列表展示） */
export interface RoomSummary {
  roomId: string;
  name: string;
  topic?: string | undefined;
  avatarUrl?: string | undefined;
  memberCount: number;
  lastMessage?: MessageSummary | undefined;
  unreadCount: number;
  isDirect: boolean;
}

/** 消息摘要（用于房间列表预览） */
export interface MessageSummary {
  body: string;
  sender: string;
  timestamp: number;
  eventType: string;
}

/** 聊天消息 */
export interface ChatMessage {
  eventId: string;
  roomId: string;
  sender: string;
  senderName: string;
  senderAvatar?: string | undefined;
  body: string;
  timestamp: number;
  contentType: MessageContentType;
  content?: Record<string, unknown> | undefined;
  status: MessageStatus;
}

/** 消息内容类型 */
export type MessageContentType = 'text' | 'image' | 'audio' | 'video' | 'file' | 'emote' | 'notice';

/** 消息状态 */
export type MessageStatus = 'sending' | 'sent' | 'failed';

/** 联系人信息 */
export interface Contact {
  userId: string;
  displayName?: string;
  avatarUrl?: string;
  statusMsg?: string;
  presence?: 'online' | 'offline' | 'unavailable';
  lastActive?: number;
}

/** 朋友圈动态 */
export interface Moment {
  id: string;
  author: Contact;
  content: string;
  images?: string[];
  timestamp: number;
  likes: number;
  liked: boolean;
  comments: Comment[];
}

/** 评论 */
export interface Comment {
  id: string;
  author: Contact;
  content: string;
  timestamp: number;
}

/** 语音消息数据 */
export interface VoiceData {
  url: string;
  duration: number;
  mimeType: string;
}

/** 视频通话状态 */
export type CallState = 'idle' | 'calling' | 'ringing' | 'connected' | 'ended' | 'failed';

/** 同步状态 */
export type SyncState = 'syncing' | 'synced' | 'error' | 'stopped' | 'reconnecting';

/**
 * 将 Matrix Room 转换为 RoomSummary
 */
export function roomToSummary(room: Room): RoomSummary {
  const timeline = room.getLiveTimeline();
  const events = timeline.getEvents();
  const lastEvent = events.length > 0 ? events[events.length - 1] : undefined;

  let lastMessage: MessageSummary | undefined;
  if (lastEvent && lastEvent.getType() === 'm.room.message') {
    const content = lastEvent.getContent<Record<string, unknown>>();
    lastMessage = {
      body: (content?.body as string) || '',
      sender: lastEvent.getSender() || '',
      timestamp: lastEvent.getTs(),
      eventType: lastEvent.getType(),
    };
  }

  return {
    roomId: room.roomId,
    name: room.name || room.getDefaultRoomName(room.getMyMembership()) || room.roomId,
    topic: undefined,
    avatarUrl: room.getMxcAvatarUrl() ?? undefined,
    memberCount: room.getJoinedMemberCount(),
    lastMessage,
    unreadCount: room.getUnreadNotificationCount(NotificationCountType.Total),
    isDirect: room.isElementVideoRoom(),
  };
}

/**
 * 将 MatrixEvent 转换为 ChatMessage
 */
export function eventToMessage(
  event: MatrixEvent,
  member?: RoomMember,
): ChatMessage {
  const content = event.getContent<Record<string, unknown>>();
  const msgtype = content?.msgtype as string | undefined;
  const body = (content?.body as string) ?? '';

  let contentType: MessageContentType = 'text';
  if (msgtype === 'm.image') contentType = 'image';
  else if (msgtype === 'm.audio') contentType = 'audio';
  else if (msgtype === 'm.video') contentType = 'video';
  else if (msgtype === 'm.file') contentType = 'file';
  else if (msgtype === 'm.emote') contentType = 'emote';
  else if (msgtype === 'm.notice') contentType = 'notice';

  return {
    eventId: event.getId() || '',
    roomId: event.getRoomId() || '',
    sender: event.getSender() || '',
    senderName: member?.name || event.getSender() || '',
    senderAvatar: member?.getMxcAvatarUrl() ?? undefined,
    body,
    timestamp: event.getTs(),
    contentType,
    content: content,
    status: 'sent',
  };
}
