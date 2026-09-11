/**
 * @file scheduled-task.ts
 * @description 定时任务 API 接口
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { ApiResponse } from '@/types/api';

// ============ 类型定义 ============

/** 定时任务状态 */
export type ScheduledTaskStatus = 'active' | 'paused' | 'stopped';

/** 动作类型 */
export type ActionType = 'workflow' | 'report' | 'webhook' | 'script';

/** 定时任务 */
export interface ScheduledTask {
  id: string;
  name: string;
  cronExpression: string;
  actionType: ActionType;
  actionParams: Record<string, unknown>;
  status: ScheduledTaskStatus;
  lastRunAt?: string;
  nextRunAt?: string;
  created_by: string;
  created_at: string;
  updated_at: string;
}

// ============ 请求参数 ============

/** 创建定时任务参数 */
export interface CreateScheduledTaskParam {
  name: string;
  cronExpression: string;
  actionType: ActionType;
  actionParams?: Record<string, unknown>;
}

/** 更新定时任务参数 */
export interface UpdateScheduledTaskParam {
  name?: string;
  cronExpression?: string;
  actionParams?: Record<string, unknown>;
  status?: ScheduledTaskStatus;
}

/** 列表查询参数 */
export interface ScheduledTaskListParam {
  page?: number;
  page_size?: number;
  keyword?: string;
  actionType?: ActionType;
  status?: ScheduledTaskStatus;
}

// ============ API 函数 ============

const baseUrl = '/scheduled-tasks';

/**
 * @brief 获取定时任务列表
 */
export function listScheduledTasks(params?: ScheduledTaskListParam) {
  return httpClient.get<ApiResponse<{ list: ScheduledTask[]; total: number }>>(`${baseUrl}`, { params });
}

/**
 * @brief 获取定时任务详情
 */
export function getScheduledTask(id: string) {
  return httpClient.get<ApiResponse<ScheduledTask>>(`${baseUrl}/${id}`);
}

/**
 * @brief 创建定时任务
 */
export function createScheduledTask(data: CreateScheduledTaskParam) {
  return httpClient.post<ApiResponse<{ id: string }>>(`${baseUrl}`, data);
}

/**
 * @brief 更新定时任务
 */
export function updateScheduledTask(id: string, data: UpdateScheduledTaskParam) {
  return httpClient.put<ApiResponse<void>>(`${baseUrl}/${id}`, data);
}

/**
 * @brief 删除定时任务
 */
export function deleteScheduledTask(id: string) {
  return httpClient.delete<ApiResponse<void>>(`${baseUrl}/${id}`);
}

/**
 * @brief 触发定时任务
 */
export function triggerScheduledTask(id: string) {
  return httpClient.post<ApiResponse<void>>(`${baseUrl}/${id}/trigger`);
}

/**
 * @brief 暂停定时任务
 */
export function pauseScheduledTask(id: string) {
  return httpClient.put<ApiResponse<void>>(`${baseUrl}/${id}/pause`);
}

/**
 * @brief 恢复定时任务
 */
export function resumeScheduledTask(id: string) {
  return httpClient.put<ApiResponse<void>>(`${baseUrl}/${id}/resume`);
}