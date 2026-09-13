/**
 * @file clean.ts
 * @description 清运服务 API
 * @date 2026-05-19
 */

import { httpClient } from '@/utils/alova';
import type { PaginationParams } from '@/utils/alova';
import { handleApiError } from '@/utils/apiErrorHandler';

// ============ 类型定义 ============

/** 发票 */
export interface Invoice {
  id: string;
  order_id: string;
  user_id: string;
  amount: number;
  status: 'pending' | 'issued' | 'void';
  invoice_no?: string;
  created_at: string;
  updated_at: string;
}

/** 发票查询参数 */
export interface InvoiceQuery extends PaginationParams {
  user_id?: string;
  status?: string;
}

/** 订单 */
export interface Order {
  id: string;
  order_no: string;
  user_id: string;
  amount: number;
  status: 'pending' | 'paid' | 'completed' | 'cancelled';
  pay_type: string;
  created_at: string;
  updated_at: string;
  paid_at?: string;
}

/** 订单查询参数 */
export interface OrderQuery extends PaginationParams {
  user_id?: string;
  status?: string;
  start_date?: string;
  end_date?: string;
}

/** 员工 */
export interface Staff {
  id: string;
  username?: string;
  realname?: string;
  userkey?: string;
  phone?: string;
  email?: string;
  departid?: string;
  status?: string;
  browser?: string;
  location?: string;
  activiti_sync?: boolean;
}

/** 员工查询参数 */
export interface StaffQuery {
  id: string;
}

/** 正式账单 */
export interface FormalBill {
  id: string;
  order_no: string;
  user_id: string;
  amount: number;
  status: 'pending' | 'confirmed' | 'paid';
  created_at: string;
}

/** 正式账单查询参数 */
export interface FormalBillQuery extends PaginationParams {
  user_id?: string;
  status?: string;
  start_date?: string;
  end_date?: string;
}

/** 支付信息 */
export interface PaymentInfo {
  id: string;
  order_id: string;
  user_id: string;
  amount: number;
  pay_type: string;
  pay_time: string;
  status: string;
}

/** 支付信息查询参数 */
export interface PaymentInfoQuery extends PaginationParams {
  user_id?: string;
  status?: string;
}

/** 支付统计查询参数 */
export interface PaymentStatisticsQuery {
  start_date: string;
  end_date: string;
  user_id?: string;
}

/** 网络支付 */
export interface PaymentWeb {
  id: string;
  order_id: string;
  user_id: string;
  amount: number;
  channel: string;
  status: string;
  pay_time: string;
}

/** 网络支付查询参数 */
export interface PaymentWebQuery extends PaginationParams {
  user_id?: string;
  status?: string;
  channel?: string;
}

/** 网络支付统计查询参数 */
export interface PaymentWebStatisticsQuery {
  start_date: string;
  end_date: string;
  user_id?: string;
}

// ============ 发票管理 API ============

/**
 * 获取发票数量
 */
export async function countInvoices(params: InvoiceQuery) {
  try {
    return await httpClient.get('/clean/invoice/count', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取发票列表
 */
export async function listInvoices(params: InvoiceQuery) {
  try {
    return await httpClient.get('/clean/invoice/info', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 更新发票
 */
export async function updateInvoice(data: { id: string; invoice_no?: string; status?: string }) {
  try {
    return await httpClient.post('/clean/invoice/update', data);
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

// ============ 订单管理 API ============

/**
 * 获取订单数量
 */
export async function countOrders(params: OrderQuery) {
  try {
    return await httpClient.get('/clean/order/count', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取订单列表
 */
export async function listOrders(params: OrderQuery) {
  try {
    return await httpClient.get('/clean/order/info', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

// ============ 员工管理 API ============

/**
 * 获取员工信息
 */
export async function getStaff(query: StaffQuery) {
  try {
    return await httpClient.post('/clean/staff/info', query);
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

// ============ 统计管理 API ============

/**
 * 获取正式账单数量
 */
export async function countFormalBills(params: FormalBillQuery) {
  try {
    return await httpClient.get('/clean/formal/count', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取正式账单列表
 */
export async function listFormalBills(params: FormalBillQuery) {
  try {
    return await httpClient.get('/clean/formal/info', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取正式账单统计
 */
export async function getFormalBillTotal(params: FormalBillQuery) {
  try {
    return await httpClient.get('/clean/formal/total', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取支付信息数量
 */
export async function countPaymentInfo(params: PaymentInfoQuery) {
  try {
    return await httpClient.get('/clean/payment/info/count', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取支付信息列表
 */
export async function listPaymentInfo(params: PaymentInfoQuery) {
  try {
    return await httpClient.get('/clean/payment/info/info', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取支付信息统计
 */
export async function getPaymentInfoTotal(params: PaymentStatisticsQuery) {
  try {
    return await httpClient.get('/clean/payment/info/total', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取网络支付数量
 */
export async function countPaymentWeb(params: PaymentWebQuery) {
  try {
    return await httpClient.get('/clean/payment/web/count', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取网络支付列表
 */
export async function listPaymentWeb(params: PaymentWebQuery) {
  try {
    return await httpClient.get('/clean/payment/web/info', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

/**
 * 获取网络支付统计
 */
export async function getPaymentWebTotal(params: PaymentWebStatisticsQuery) {
  try {
    return await httpClient.get('/clean/payment/web/total', { params });
  } catch (error) {
    handleApiError(error, '清运服务');
    throw error;
  }
}

// ============ 导出 ============

export const cleanApi = {
  // 发票管理
  invoice: {
    count: countInvoices,
    list: listInvoices,
    update: updateInvoice,
  },
  
  // 订单管理
  order: {
    count: countOrders,
    list: listOrders,
  },
  
  // 员工管理
  staff: {
    get: getStaff,
  },
  
  // 正式账单
  formal: {
    count: countFormalBills,
    list: listFormalBills,
    total: getFormalBillTotal,
  },
  
  // 支付信息
  paymentInfo: {
    count: countPaymentInfo,
    list: listPaymentInfo,
    total: getPaymentInfoTotal,
  },
  
  // 网络支付
  paymentWeb: {
    count: countPaymentWeb,
    list: listPaymentWeb,
    total: getPaymentWebTotal,
  },
};

export default cleanApi;