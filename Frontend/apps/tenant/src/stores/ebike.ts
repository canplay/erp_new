import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface LoginParams {
  username: string;
  password: string;
  level: number;
  name: string;
}

export interface ScheduleParams {
  system: number;
  alert: number;
}

export interface ScheduleItem {
  key: string;
  value: unknown;
  level: 'system' | 'alert';
}

export interface ScheduleLevel {
  run: boolean;
  list: Map<string, () => void> | null;
  interval: ReturnType<typeof setInterval> | null;
}

export interface ScheduleState {
  now: {
    time: string;
    interval: ReturnType<typeof setInterval> | null;
  };
  system: ScheduleLevel;
  alert: ScheduleLevel;
}

export interface OptionsState {
  system: number;
  alert: number;
  list: Array<Record<string, unknown>>;
}

export interface PaginationState {
  page: number;
  rowsPerPage: number;
  rowsNumber: number;
}

export interface UserState {
  username: string;
  password: string;
  level: number;
  name: string;
}

export const useEbikeStore = defineStore('ebike', () => {
  const backend = {
    // 指向内部 api-gateway 的 ebike 路由（经 nginx /api/ 反代）
    // public: 对外发布（政府公众查询）预留
    // 修复：统一使用 /v1/ebike，与 alova 的 /api 前缀正确拼接
    public: '/v1/ebike',
    private: '/v1/ebike',
  };

  const test = ref(false);
  const showDrawPanel = ref(false);

  const options = ref<OptionsState>({
    system: 5,
    alert: 5,
    list: [],
  });

  const schedule = ref<ScheduleState>({
    now: {
      time: '0000-00-00 00:00:00',
      interval: null,
    },
    system: {
      run: true,
      list: null,
      interval: null,
    },
    alert: {
      run: true,
      list: null,
      interval: null,
    },
  });

  const user = ref<UserState>({
    username: '',
    password: '',
    level: 0,
    name: '',
  });

  // 分页状态（车辆/订单列表）
  const pagination = ref<PaginationState>({
    page: 1,
    rowsPerPage: 20,
    rowsNumber: 0,
  });

  // 防抖/节流控制
  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let filterThrottleTimer: ReturnType<typeof setTimeout> | null = null;
  const FILTER_THROTTLE_MS = 300; // 筛选/分页节流间隔（毫秒）

  /** 防抖搜索：300ms 内只触发最后一次请求 */
  function debouncedSearch(callback: () => void, delay = 300): void {
    if (searchDebounceTimer !== null) {
      clearTimeout(searchDebounceTimer);
    }
    searchDebounceTimer = setTimeout(() => {
      searchDebounceTimer = null;
      callback();
    }, delay);
  }

  /** 节流操作：300ms 内只执行一次 */
  function throttledAction(callback: () => void): void {
    const _now = Date.now();
    if (filterThrottleTimer !== null) {
      clearTimeout(filterThrottleTimer);
    }
    // 如果距离上次执行不足 300ms，则调度下一次
    filterThrottleTimer = setTimeout(() => {
      filterThrottleTimer = null;
      callback();
    }, FILTER_THROTTLE_MS);
  }

  function resetPagination(): void {
    pagination.value = { page: 1, rowsPerPage: 20, rowsNumber: 0 };
  }

  function updatePagination(total: number): void {
    pagination.value.rowsNumber = total;
  }

  function setPage(page: number): void {
    pagination.value.page = page;
  }

  function setRowsPerPage(size: number): void {
    pagination.value.rowsPerPage = size;
    pagination.value.page = 1;
  }

  function getPaginationParams(): { limit: number; offset: number } {
    const { page, rowsPerPage } = pagination.value;
    return {
      limit: rowsPerPage,
      offset: (page - 1) * rowsPerPage,
    };
  }

  function getScheduleList(level: 'system' | 'alert'): Map<string, () => void> | null {
    return schedule.value[level]?.list ?? null;
  }

  function isScheduleRunning(level: 'system' | 'alert'): boolean {
    return schedule.value[level]?.run ?? false;
  }

  function login(params: LoginParams): void {
    user.value.username = params.username;
    user.value.password = params.password;
    user.value.level = params.level;
    user.value.name = params.name;
  }

  function setOptions(params: ScheduleParams): void {
    options.value.system = params.system;
    options.value.alert = params.alert;
  }

  function setOptionsList(params: Array<Record<string, unknown>>): void {
    options.value.list = params;
  }

  function setNow(params: ReturnType<typeof setInterval>): void {
    schedule.value.now.interval = params;
  }

  function setNowTime(time: string): void {
    schedule.value.now.time = time;
  }

  function initSchedule(): void {
    if (schedule.value.system.list === null) {
      schedule.value.system.list = new Map();
      schedule.value.system.interval = setInterval(() => {
        if (schedule.value.system.run && schedule.value.system.list) {
          schedule.value.system.list.forEach((element) => {
            if (element) element();
          });
        }
      }, options.value.system * 1000);
    }

    if (schedule.value.alert.list === null) {
      schedule.value.alert.list = new Map();
      schedule.value.alert.interval = setInterval(() => {
        if (schedule.value.alert.run && schedule.value.alert.list) {
          schedule.value.alert.list.forEach((element) => {
            if (element) element();
          });
        }
      }, options.value.alert * 1000);
    }
  }

  function clearSchedule(): void {
    if (schedule.value.now.interval !== null) {
      clearInterval(schedule.value.now.interval);
      schedule.value.now.interval = null;
    }
    if (schedule.value.system.interval !== null) {
      clearInterval(schedule.value.system.interval);
      schedule.value.system.interval = null;
    }
    if (schedule.value.alert.interval !== null) {
      clearInterval(schedule.value.alert.interval);
      schedule.value.alert.interval = null;
    }
    schedule.value.system.list = null;
    schedule.value.alert.list = null;
  }

  function addSchedule(params: ScheduleItem): void {
    const list = getScheduleList(params.level);
    if (list != null) {
      list.set(params.key, params.value as () => void);
    }
  }

  function delSchedule(params: { key: string; level: 'system' | 'alert' }): void {
    const list = getScheduleList(params.level);
    if (list != null) {
      list.delete(params.key);
    }
  }

  function runPauseSchedule(params: { level: 'system' | 'alert'; value: boolean }): void {
    schedule.value[params.level].run = params.value;
  }

  return {
    backend,
    test,
    showDrawPanel,
    options,
    schedule,
    user,
    pagination,
    getScheduleList,
    isScheduleRunning,
    login,
    setOptions,
    setOptionsList,
    setNow,
    setNowTime,
    initSchedule,
    clearSchedule,
    addSchedule,
    delSchedule,
    runPauseSchedule,
    resetPagination,
    updatePagination,
    setPage,
    setRowsPerPage,
    getPaginationParams,
    debouncedSearch,
    throttledAction,
  };
});


/**
 * @file feedback.ts
 * @description 反馈状态管理
 * @date 2026-05-05
 */

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