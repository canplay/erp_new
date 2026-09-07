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
