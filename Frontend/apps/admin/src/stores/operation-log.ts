/**
 * @file operation-log.ts
 * @description 操作日志状态管理
 * @date 2026-04-03
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '@/utils/logger';
import {
  listOperationLogs,
  listAuditLogs,
  getOperationLog,
} from '@/api/operation-log';
import type { OperationLog, AuditLog, OperationType } from '@/types/log';

export const useOperationLogStore = defineStore('operationLog', () => {
  // ============ 状态定义 ============

  /** 操作日志列表 */
  const operationLogs = ref<OperationLog[]>([]);

  /** 审计日志列表 */
  const auditLogs = ref<AuditLog[]>([]);

  /** 加载状态 */
  const isLoading = ref(false);

  /** 分页信息 */
  const pagination = ref({
    page: 1,
    page_size: 20,
    total: 0,
  });

  /** 筛选条件 */
  const filters = ref<{
    keyword: string;
    username: string;
    module: string;
    operation_type: OperationType | null;
    success: boolean | null;
    start_date: string;
    end_date: string;
  }>({
    keyword: '',
    username: '',
    module: '',
    operation_type: null,
    success: null,
    start_date: '',
    end_date: '',
  });

  // ============ 计算属性 ============

  /** 危险操作（如删除） */
  const dangerousLogs = computed(() =>
    operationLogs.value.filter((log) => log.operation_type === 'delete' || !log.success)
  );

  // ============ Actions ============

  /**
   * @brief 获取操作日志列表
   */
  async function fetchOperationLogs() {
    isLoading.value = true;
    try {
      const params: {
        page: number;
        page_size: number;
        keyword?: string;
        username?: string;
        module?: string;
        operation_type?: OperationType;
        success?: boolean;
        start_date?: string;
        end_date?: string;
      } = {
        page: pagination.value.page,
        page_size: pagination.value.page_size,
      };
      if (filters.value.keyword) params.keyword = filters.value.keyword;
      if (filters.value.username) params.username = filters.value.username;
      if (filters.value.module) params.module = filters.value.module;
      if (filters.value.operation_type !== null) params.operation_type = filters.value.operation_type ?? undefined;
      if (filters.value.success !== null) params.success = filters.value.success ?? undefined;
      if (filters.value.start_date) params.start_date = filters.value.start_date;
      if (filters.value.end_date) params.end_date = filters.value.end_date;
      const response = await listOperationLogs(params);

      const data = (response as unknown as { data?: { list?: OperationLog[]; total?: number } }).data;
      if (data) {
        operationLogs.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取操作日志失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取审计日志列表
   */
  async function fetchAuditLogs() {
    isLoading.value = true;
    try {
      const params: {
        page: number;
        page_size: number;
        keyword?: string;
        username?: string;
        resource_type?: string;
        action?: string;
        approval_status?: string;
        start_date?: string;
        end_date?: string;
      } = {
        page: pagination.value.page,
        page_size: pagination.value.page_size,
      };
      if (filters.value.keyword) params.keyword = filters.value.keyword;
      if (filters.value.username) params.username = filters.value.username;
      if (filters.value.start_date) params.start_date = filters.value.start_date;
      if (filters.value.end_date) params.end_date = filters.value.end_date;
      const response = await listAuditLogs(params);

      const data = (response as unknown as { data?: { list?: AuditLog[]; total?: number } }).data;
      if (data) {
        auditLogs.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【获取审计日志失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 获取操作日志详情
   * @param id - 日志ID
   */
  async function fetchOperationLogDetail(id: number) {
    try {
      const response = await getOperationLog(id);
      return (response as unknown as { data?: OperationLog }).data;
    } catch (error) {
      logger.error('【获取操作日志详情失败】', error);
      throw error;
    }
  }

  /**
   * @brief 设置筛选条件
   */
  function setFilters(newFilters: Partial<typeof filters.value>) {
    filters.value = { ...filters.value, ...newFilters };
    pagination.value.page = 1;
  }

  /**
   * @brief 重置筛选
   */
  function resetFilters() {
    filters.value = {
      keyword: '',
      username: '',
      module: '',
      operation_type: null,
      success: null,
      start_date: '',
      end_date: '',
    };
    pagination.value.page = 1;
  }

  return {
    // 状态
    operationLogs,
    auditLogs,
    isLoading,
    pagination,
    filters,

    // 计算属性
    dangerousLogs,

    // 方法
    fetchOperationLogs,
    fetchAuditLogs,
    fetchOperationLogDetail,
    setFilters,
    resetFilters,
  };
});

