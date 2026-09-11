/**
 * @file tenant.ts
 * @description 多租户相关类型定义
 * @date 2026-04-04
 */

/**
 * @brief 租户信息
 */
export interface Tenant {
  id: number;
  name: string;
  code: string;
  logo?: string;
  domain?: string;
  plan: 'free' | 'basic' | 'professional' | 'enterprise';
  status: 'active' | 'suspended' | 'trial' | 'disabled';
  expires_at?: string;
  user_count: number;
  maxUsers: number;
  storageUsed: number;
  storageLimit: number;
  settings: TenantSettings;
  created_at: string;
  updated_at: string;
}

/**
 * @brief 租户设置
 */
export interface TenantSettings {
  theme?: string;
  language?: string;
  timezone?: string;
  allow_register?: boolean;
  require_email_verify?: boolean;
  allow_oauth?: boolean;
  custom_fields?: Record<string, unknown>;
}

/**
 * @brief 租户用户
 */
export interface TenantUser {
  id: number;
  tenant_id: number;
  user_id: number;
  username: string;
  email: string;
  role: 'owner' | 'admin' | 'member' | 'guest';
  department?: string;
  position?: string;
  joinedAt: string;
  last_active_at?: string;
}

/**
 * @brief 套餐信息
 */
export interface Plan {
  id: string;
  name: string;
  price: number;
  interval: 'month' | 'year';
  features: PlanFeature[];
  limits: PlanLimits;
}

/**
 * @brief 套餐功能
 */
export interface PlanFeature {
  key: string;
  name: string;
  enabled: boolean;
}

/**
 * @brief 使用统计
 */
export interface TenantUsage {
  users?: { used: number; limit: number };
  storage?: { used: number; limit: number };
  apiCalls?: { used: number; limit: number };
  // 修复 (2026-08-07): 后端实际返回扁平结构(tenant_routes get_usage_stats)
  users_count?: number;
  storage_used?: number;
  tenant_id?: number;
}

/**
 * @brief 套餐限制
 */
export interface PlanLimits {
  users: number;
  storage: number;
  apiCalls: number;
  customDomain: boolean;
  sso: boolean;
  auditLog: boolean;
  priority: 'low' | 'medium' | 'high';
}
