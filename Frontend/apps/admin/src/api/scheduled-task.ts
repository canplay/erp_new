/**
 * @file scheduled-task.ts
 * @description 定时任务 API 接口
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { ApiResponse } from '@/types/api';
import { handleApiError } from '@/utils/apiErrorHandler';

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
  try {
    return await httpClient.get(`${baseUrl}`, { params });
  } catch (error) {
    handleApiError(error, '定时任务');
    throw error;
  }
}

/**
 * @brief 获取定时任务详情
 */
export function getScheduledTask(id: string) {
  try {
    return await httpClient.get(`${baseUrl}/${id}`);
  } catch (error) {
    handleApiError(error, '定时任务');
    throw error;
  }
}

/**
 * @brief 创建定时任务
 */
export function createScheduledTask(data: CreateScheduledTaskParam) {
  try {
    return await httpClient.post(`${baseUrl}`, data);
  } catch (error) {
    handleApiError(error, '定时任务');
    throw error;
  }
}

/**
 * @brief 更新定时任务
 */
export function updateScheduledTask(id: string, data: UpdateScheduledTaskParam) {
  try {
    return await httpClient.put(`${baseUrl}/${id}`, data);
  } catch (error) {
    handleApiError(error, '定时任务');
    throw error;
  }
}

/**
 * @brief 删除定时任务
 */
export function deleteScheduledTask(id: string) {
  try {
    return await httpClient.delete(`${baseUrl}/${id}`);
  } catch (error) {
    handleApiError(error, '定时任务');
    throw error;
  }
}

/**
 * @brief 触发定时任务
 */
export function triggerScheduledTask(id: string) {
  try {
    return await httpClient.post(`${baseUrl}/${id}/trigger`);
  } catch (error) {
    handleApiError(error, '定时任务');
    throw error;
  }
}

/**
 * @brief 暂停定时任务
 */
export function pauseScheduledTask(id: string) {
  try {
    return await httpClient.put(`${baseUrl}/${id}/pause`);
  } catch (error) {
    handleApiError(error, '定时任务');
    throw error;
  }
}

/**
 * @brief 恢复定时任务
 */
export function resumeScheduledTask(id: string) {
  try {
    return await httpClient.put(`${baseUrl}/${id}/resume`);
  } catch (error) {
    handleApiError(error, '定时任务');
    throw error;
  }
}