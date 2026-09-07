/**
 * @file feedback.ts
 * @description 反馈状态管理
 * @date 2026-05-05
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '@/utils/logger';
import {
  getFeedbackList,
  getFeedbackDetail,
  handleFeedback,
  transferFeedback,
  addFeedbackReply,
  closeFeedback,
  deleteFeedback,
  batchHandleFeedback,
  getFeedbackStatistics,
  getFeedbackTypeStatistics,
  getFeedbackHandlers,
  type Feedback,
  type FeedbackQueryParams,
  type FeedbackHandleParams,
} from '@/api/feedback';
/** API 响应类型定义 */
interface FeedbackListResponse {
  list: Feedback[];
  total: number;
}

interface StatisticsResponse {
  total: number;
  pending: number;
  processing: number;
  resolved: number;
  rejected: number;
  avg_response_time: number;
  satisfactionRate: number;
}

interface TypeStatisticsItem {
  type: string;
  count: number;
  percentage: number;
}

interface HandlerItem {
  id: number;
  name: string;
}
export const useFeedbackStore = defineStore("feedback", () => {
  // ============ 状态定义 ============

  /** 反馈列表 */
  const feedbackList = ref<Feedback[]>([]);

  /** 当前反馈详情 */
  const currentFeedback = ref<Feedback | null>(null);

  /** 处理人列表 */
  const handlers = ref<Array<{ id: number; name: string }>>([]);

  /** 统计数据 */
  const statistics = ref<{
    total: number;
    pending: number;
    processing: number;
    resolved: number;
    rejected: number;
    avg_response_time: number;
    satisfactionRate: number;
  } | null>(null);

  /** 类型统计 */
  const typeStatistics = ref<Array<{
    type: string;
    count: number;
    percentage: number;
  }>>([]);

  /** 加载状态 */
  const isLoading = ref(false);

  /** 分页信息 */
  const pagination = ref({
    page: 1,
    page_size: 15,
    total: 0,
  });
  // ============ 计算属性 ============

  /** 待处理数量 */
  const pendingCount = computed(() => {
    if (!statistics.value) return 0;
    return statistics.value.pending;
  });

  /** 处理中数量 */
  const processingCount = computed(() => {
    if (!statistics.value) return 0;
    return statistics.value.processing;
  });

  /** 已解决数量 */
  const resolvedCount = computed(() => {
    if (!statistics.value) return 0;
    return statistics.value.resolved;
  });

  /** 解决率 */
  const resolutionRate = computed(() => {
    if (!statistics.value || statistics.value.total === 0) return 0;
    return Math.round((statistics.value.resolved / statistics.value.total) * 100);
  });
  // ============ 方法 ============

  /**
   * @brief 加载反馈列表
   */
  async function fetchFeedbackList(params?: FeedbackQueryParams) {
    isLoading.value = true;
    try {
      const response = await getFeedbackList({
        page: pagination.value.page,
        page_size: pagination.value.page_size,
        ...params,
      });
      const data = (response as { data?: FeedbackListResponse }).data;
      if (data) {
        feedbackList.value = data.list || [];
        pagination.value.total = data.total || 0;
      }
    } catch (error) {
      logger.error('【加载反馈列表失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * @brief 加载反馈详情
   */
  async function fetchFeedbackDetail(id: number) {
    isLoading.value = true;
    try {
      const response = await getFeedbackDetail(id);
      currentFeedback.value = (response as { data?: Feedback }).data || null;
      return currentFeedback.value;
    } catch (error) {
      logger.error('【加载反馈详情失败】', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }
  /**
   * @brief 处理反馈
   */
  async function processFeedback(id: number, data: FeedbackHandleParams) {
    try {
      await handleFeedback(id, data);
      // 更新列表中的状态
      const feedback = feedbackList.value.find((f) => f.id === id);
      if (feedback) {
        feedback.status = data.status;
        feedback.handlerReply = data.handlerReply;
        feedback.handlerTime = new Date().toISOString();
      }
      // 更新当前详情
      if (currentFeedback.value?.id === id) {
        currentFeedback.value.status = data.status;
        currentFeedback.value.handlerReply = data.handlerReply;
        currentFeedback.value.handlerTime = new Date().toISOString();
      }
      return true;
    } catch (error) {
      logger.error('【处理反馈失败】', error);
      throw error;
    }
  }

  /**
   * @brief 转交反馈
   */
  async function transferFeedbackHandler(id: number, handlerId: number) {
    try {
      await transferFeedback(id, handlerId);
      await fetchFeedbackDetail(id);
      return true;
    } catch (error) {
      logger.error('【转交反馈失败】', error);
      throw error;
    }
  }

  /**
   * @brief 添加回复
   */
  async function replyToFeedback(id: number, reply: string) {
    try {
      await addFeedbackReply(id, reply);
      await fetchFeedbackDetail(id);
      return true;
    } catch (error) {
      logger.error('【添加回复失败】', error);
      throw error;
    }
  }
  /**
   * @brief 关闭反馈
   */
  async function closeFeedbackById(id: number) {
    try {
      await closeFeedback(id);
      const feedback = feedbackList.value.find((f) => f.id === id);
      if (feedback) {
        feedback.status = 'closed';
      }
      if (currentFeedback.value?.id === id) {
        currentFeedback.value.status = 'closed';
      }
      return true;
    } catch (error) {
      logger.error('【关闭反馈失败】', error);
      throw error;
    }
  }

  /**
   * @brief 删除反馈
   */
  async function removeFeedback(id: number) {
    try {
      await deleteFeedback(id);
      feedbackList.value = feedbackList.value.filter((f) => f.id !== id);
      if (currentFeedback.value?.id === id) {
        currentFeedback.value = null;
      }
      return true;
    } catch (error) {
      logger.error('【删除反馈失败】', error);
      throw error;
    }
  }

  /**
   * @brief 批量处理反馈
   */
  async function batchProcessFeedback(ids: number[], data: FeedbackHandleParams) {
    try {
      await batchHandleFeedback(ids, data);
      await fetchFeedbackList();
      return true;
    } catch (error) {
      logger.error('【批量处理反馈失败】', error);
      throw error;
    }
  }
  /**
   * @brief 加载统计数据
   */
  async function fetchStatistics(params?: { start_date?: string; end_date?: string }) {
    try {
      const response = await getFeedbackStatistics(params);
      statistics.value = (response as { data?: StatisticsResponse }).data || null;
    } catch (error) {
      logger.error('【加载统计数据失败】', error);
    }
  }

  /**
   * @brief 加载类型统计
   */
  async function fetchTypeStatistics(params?: { start_date?: string; end_date?: string }) {
    try {
      const response = await getFeedbackTypeStatistics(params);
      const respData = response as { list?: TypeStatisticsItem[]; data?: TypeStatisticsItem[] | { list?: TypeStatisticsItem[] } };
      const dataObj = respData.data as { list?: TypeStatisticsItem[] } | undefined;
      typeStatistics.value = respData.list || dataObj?.list || (Array.isArray(respData.data) ? (respData.data) : []) || [];
    } catch (error) {
      logger.error('【加载类型统计失败】', error);
    }
  }

  /**
   * @brief 加载处理人列表
   */
  async function fetchHandlers() {
    try {
      const response = await getFeedbackHandlers();
      const respData = response as { list?: HandlerItem[]; data?: HandlerItem[] | { list?: HandlerItem[] } };
      const dataObj = respData.data as { list?: HandlerItem[] } | undefined;
      handlers.value = respData.list || dataObj?.list || (Array.isArray(respData.data) ? (respData.data) : []) || [];
    } catch (error) {
      logger.error('【加载处理人列表失败】', error);
    }
  }

  /**
   * @brief 设置分页
   */
  function setPage(page: number) {
    pagination.value.page = page;
  }

  /**
   * @brief 加载所有数据
   */
  async function loadAll(params?: FeedbackQueryParams) {
    await Promise.all([
      fetchFeedbackList(params),
      fetchStatistics(),
      fetchTypeStatistics(),
    ]);
  }
  return {
    // 状态
    feedbackList,
    currentFeedback,
    handlers,
    statistics,
    typeStatistics,
    isLoading,
    pagination,

    // 计算属性
    pendingCount,
    processingCount,
    resolvedCount,
    resolutionRate,

    // 方法
    fetchFeedbackList,
    fetchFeedbackDetail,
    processFeedback,
    transferFeedbackHandler,
    replyToFeedback,
    closeFeedbackById,
    removeFeedback,
    batchProcessFeedback,
    fetchStatistics,
    fetchTypeStatistics,
    fetchHandlers,
    setPage,
    loadAll,
  };
});