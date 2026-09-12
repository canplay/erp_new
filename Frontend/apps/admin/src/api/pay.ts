/**
 * @file pay.ts
 * @description 支付服务 API
 * @date 2026-05-19
 */

import { httpClient } from '@/utils/alova';
import type { PaginationParams } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

// ============ 类型定义 ============

/** 支付订单 */
export interface PayOrder {
  id: string;
  user_id: string;
  order_no: string;
  amount: number;
  status: 'pending' | 'paid' | 'failed' | 'refunded';
  pay_type: 'ccb' | 'ums' | 'other';
  created_at: string;
  updated_at: string;
  paid_at?: string;
}

/** 支付查询参数 */
export interface PayQuery extends PaginationParams {
  user_id?: string;
  status?: string;
  start_date?: string;
  end_date?: string;
}

/** 创建支付参数 */
export interface PayCreateParams {
  user_id: string;
  amount: number;
  pay_type: 'ccb' | 'ums';
  order_no?: string;
  subject?: string;
  body?: string;
}

/** CCB 支付参数 */
export interface CcbOrderParams {
  merchant_order_id: string;
  amount: string;
  order_desc: string;
  user_id?: string;
}

/** UMS 支付参数 */
export interface UmsOrderParams {
  order_id: string;
  amount: number;
  mer_id: string;
  subject: string;
}

// ============ API 函数 ============

/**
 * 获取支付订单数量
 */
export function countPayOrders(params: PayQuery) {
  try {
    return await httpClient.post('/v1/pay/count', { params });
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * 获取支付订单列表
 */
export function listPayOrders(params: PayQuery) {
  try {
    return await httpClient.post('/v1/pay/list', { params });
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * 获取用户最近支付订单
 */
export function getLatestPayOrder(user_id: string) {
  try {
    return await httpClient.get(`/v1/pay/latest/${user_id}`);
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * 创建支付订单
 */
export function createPayOrder(data: PayCreateParams) {
  try {
    return await httpClient.post('/v1/pay/create', data);
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

// ============ CCB 支付接口 ============

/**
 * CCB 支付查询
 */
export function ccbQuery(params: { merchant_order_id: string }) {
  try {
    return await httpClient.post('/v1/pay/ccb/query', { params });
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * CCB 创建支付订单
 */
export function ccbCreate(data: CcbOrderParams) {
  try {
    return await httpClient.post('/v1/pay/ccb/create', data);
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * CCB 验证支付结果
 */
export function ccbVerify(orderId: string) {
  try {
    return await httpClient.get(`/v1/pay/ccb/verify/${orderId}`);
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * CCB 退款
 */
export function ccbRefund(data: { order_id: string; amount: number }) {
  try {
    return await httpClient.post('/v1/pay/ccb/refund', data);
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

// ============ UMS 支付接口 ============

/**
 * UMS 支付查询
 */
export function umsQuery(params: { order_id: string }) {
  try {
    return await httpClient.post('/v1/pay/ums/query', { params });
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * UMS 创建支付订单
 */
export function umsCreate(data: UmsOrderParams) {
  try {
    return await httpClient.post('/v1/pay/ums/create', data);
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * UMS 关闭支付订单
 */
export function umsClose(data: { order_id: string }) {
  try {
    return await httpClient.post('/v1/pay/ums/close', data);
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * UMS 退款
 */
export function umsRefund(data: { order_id: string; amount: number; reason?: string }) {
  try {
    return await httpClient.post('/v1/pay/ums/refund', data);
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

/**
 * UMS 获取订单信息
 */
export function umsInfo(orderId: string) {
  try {
    return await httpClient.get(`/v1/pay/ums/info/${orderId}`);
  } catch (error) {
    handleApiError(error, '支付服务');
    throw error;
  }
}

// ============ 导出 ============

export const payApi = {
  // 基础支付
  count: countPayOrders,
  list: listPayOrders,
  latest: getLatestPayOrder,
  create: createPayOrder,
  
  // CCB 支付
  ccb: {
    query: ccbQuery,
    create: ccbCreate,
    verify: ccbVerify,
    refund: ccbRefund,
  },
  
  // UMS 支付
  ums: {
    query: umsQuery,
    create: umsCreate,
    close: umsClose,
    refund: umsRefund,
    info: umsInfo,
  },
};

export default payApi;