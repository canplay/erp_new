/**
 * @file useDataDashboard.ts
 * @description 数据统计大屏页面业务逻辑 composable
 * @date 2026-07-07
 */

import { ref, reactive } from 'vue';
import { useI18n } from 'vue-i18n';
import * as echarts from 'echarts';
import { getReportData } from '@/api/report';
import { logger } from '@/utils/logger';

// ======================== 类型定义 ========================

interface Stats {
  users: { total: number; active: number; online: number; growth: number };
  orders: { total: number; growth: number };
  revenue: { total: number; growth: number };
}

interface GeoItem {
  province: string;
  percent: number;
}

interface UserItem {
  id: number;
  username: string;
  email: string;
  created_at: string;
}

interface OrderItem {
  id: number;
  product: string;
  amount: number;
  created_at: string;
}

// ======================== Composable ========================

export function useDataDashboard() {
  const { t: $t } = useI18n();

  // ---- 时间范围 ----
  const timeRange = ref('7d');
  const timeRangeOptions = [
    { label: $t('dashboard.last7days') || '最近7天', value: '7d' },
    { label: $t('dashboard.last30days') || '最近30天', value: '30d' },
    { label: $t('dashboard.last90days') || '最近90天', value: '90d' },
    { label: $t('dashboard.thisYear') || '今年', value: 'year' },
  ];

  // ---- 统计数据 ----
  const stats = reactive<Stats>({
    users: { total: 0, active: 0, online: 0, growth: 0 },
    orders: { total: 0, growth: 0 },
    revenue: { total: 0, growth: 0 },
  });

  // ---- 列表数据 ----
  const geoData = ref<GeoItem[]>([]);
  const latestUsers = ref<UserItem[]>([]);
  const latestOrders = ref<OrderItem[]>([]);

  // ---- Chart 实例（内部管理） ----
  let visitChart: echarts.ECharts | null = null;
  let deviceChart: echarts.ECharts | null = null;
  let userGrowthChart: echarts.ECharts | null = null;
  let orderChart: echarts.ECharts | null = null;

  // ======================== 工具函数 ========================

  function formatNumber(num: number): string {
    if (num >= 10000) {
      return (num / 10000).toFixed(1) + 'w';
    }
    return num.toLocaleString();
  }

  function formatTimeAgo(dateStr: string): string {
    const diff = Date.now() - new Date(dateStr).getTime();
    const minutes = Math.floor(diff / 60000);
    if (minutes < 60) return `${minutes}分钟前`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}小时前`;
    const days = Math.floor(hours / 24);
    return `${days}天前`;
  }

  function getGeoColor(index: number): string {
    const colors = ['primary', 'positive', 'warning', 'info', 'secondary', 'grey'];
    return colors[index % colors.length] ?? 'grey';
  }

  // ======================== 数据加载 ========================

  async function loadDashboardData(): Promise<void> {
    try {
      const response = await getReportData('dashboard-stats');
      if (response.data?.data) {
        const data = response.data.data as {
          users?: { total: number; active: number; online: number; growth: number };
          orders?: { total: number; growth: number };
          revenue?: { total: number; growth: number };
        };
        if (data.users) {
          stats.users = { ...stats.users, ...data.users };
        }
        if (data.orders) {
          stats.orders = { ...stats.orders, ...data.orders };
        }
        if (data.revenue) {
          stats.revenue = { ...stats.revenue, ...data.revenue };
        }
        logger.info('【仪表盘数据加载成功】', { timeRange: timeRange.value });
      }
    } catch (error) {
      // API 未实现时使用默认数据（降级处理）
      logger.warn('【仪表盘 API 未实现，使用默认数据】', error);
      logger.info('【加载仪表盘数据】时间范围:', timeRange.value, '(使用默认模拟数据)');
    }
  }

  // ======================== 图表初始化 ========================

  function initVisitChart(el: HTMLDivElement): void {
    visitChart = echarts.init(el);
    const option: echarts.EChartsOption = {
      tooltip: { trigger: 'axis' },
      legend: { data: ['访问量', '用户数'], bottom: 0 },
      grid: { left: '3%', right: '4%', bottom: '15%', containLabel: true },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: ['周一', '周二', '周三', '周四', '周五', '周六', '周日'],
      },
      yAxis: { type: 'value' },
      series: [
        {
          name: '访问量',
          type: 'line',
          smooth: true,
          data: [820, 932, 901, 1234, 1290, 1330, 1520],
          areaStyle: { opacity: 0.3 },
        },
        {
          name: '用户数',
          type: 'line',
          smooth: true,
          data: [320, 432, 501, 634, 790, 830, 920],
        },
      ],
    };
    visitChart.setOption(option);
  }

  function initDeviceChart(el: HTMLDivElement): void {
    deviceChart = echarts.init(el);
    const option: echarts.EChartsOption = {
      tooltip: { trigger: 'item' },
      legend: { orient: 'vertical', right: '5%', top: 'center' },
      series: [
        {
          type: 'pie',
          radius: ['40%', '70%'],
          center: ['35%', '50%'],
          avoidLabelOverlap: false,
          itemStyle: { borderRadius: 10, borderColor: '#fff', borderWidth: 2 },
          label: { show: false },
          emphasis: { label: { show: true, fontSize: 14, fontWeight: 'bold' } },
          data: [
            { value: 60, name: 'PC', itemStyle: { color: '#1976D2' } },
            { value: 25, name: 'Mobile', itemStyle: { color: '#4CAF50' } },
            { value: 10, name: 'Tablet', itemStyle: { color: '#FFC107' } },
            { value: 5, name: 'Other', itemStyle: { color: '#9E9E9E' } },
          ],
        },
      ],
    };
    deviceChart.setOption(option);
  }

  function initUserGrowthChart(el: HTMLDivElement): void {
    userGrowthChart = echarts.init(el);
    const option: echarts.EChartsOption = {
      tooltip: { trigger: 'axis' },
      grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
      xAxis: {
        type: 'category',
        data: ['1月', '2月', '3月', '4月', '5月', '6月'],
      },
      yAxis: { type: 'value' },
      series: [
        {
          type: 'bar',
          data: [320, 452, 631, 824, 956, 1205],
          itemStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#1976D2' },
              { offset: 1, color: '#42A5F5' },
            ]),
            borderRadius: [4, 4, 0, 0],
          },
        },
      ],
    };
    userGrowthChart.setOption(option);
  }

  function initOrderChart(el: HTMLDivElement): void {
    orderChart = echarts.init(el);
    const option: echarts.EChartsOption = {
      tooltip: { trigger: 'item' },
      legend: { bottom: 0 },
      series: [
        {
          type: 'pie',
          radius: ['35%', '60%'],
          center: ['50%', '45%'],
          data: [
            { value: 335, name: '已完成', itemStyle: { color: '#4CAF50' } },
            { value: 234, name: '处理中', itemStyle: { color: '#FFC107' } },
            { value: 154, name: '已取消', itemStyle: { color: '#F44336' } },
          ],
        },
      ],
    };
    orderChart.setOption(option);
  }

  /**
   * 初始化所有图表，传入 DOM 元素引用
   */
  function initCharts(refs: {
    visitChartEl: HTMLDivElement | null;
    deviceChartEl: HTMLDivElement | null;
    userGrowthChartEl: HTMLDivElement | null;
    orderChartEl: HTMLDivElement | null;
  }): void {
    if (refs.visitChartEl) initVisitChart(refs.visitChartEl);
    if (refs.deviceChartEl) initDeviceChart(refs.deviceChartEl);
    if (refs.userGrowthChartEl) initUserGrowthChart(refs.userGrowthChartEl);
    if (refs.orderChartEl) initOrderChart(refs.orderChartEl);
  }

  // ======================== 窗口响应 ========================

  function handleResize(): void {
    visitChart?.resize();
    deviceChart?.resize();
    userGrowthChart?.resize();
    orderChart?.resize();
  }

  // ======================== 清理 ========================

  function disposeCharts(): void {
    visitChart?.dispose();
    deviceChart?.dispose();
    userGrowthChart?.dispose();
    orderChart?.dispose();
    visitChart = null;
    deviceChart = null;
    userGrowthChart = null;
    orderChart = null;
  }

  // ======================== 返回 ========================

  return {
    // 状态
    timeRange,
    timeRangeOptions,
    stats,
    geoData,
    latestUsers,
    latestOrders,
    // 方法
    formatNumber,
    formatTimeAgo,
    getGeoColor,
    loadDashboardData,
    initCharts,
    handleResize,
    disposeCharts,
  };
}
