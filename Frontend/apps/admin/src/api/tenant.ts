/**
 * @file tenant.ts
 * @description 租户管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { Tenant, TenantUser } from '@/types/tenant';
import { handleApiError } from '@/utils/apiErrorHandler';

// ============ 管理员 API ============

/**
 * @brief 获取所有租户列表（管理员）
 */
export function listAllTenants(params?: {
  page?: number;
  page_size?: number;
  keyword?: string;
  status?: string;
  plan?: string;
}) {
  try {
    return await httpClient.get('/admin/tenants', { params });
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 获取租户详情（管理员）
 */
export function getTenantById(id: number) {
  try {
    return await httpClient.get(`/admin/tenants/${id}`);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 创建租户（管理员）
 */
export function createTenant(data: {
  name: string;
  code: string;
  domain?: string;
  plan: string;
  adminEmail: string;
  adminName?: string;
  maxUsers?: number;
  maxStorage?: number;
  expires_at?: string;
}) {
  try {
    return await httpClient.post('/admin/tenants', data);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 更新租户（管理员）
 */
export function updateTenantAdmin(id: number, data: {
  name?: string;
  domain?: string;
  plan?: string;
  status?: string;
  maxUsers?: number;
  maxStorage?: number;
  expires_at?: string;
}) {
  try {
    return await httpClient.put(`/admin/tenants/${id}`, data);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 删除租户（管理员）
 */
export function deleteTenant(id: number) {
  try {
    return await httpClient.delete(`/admin/tenants/${id}`);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 启用租户（管理员）
 */
export function enableTenant(id: number) {
  try {
    return await httpClient.put(`/admin/tenants/${id}/enable`);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 禁用租户（管理员）
 */
export function disableTenant(id: number) {
  try {
    return await httpClient.put(`/admin/tenants/${id}/disable`);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 重置租户配额（管理员）
 */
export function resetTenantQuota(id: number) {
  try {
    return await httpClient.post(`/admin/tenants/${id}/reset-quota`);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 获取租户用户列表（管理员）
 */
export function listTenantUsersAdmin(tenantId: number, params?: {
  page?: number;
  page_size?: number;
  role?: string;
  keyword?: string;
}) {
  try {
    return await httpClient.get(`/admin/tenants/${tenantId}/users`, { params });
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 添加租户用户（管理员）
 */
export function addTenantUserAdmin(tenantId: number, data: {
  user_id: number;
  role: string;
  department?: string;
  position?: string;
}) {
  try {
    return await httpClient.post(`/admin/tenants/${tenantId}/users`, data);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 移除租户用户（管理员）
 */
export function removeTenantUserAdmin(tenantId: number, user_id: number) {
  try {
    return await httpClient.delete(`/admin/tenants/${tenantId}/users/${user_id}`);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 根据邮箱搜索用户
 */
export function searchUsersByEmail(email: string) {
  try {
    return await httpClient.get('/admin/users/search', { params: { email } });
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 获取租户套餐列表
 */
export function getTenantPlans() {
  try {
    return await httpClient.get('/admin/tenant-plans');
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

// ============ 用户端 API ============

/**
 * @brief 获取当前租户信息
 */
export function getCurrentTenant() {
  try {
    return await httpClient.get('/tenant/current');
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 更新租户信息
 */
export function updateTenant(id: number, data: Partial<Tenant>) {
  try {
    return await httpClient.put(`/tenant/${id}`, data);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 获取租户用户列表
 */
export function listTenantUsers(params?: {
  page?: number;
  page_size?: number;
  role?: string;
  keyword?: string;
}) {
  try {
    return await httpClient.get('/tenant/users', { params });
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 添加租户用户
 */
export function addTenantUser(data: {
  user_id: number;
  role: string;
  department?: string;
  position?: string;
}) {
  try {
    return await httpClient.post('/tenant/users', data);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 更新租户用户
 */
export function updateTenantUser(user_id: number, data: Partial<TenantUser>) {
  try {
    return await httpClient.put(`/tenant/users/${user_id}`, data);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 移除租户用户
 */
export function removeTenantUser(user_id: number) {
  try {
    return await httpClient.delete(`/tenant/users/${user_id}`);
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 切换租户
 */
export function switchTenant(tenantId: number) {
  try {
    return await httpClient.post('/tenant/switch', { tenant_id: tenantId });
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 获取套餐列表
 */
export function listPlans() {
  try {
    return await httpClient.get('/tenant/plans');
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 获取当前套餐
 */
export function getCurrentPlan() {
  try {
    return await httpClient.get('/tenant/plan/current');
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 升级/续费套餐
 */
export function upgradePlan(planId: string, interval: 'month' | 'year') {
  try {
    return await httpClient.post('/tenant/plan/upgrade', { plan_id: planId, interval });
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 获取使用统计
 */
export function getUsageStats() {
  try {
    return await httpClient.get('/tenant/usage');
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 获取审计日志
 */
export function getAuditLogs(params?: {
  page?: number;
  page_size?: number;
  action?: string;
  user_id?: number;
  start_date?: string;
  end_date?: string;
}) {
  try {
    return await httpClient.get('/tenant/audit-logs', { params });
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}

/**
 * @brief 导出审计日志
 */
export function exportAuditLogs(params: {
  start_date: string;
  end_date: string;
  format: 'csv' | 'xlsx';
}) {
  try {
    return await httpClient.get('/tenant/audit-logs/export', {
    params,
    responseType: 'blob',  });
  } catch (error) {
    handleApiError(error, '租户管理');
    throw error;
  }
}