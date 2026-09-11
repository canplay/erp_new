/**
 * @file feedback.ts
 * @description 意见反馈 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';

/**
 * @brief 反馈类型
 */
export type FeedbackType = 'suggestion' | 'bug' | 'complaint' | 'other';

/**
 * @brief 反馈状态
 */
export type FeedbackStatus = 'pending' | 'processing' | 'resolved' | 'rejected' | 'closed';

/**
 * @brief 反馈附件
 */
export interface FeedbackAttachment {
  name: string;
  url: string;
}

/**
 * @brief 客户端信息
 */
export interface ClientInfo {
  browser?: string;
  os?: string;
  resolution?: string;
  language?: string;
}

/**
 * @brief 反馈接口
 */
export interface Feedback {
  id: number;
  user_id: number;
  userName: string;
  type: FeedbackType;
  title: string;
  content: string;
  attachments?: FeedbackAttachment[];
  contact?: string;
  status: FeedbackStatus;
  handlerId?: number;
  handlerName?: string;
  handlerReply?: string;
  handlerTime?: string;
  rating?: number;
  handlerAttachments?: FeedbackAttachment[];
  clientInfo?: ClientInfo;
  pageUrl?: string;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 反馈查询参数
 */
export interface FeedbackQueryParams {
  type?: FeedbackType;
  status?: FeedbackStatus;
  keyword?: string;
  start_date?: string;
  end_date?: string;
  handlerId?: number;
  page?: number;
  page_size?: number;
}

/**
 * @brief 反馈创建参数（用户提交）
 */
export interface FeedbackCreateParams {
  type: FeedbackType;
  title: string;
  content: string;
  attachments?: FeedbackAttachment[];
  contact?: string;
  clientInfo?: ClientInfo;
  pageUrl?: string;
}

/**
 * @brief 反馈处理参数
 */
export interface FeedbackHandleParams {
  status: FeedbackStatus;
  handlerReply: string;
  handlerAttachments?: FeedbackAttachment[];
}

/**
 * @brief 获取反馈列表（管理员）
 */
export function getFeedbackList(params?: FeedbackQueryParams) {
  return httpClient.get('/admin/feedback', { params });
}

/**
 * @brief 获取反馈详情
 */
export function getFeedbackDetail(id: number) {
  return httpClient.get(`/admin/feedback/${id}`);
}

/**
 * @brief 提交反馈（用户）
 */
export function submitFeedback(data: FeedbackCreateParams) {
  return httpClient.post('/feedback', data);
}

/**
 * @brief 处理反馈（管理员）
 */
export function handleFeedback(id: number, data: FeedbackHandleParams) {
  return httpClient.put(`/admin/feedback/${id}/handle`, data);
}

/**
 * @brief 转交反馈（更换处理人）
 */
export function transferFeedback(id: number, handlerId: number) {
  return httpClient.put(`/admin/feedback/${id}/transfer`, { handlerId });
}

/**
 * @brief 添加反馈回复
 */
export function addFeedbackReply(id: number, reply: string) {
  return httpClient.post(`/admin/feedback/${id}/reply`, { reply });
}

/**
 * @brief 关闭反馈
 */
export function closeFeedback(id: number) {
  return httpClient.put(`/admin/feedback/${id}/close`);
}

/**
 * @brief 删除反馈
 */
export function deleteFeedback(id: number) {
  return httpClient.delete(`/admin/feedback/${id}`);
}

/**
 * @brief 批量处理反馈
 */
export function batchHandleFeedback(ids: number[], data: FeedbackHandleParams) {
  return httpClient.put('/admin/feedback/batch-handle', { ids, ...data });
}

/**
 * @brief 获取反馈统计数据
 */
export function getFeedbackStatistics(params?: { start_date?: string; end_date?: string }) {
  return httpClient.get('/admin/feedback/statistics', { params });
}

/**
 * @brief 获取反馈类型统计
 */
export function getFeedbackTypeStatistics(params?: { start_date?: string; end_date?: string }) {
  return httpClient.get('/admin/feedback/statistics/by-type', { params });
}

/**
 * @brief 获取处理人列表
 */
export function getFeedbackHandlers() {
  return httpClient.get('/admin/feedback/handlers');
}