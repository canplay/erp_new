/**
 * @file tenant.ts
 * @description 租户管理 API
 * @date 2026-05-17
 * @description 2026-05-17 更新：统一使用 httpClient
 */

import { httpClient } from '@/utils/alova';
import type { Tenant, TenantUser } from '@/types/tenant';

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
  return httpClient.get('/admin/tenants', { params });
}

/**
 * @brief 获取租户详情（管理员）
 */
export function getTenantById(id: number) {
  return httpClient.get(`/admin/tenants/${id}`);
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
  return httpClient.post('/admin/tenants', data);
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
  return httpClient.put(`/admin/tenants/${id}`, data);
}

/**
 * @brief 删除租户（管理员）
 */
export function deleteTenant(id: number) {
  return httpClient.delete(`/admin/tenants/${id}`);
}

/**
 * @brief 启用租户（管理员）
 */
export function enableTenant(id: number) {
  return httpClient.put(`/admin/tenants/${id}/enable`);
}

/**
 * @brief 禁用租户（管理员）
 */
export function disableTenant(id: number) {
  return httpClient.put(`/admin/tenants/${id}/disable`);
}

/**
 * @brief 重置租户配额（管理员）
 */
export function resetTenantQuota(id: number) {
  return httpClient.post(`/admin/tenants/${id}/reset-quota`);
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
  return httpClient.get(`/admin/tenants/${tenantId}/users`, { params });
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
  return httpClient.post(`/admin/tenants/${tenantId}/users`, data);
}

/**
 * @brief 移除租户用户（管理员）
 */
export function removeTenantUserAdmin(tenantId: number, user_id: number) {
  return httpClient.delete(`/admin/tenants/${tenantId}/users/${user_id}`);
}

/**
 * @brief 根据邮箱搜索用户
 */
export function searchUsersByEmail(email: string) {
  return httpClient.get('/admin/users/search', { params: { email } });
}

/**
 * @brief 获取租户套餐列表
 */
export function getTenantPlans() {
  return httpClient.get('/admin/tenant-plans');
}

// ============ 用户端 API ============

/**
 * @brief 获取当前租户信息
 */
export function getCurrentTenant() {
  return httpClient.get('/tenant/current');
}

/**
 * @brief 更新租户信息
 */
export function updateTenant(id: number, data: Partial<Tenant>) {
  return httpClient.put(`/tenant/${id}`, data);
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
  return httpClient.get('/tenant/users', { params });
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
  return httpClient.post('/tenant/users', data);
}

/**
 * @brief 更新租户用户
 */
export function updateTenantUser(user_id: number, data: Partial<TenantUser>) {
  return httpClient.put(`/tenant/users/${user_id}`, data);
}

/**
 * @brief 移除租户用户
 */
export function removeTenantUser(user_id: number) {
  return httpClient.delete(`/tenant/users/${user_id}`);
}

/**
 * @brief 切换租户
 */
export function switchTenant(tenantId: number) {
  return httpClient.post('/tenant/switch', { tenant_id: tenantId });
}

/**
 * @brief 获取套餐列表
 */
export function listPlans() {
  return httpClient.get('/tenant/plans');
}

/**
 * @brief 获取当前套餐
 */
export function getCurrentPlan() {
  return httpClient.get('/tenant/plan/current');
}

/**
 * @brief 升级/续费套餐
 */
export function upgradePlan(planId: string, interval: 'month' | 'year') {
  return httpClient.post('/tenant/plan/upgrade', { plan_id: planId, interval });
}

/**
 * @brief 获取使用统计
 */
export function getUsageStats() {
  return httpClient.get('/tenant/usage');
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
  return httpClient.get('/tenant/audit-logs', { params });
}

/**
 * @brief 导出审计日志
 */
export function exportAuditLogs(params: {
  start_date: string;
  end_date: string;
  format: 'csv' | 'xlsx';
}) {
  return httpClient.get('/tenant/audit-logs/export', {
    params,
    responseType: 'blob',
  });
}