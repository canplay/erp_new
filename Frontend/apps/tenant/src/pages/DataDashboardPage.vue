<template>
  <q-page class="q-pa-md data-dashboard">
    <!-- 页面标题和控制栏 -->
    <div class="row items-center q-mb-md">
      <div class="text-h5 text-weight-bold">{{ $t('dashboard.dataDashboard') }}</div>
      <q-space />
      <q-select v-model="timeRange" :options="timeRangeOptions" dense outlined emit-value map-options style="min-width: 150px" />
      <q-btn flat color="primary" icon="refresh" :label="$t('common.refresh')" class="q-ml-md" @click="loadDashboardData" />
    </div>

    <!-- 统计卡片 -->
    <div class="row q-col-gutter-md q-mb-md">
      <!-- 总用户数 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card bordered class="stat-card">
          <q-card-section class="row items-center">
            <div class="col">
              <div class="text-caption text-grey">{{ $t('dashboard.total_users') }}</div>
              <div class="text-h4 text-primary q-mt-sm">{{ stats.users.total }}</div>
              <div class="text-caption">
                <q-icon name="trending_up" color="positive" size="14px" />
                <span class="text-positive q-ml-xs">{{ stats.users.growth }}%</span>
                <span class="text-grey q-ml-xs">{{ $t('dashboard.growth') }}</span>
              </div>
            </div>
            <q-avatar size="60px" color="primary" text-color="white" icon="people" />
          </q-card-section>
        </q-card>
      </div>

      <!-- 活跃用户 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card bordered class="stat-card">
          <q-card-section class="row items-center">
            <div class="col">
              <div class="text-caption text-grey">{{ $t('dashboard.active_users') }}</div>
              <div class="text-h4 text-positive q-mt-sm">{{ stats.users.active }}</div>
              <div class="text-caption">
                <span class="text-grey">{{ $t('dashboard.onlineNow') }}: </span>
                <span class="text-positive">{{ stats.users.online }}</span>
              </div>
            </div>
            <q-avatar size="60px" color="positive" text-color="white" icon="person" />
          </q-card-section>
        </q-card>
      </div>

      <!-- 总订单 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card bordered class="stat-card">
          <q-card-section class="row items-center">
            <div class="col">
              <div class="text-caption text-grey">{{ $t('dashboard.totalOrders') }}</div>
              <div class="text-h4 text-warning q-mt-sm">{{ stats.orders.total }}</div>
              <div class="text-caption">
                <q-icon name="trending_up" color="positive" size="14px" />
                <span class="text-positive q-ml-xs">{{ stats.orders.growth }}%</span>
                <span class="text-grey q-ml-xs">{{ $t('dashboard.growth') }}</span>
              </div>
            </div>
            <q-avatar size="60px" color="warning" text-color="white" icon="shopping_cart" />
          </q-card-section>
        </q-card>
      </div>

      <!-- 总收入 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card bordered class="stat-card">
          <q-card-section class="row items-center">
            <div class="col">
              <div class="text-caption text-grey">{{ $t('dashboard.totalRevenue') }}</div>
              <div class="text-h4 text-info q-mt-sm">¥{{ formatNumber(stats.revenue.total) }}</div>
              <div class="text-caption">
                <q-icon name="trending_up" color="positive" size="14px" />
                <span class="text-positive q-ml-xs">{{ stats.revenue.growth }}%</span>
                <span class="text-grey q-ml-xs">{{ $t('dashboard.growth') }}</span>
              </div>
            </div>
            <q-avatar size="60px" color="info" text-color="white" icon="attach_money" />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 图表区域 -->
    <div class="row q-col-gutter-md q-mb-md">
      <!-- 访问趋势图 -->
      <div class="col-12 col-md-8">
        <q-card bordered class="chart-card">
          <q-card-section>
            <div class="text-subtitle1 text-weight-bold">{{ $t('dashboard.visitTrend') }}</div>
          </q-card-section>
          <q-card-section>
            <div ref="visitChartRef" style="height: 300px"></div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 设备分布 -->
      <div class="col-12 col-md-4">
        <q-card bordered class="chart-card">
          <q-card-section>
            <div class="text-subtitle1 text-weight-bold">{{ $t('dashboard.deviceDistribution') }}</div>
          </q-card-section>
          <q-card-section>
            <div ref="deviceChartRef" style="height: 300px"></div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 第二行图表 -->
    <div class="row q-col-gutter-md q-mb-md">
      <!-- 用户增长 -->
      <div class="col-12 col-md-4">
        <q-card bordered class="chart-card">
          <q-card-section>
            <div class="text-subtitle1 text-weight-bold">{{ $t('dashboard.userGrowth') }}</div>
          </q-card-section>
          <q-card-section>
            <div ref="userGrowthChartRef" style="height: 250px"></div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 订单统计 -->
      <div class="col-12 col-md-4">
        <q-card bordered class="chart-card">
          <q-card-section>
            <div class="text-subtitle1 text-weight-bold">{{ $t('dashboard.orderStats') }}</div>
          </q-card-section>
          <q-card-section>
            <div ref="orderChartRef" style="height: 250px"></div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 地理分布 -->
      <div class="col-12 col-md-4">
        <q-card bordered class="chart-card">
          <q-card-section>
            <div class="text-subtitle1 text-weight-bold">{{ $t('dashboard.geoDistribution') }}</div>
          </q-card-section>
          <q-card-section>
            <div class="geo-list">
              <div v-for="(item, index) in geoData" :key="index" class="geo-item">
                <div class="row items-center">
                  <div class="col-6">
                    <q-icon name="place" :color="getGeoColor(index)" size="16px" />
                    <span class="q-ml-xs">{{ item.province }}</span>
                  </div>
                  <div class="col-4">
                    <q-linear-progress :value="item.percent / 100" :color="getGeoColor(index)" class="q-mt-xs" />
                  </div>
                  <div class="col-2 text-right">
                    <span class="text-weight-bold">{{ item.percent }}%</span>
                  </div>
                </div>
              </div>
            </div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 最新动态 -->
    <div class="row q-col-gutter-md">
      <div class="col-12 col-md-6">
        <q-card bordered>
          <q-card-section>
            <div class="text-subtitle1 text-weight-bold">{{ $t('dashboard.latestUsers') }}</div>
          </q-card-section>
          <q-card-section>
            <q-list separator>
              <q-item v-for="user in latestUsers" :key="user.id">
                <q-item-section avatar>
                  <q-avatar color="primary" text-color="white" size="40px">
                    {{ user.username?.charAt(0)?.toUpperCase() }}
                  </q-avatar>
                </q-item-section>
                <q-item-section>
                  <q-item-label>{{ user.username }}</q-item-label>
                  <q-item-label caption>{{ user.email }}</q-item-label>
                </q-item-section>
                <q-item-section side>
                  <q-item-label caption>{{ formatTimeAgo(user.created_at) }}</q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>

      <div class="col-12 col-md-6">
        <q-card bordered>
          <q-card-section>
            <div class="text-subtitle1 text-weight-bold">{{ $t('dashboard.latestOrders') }}</div>
          </q-card-section>
          <q-card-section>
            <q-list separator>
              <q-item v-for="order in latestOrders" :key="order.id">
                <q-item-section avatar>
                  <q-avatar color="warning" text-color="white" size="40px">
                    <q-icon name="shopping_bag" />
                  </q-avatar>
                </q-item-section>
                <q-item-section>
                  <q-item-label>订单 #{{ order.id }}</q-item-label>
                  <q-item-label caption>{{ order.product }}</q-item-label>
                </q-item-section>
                <q-item-section side>
                  <q-item-label class="text-weight-bold text-warning">¥{{ order.amount }}</q-item-label>
                  <q-item-label caption>{{ formatTimeAgo(order.created_at) }}</q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file DataDashboardPage.vue
 * @description 数据统计大屏页面
 * @date 2026-04-04
 */

import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useDataDashboard } from '@erp-new-frontend-monorepo/composables/src/useDataDashboard';;

const { t: $t } = useI18n();

// 模板 DOM 引用
const visitChartRef = ref<HTMLDivElement | null>(null);
const deviceChartRef = ref<HTMLDivElement | null>(null);
const userGrowthChartRef = ref<HTMLDivElement | null>(null);
const orderChartRef = ref<HTMLDivElement | null>(null);

// 业务逻辑解构
const {
  timeRange,
  timeRangeOptions,
  stats,
  geoData,
  latestUsers,
  latestOrders,
  formatNumber,
  formatTimeAgo,
  getGeoColor,
  loadDashboardData,
  initCharts,
  handleResize,
  disposeCharts,
} = useDataDashboard();

// 生命周期
onMounted(() => {
  void loadDashboardData();
  initCharts({
    visitChartEl: visitChartRef.value,
    deviceChartEl: deviceChartRef.value,
    userGrowthChartEl: userGrowthChartRef.value,
    orderChartEl: orderChartRef.value,
  });
  window.addEventListener('resize', handleResize);
});

onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
  disposeCharts();
});
</script>

<style scoped>
.data-dashboard {
  background: #f5f5f5;
  min-height: 100vh;
}

.stat-card {
  transition: transform 0.2s, box-shadow 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.chart-card {
  height: 100%;
}

.geo-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.geo-item {
  padding: 4px 0;
}
</style>
