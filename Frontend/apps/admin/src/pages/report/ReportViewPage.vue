/**
 * @file ReportViewPage.vue
 * @brief 报表预览页面
 * @description 支持表格、图表、仪表盘三种类型的报表预览
 */

<template>
  <q-page class="q-pa-md">
    <!-- 加载状态 -->
    <div v-if="loading" class="row justify-center items-center q-py-xl">
      <q-spinner-dots size="50px" color="primary" />
    </div>

    <!-- 报表内容 -->
    <div v-else-if="reportData">
      <!-- 表格类型 -->
      <template v-if="reportType === 'table'">
        <q-card>
          <q-card-section>
            <div class="text-h6">{{ reportName }}</div>
          </q-card-section>
          <q-separator />
          <q-card-section>
            <q-table
              :rows="tableRows"
              :columns="tableColumns"
              row-key="index"
              flat
              bordered
              :pagination="{ rowsPerPage: 0 }"
            />
          </q-card-section>
          <q-separator v-if="summary" />
          <q-card-section v-if="summary" class="bg-grey-1">
            <div class="text-subtitle2">汇总</div>
            <div class="row q-col-gutter-md q-mt-sm">
              <div v-for="(value, key) in summary" :key="key" class="col-auto">
                <span class="text-grey-7">{{ key }}:</span>
                <span class="q-ml-sm text-weight-medium">{{ value }}</span>
              </div>
            </div>
          </q-card-section>
        </q-card>
      </template>

      <!-- 图表类型 -->
      <template v-else-if="reportType === 'chart'">
        <q-card>
          <q-card-section>
            <div class="text-h6">{{ reportName }}</div>
          </q-card-section>
          <q-separator />
          <q-card-section>
            <div style="width: 100%; height: 400px">
              <ChartWidget :config="reportChartConfig || {}" height="100%" />
            </div>
          </q-card-section>
        </q-card>
      </template>

      <!-- 仪表盘类型 -->
      <template v-else-if="reportType === 'dashboard'">
        <q-card>
          <q-card-section>
            <div class="text-h6">{{ reportName }}</div>
          </q-card-section>
          <q-separator />
          <q-card-section>
            <!-- 指标卡片 -->
            <div v-if="metrics && metrics.length > 0" class="row q-col-gutter-md q-mb-lg">
              <div v-for="metric in metrics" :key="metric.name" class="col-12 col-sm-6 col-md-4">
                <q-card class="text-center q-pa-md" bordered>
                  <div class="text-h5 text-weight-bold">{{ metric.value }}</div>
                  <div class="text-grey-7">{{ metric.name }}</div>
                  <div class="q-mt-sm">
                    <q-badge
                      :color="metric.change >= 0 ? 'positive' : 'negative'"
                      :label="`${metric.change >= 0 ? '+' : ''}${metric.change}%`"
                    />
                  </div>
                </q-card>
              </div>
            </div>

            <!-- 图表 -->
            <div v-if="dashboardCharts && dashboardCharts.length > 0" class="row q-col-gutter-md">
              <div
                v-for="(chart, index) in dashboardCharts"
                :key="index"
                class="col-12"
                :class="dashboardCharts.length === 1 ? '' : 'col-md-6'"
              >
                <q-card bordered class="q-pa-md">
                  <div class="text-subtitle2 q-mb-md">{{ chart.name }}</div>
                  <ECharts :config="getChartConfigForDashboard(chart)" />
                </q-card>
              </div>
            </div>
          </q-card-section>
        </q-card>
      </template>
    </div>

    <!-- 空状态 -->
    <EmptyState
      v-else
      icon="assessment"
      :message="$t('common.noReportData')"
    />

    <!-- 工具栏 -->
    <div class="fixed-bottom-right q-pa-md">
      <q-btn-group rounded>
        <q-btn color="primary" icon="refresh" @click="loadReportData">
          <q-tooltip>刷新</q-tooltip>
        </q-btn>
        <q-btn color="positive" icon="download" @click="handleExport">
          <q-tooltip>导出</q-tooltip>
        </q-btn>
        <q-btn color="grey" icon="close" @click="handleBack">
          <q-tooltip>返回</q-tooltip>
        </q-btn>
      </q-btn-group>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
/**
 * @brief 报表预览页面
 * @description 根据报表类型渲染不同的预览视图
 * 支持表格、图表、仪表盘三种类型
 */
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { getReportData, exportReport, type ReportType } from '@/api/report';
import ECharts from '@/components/ECharts/Main.vue';
import { computed } from 'vue';
import type { chartConfig } from '@/components/ECharts/chartOptions';
import EmptyState from '@/components/EmptyState.vue';

const $q = useQuasar();
const route = useRoute();
const router = useRouter();

const loading = ref(false);
const reportData = ref<Record<string, unknown> | null>(null);
const reportName = ref('');
const reportType = ref<ReportType>('table');

// 表格数据
const tableRows = ref<Record<string, unknown>[]>([]);
const tableColumns = ref<{ name: string; label: string; field: string; align: 'left' | 'right' | 'center' }[]>([]);
const summary = ref<Record<string, unknown> | null>(null);

interface ReportResponse {
  data?: Record<string, unknown>;
  [key: string]: unknown;
}

// 图表配置
interface ChartConfig {
  type: string;
  data: unknown[];
}

const reportChartConfig = ref<ChartConfig>({
  type: 'bar',
  data: [],
});

// 仪表盘数据
const metrics = ref<{ name: string; value: number; change: number }[]>([]);
const dashboardCharts = ref<{ name: string; type: string; labels: string[]; data: number[] }[]>([]);

// 为仪表盘获取图表配置
function getChartConfigForDashboard(chart: { name: string; type: string; labels: string[]; data: number[] }): { title: string; type: 'line' | 'bar'; data: { label: string; value: number }[] } {
  return {
    title: chart.name,
    type: chart.type as 'line' | 'bar',
    data: chart.labels.map((label, i) => ({ label, value: chart.data[i] ?? 0 })),
  };
}

// 加载报表数据
async function loadReportData() {
  const id = route.params.id as string;
  if (!id) {
    $q.notify({ type: 'negative', message: '报表ID不存在' });
    return;
  }

  loading.value = true;
  try {
    const response = await getReportData(id);
    // 类型断言：兼容新旧格式（已展开的字段）
    const respData = response as unknown as ReportResponse;
    const data = respData.data || respData;
    
    if (data) {
      reportData.value = data;
      parseReportData(data);
    }
  } catch {
    $q.notify({ type: 'negative', message: '加载报表数据失败' });
  } finally {
    loading.value = false;
  }
}

// 解析报表数据
function parseReportData(data: Record<string, unknown>) {
  const typeValue = data.type;
  const type = typeof typeValue === 'string' ? typeValue : 'table';
  reportType.value = type as ReportType;

  switch (type) {
    case 'table':
      parseTableData(data);
      break;
    case 'chart':
      parseChartData(data);
      break;
    case 'dashboard':
      parseDashboardData(data);
      break;
    default:
      $q.notify({ type: 'warning', message: `不支持的报表类型: ${type}` });
  }
}

// 解析表格数据
function parseTableData(data: Record<string, unknown>) {
  const columns = (data.columns as string[]) || [];
  const rows = (data.rows as unknown[][]) || [];
  
  tableColumns.value = columns.map((col, index) => ({
    name: `col${index}`,
    label: col,
    field: `col${index}`,
    align: 'left' as const,
  }));

  tableRows.value = rows.map((row, rowIndex) => {
    const obj: Record<string, unknown> = { index: rowIndex };
    row.forEach((cell, cellIndex) => {
      obj[`col${cellIndex}`] = cell;
    });
    return obj;
  });

  summary.value = data.summary as Record<string, unknown> | null;
}

// 解析图表数据
function parseChartData(data: Record<string, unknown>) {
  const chartType = (data.chart_type as string) || 'bar';
  const labels = (data.labels as string[]) || [];
  const datasets = (data.datasets as { label: string; data: number[] }[]) || [];

  // 使用默认数据集或第一个数据集
  const defaultData = datasets[0]?.data ?? [];
  reportChartConfig.value = {
    type: chartType,
    title: datasets[0]?.label ?? '',
    data: labels.map((label, i) => ({ label, value: defaultData[i] ?? 0 })),
  };
}

// 解析仪表盘数据
function parseDashboardData(data: Record<string, unknown>) {
  // 解析指标
  const metricsData = data.metrics as { name: string; value: number; change: number }[] || [];
  metrics.value = metricsData;

  // 解析图表
  const charts = data.charts as { name: string; type: string; data: number[] }[] || [];
  dashboardCharts.value = charts.map((chart) => {
    // 为每个图表生成标签
    const labels = chart.data.map((_, i) => `项${i + 1}`);
    return {
      name: chart.name,
      type: chart.type || 'bar',
      labels,
      data: chart.data,
    };
  });
}

// 导出报表
async function handleExport() {
  const id = route.params.id as string;
  if (!id) return;

  try {
    await exportReport(id);
    $q.notify({ type: 'positive', message: '导出任务已启动' });
  } catch {
    $q.notify({ type: 'negative', message: '导出失败' });
  }
}

// 返回列表
function handleBack() {
  void router.push('/report');
}

// 生命周期
onMounted(() => {
  reportName.value = (route.query.name as string) || '报表预览';
  void loadReportData();
});
</script>

<style scoped>
.fixed-bottom-right {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 1000;
}
</style>