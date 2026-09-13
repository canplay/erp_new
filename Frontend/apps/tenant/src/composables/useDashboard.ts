/**
 * @file useDashboard.ts
 * @description 仪表盘页面业务逻辑 composable
 */

import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import { listLoginLogs, type LoginLog } from '@/api/user';
import { httpClient } from '@/utils/alova';

export function useDashboard() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  const loading = ref(false);

  const statistics = ref({
    total_users: 0,
    active_users: 0,
    new_users_today: 0,
    login_attempts: 0,
    failed_logins: 0,
    successful_logins: 0,
  });

  const userGrowthData = ref<Array<{ label: string; value: number }>>([]);
  const loginTypeData = ref<Array<{ label: string; value: number }>>([]);
  const recentLogins = ref<LoginLog[]>([]);

  const loginColumns = computed(() => [
    { name: 'username', label: $t('loginLog.username'), field: 'username', align: 'left' as const },
    { name: 'loginType', label: $t('loginLog.loginType'), field: 'login_method', align: 'left' as const },
    { name: 'ip', label: $t('loginLog.ip'), field: 'ip', align: 'left' as const },
    { name: 'status', label: $t('loginLog.status'), field: 'success', align: 'center' as const },
    { name: 'time', label: $t('loginLog.time'), field: 'created_at', align: 'left' as const },
  ]);

  async function loadStatistics() {
    try {
      const response = await httpClient.get('/admin/stats');
      // alova transformResponse 已把业务体 {statistics, user_growth, login_types}
      // 放在 response.data（并且拍平到顶层）。不要再用 respData.data?.data ——
      // 那是旧契约，读出来恒为 undefined，导致统计卡片全部为 0。
      const respData = response as {
        data?: { statistics?: Record<string, number>; user_growth?: Array<{ label: string; value: number }>; login_types?: Array<{ label: string; value: number }> };
        statistics?: Record<string, number>;
        user_growth?: Array<{ label: string; value: number }>;
        login_types?: Array<{ label: string; value: number }>;
      };
      const respBody = respData.data || respData;
      const stats = respBody.statistics;
      if (stats) {
        statistics.value = {
          total_users: stats.total_users || 0,
          active_users: stats.active_users || 0,
          new_users_today: stats.new_users_today || 0,
          login_attempts: stats.login_attempts || 0,
          failed_logins: stats.failed_logins || 0,
          successful_logins: stats.successful_logins || 0,
        };
      }
      const userGrowth = respBody.user_growth;
      if (userGrowth && Array.isArray(userGrowth)) {
        userGrowthData.value = userGrowth.map((item) => ({ label: item.label, value: item.value }));
      }
      const loginTypes = respBody.login_types;
      if (loginTypes && Array.isArray(loginTypes)) {
        loginTypeData.value = loginTypes.map((item) => ({ label: item.label, value: item.value }));
      }
    } catch (error) {
      logger.error('【获取统计失败】', error);
      statistics.value = { total_users: 0, active_users: 0, new_users_today: 0, login_attempts: 0, failed_logins: 0, successful_logins: 0 };
    }
  }

  async function loadRecentLogins() {
    try {
      const response = await listLoginLogs({ page: 1, page_size: 5 });
      const respData = response as { list?: LoginLog[]; data?: { list?: LoginLog[] } };
      const data = respData.list ? { list: respData.list } : respData.data;
      if (data?.list) {
        recentLogins.value = data.list;
      } else if (Array.isArray(data)) {
        recentLogins.value = data as LoginLog[];
      } else {
        recentLogins.value = [];
      }
    } catch (error) {
      logger.error('【获取登录日志失败】', error);
      recentLogins.value = [];
    }
  }

  function formatTime(timeStr: string | number): string {
    if (!timeStr) return '-';
    // 后端返回 epoch 秒（10 位），需转毫秒
    const ts = typeof timeStr === 'string' ? Number(timeStr) : timeStr;
    const date = new Date(ts < 1e12 ? ts * 1000 : ts);
    return date.toLocaleString(undefined, {
      year: 'numeric', month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit',
    });
  }

  async function handleRefresh() {
    loading.value = true;
    try {
      await Promise.all([loadStatistics(), loadRecentLogins()]);
      $q.notify({ type: 'positive', message: $t('common.success') });
    } catch (error) {
      logger.error('【刷新数据失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    } finally {
      loading.value = false;
    }
  }

  async function initData() {
    loading.value = true;
    await Promise.all([loadStatistics(), loadRecentLogins()]);
    loading.value = false;
  }

  return {
    loading, statistics, userGrowthData, loginTypeData, recentLogins, loginColumns,
    handleRefresh, formatTime, initData,
  };
}
