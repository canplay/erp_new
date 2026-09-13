<template>
  <q-page class="q-pa-md">
    <!-- 页面标题和刷新按钮 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">{{ $t('dashboard.title') }}</div>
      <q-space />
      <q-btn flat color="primary" icon="refresh" :label="$t('common.refresh')" @click="handleRefresh" :loading="loading" />
    </div>

    <!-- 统计卡片 - 现代风格 -->
    <div class="row q-col-gutter-md q-mb-lg">
      <!-- 总用户数 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card class="stat-card-modern" bordered>
          <q-card-section class="row items-center no-wrap">
            <div class="col">
              <div class="stat-card-modern__label">{{ $t('dashboard.total_users') }}</div>
              <div class="stat-card-modern__value primary-gradient">
                {{ statistics.total_users || 0 }}
              </div>
              <div class="stat-card-modern__trend stat-card-modern__trend--up">
                <q-icon name="trending_up" size="16px" />
                <span>+{{ statistics.new_users_today || 0 }} {{ $t('dashboard.newToday') }}</span>
              </div>
            </div>
            <div class="stat-card-modern__icon" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%)">
              <q-icon name="people" />
            </div>
          </q-card-section>
          <div class="stat-card-modern__accent primary-gradient-accent"></div>
        </q-card>
      </div>

      <!-- 活跃用户 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card class="stat-card-modern" bordered>
          <q-card-section class="row items-center no-wrap">
            <div class="col">
              <div class="stat-card-modern__label">{{ $t('dashboard.active_users') }}</div>
              <div class="stat-card-modern__value" style="color: #51cf66">
                {{ statistics.active_users || 0 }}
              </div>
              <div class="stat-card-modern__trend">
                <q-icon name="access_time" size="16px" />
                <span>{{ $t('dashboard.monthlyActive') }}</span>
              </div>
            </div>
            <div class="stat-card-modern__icon" style="background: linear-gradient(135deg, #51cf66 0%, #40c057 100%)">
              <q-icon name="person_pin" />
            </div>
          </q-card-section>
          <div class="stat-card-modern__accent" style="background: linear-gradient(90deg, #51cf66 0%, transparent 100%)"></div>
        </q-card>
      </div>

      <!-- 登录尝试 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card class="stat-card-modern" bordered>
          <q-card-section class="row items-center no-wrap">
            <div class="col">
              <div class="stat-card-modern__label">{{ $t('dashboard.login_attempts') }}</div>
              <div class="stat-card-modern__value" style="color: #339af0">
                {{ statistics.login_attempts || 0 }}
              </div>
              <div class="stat-card-modern__trend">
                <q-icon name="calendar_today" size="16px" />
                <span>{{ $t('dashboard.thisMonth') }}</span>
              </div>
            </div>
            <div class="stat-card-modern__icon" style="background: linear-gradient(135deg, #339af0 0%, #228be6 100%)">
              <q-icon name="login" />
            </div>
          </q-card-section>
          <div class="stat-card-modern__accent" style="background: linear-gradient(90deg, #339af0 0%, transparent 100%)"></div>
        </q-card>
      </div>

      <!-- 失败登录 -->
      <div class="col-12 col-sm-6 col-md-3">
        <q-card class="stat-card-modern" bordered>
          <q-card-section class="row items-center no-wrap">
            <div class="col">
              <div class="stat-card-modern__label">{{ $t('dashboard.failed_logins') }}</div>
              <div class="stat-card-modern__value" style="color: #ff6b6b">
                {{ statistics.failed_logins || 0 }}
              </div>
              <div class="stat-card-modern__trend stat-card-modern__trend--warning">
                <q-icon name="security" size="16px" />
                <span>{{ $t('dashboard.needAttention') }}</span>
              </div>
            </div>
            <div class="stat-card-modern__icon" style="background: linear-gradient(135deg, #ff6b6b 0%, #f03e3e 100%)">
              <q-icon name="warning" />
            </div>
          </q-card-section>
          <div class="stat-card-modern__accent" style="background: linear-gradient(90deg, #ff6b6b 0%, transparent 100%)"></div>
        </q-card>
      </div>
    </div>

    <!-- 快捷操作和最近登录 -->
    <div class="row q-col-gutter-md">
      <!-- 快捷操作 -->
      <div class="col-12 col-md-4">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('dashboard.quickActions') }}</div>
            <div class="row q-col-gutter-sm">
              <div class="col-6">
                <q-btn
                  outline
                  color="primary"
                  class="full-width"
                  icon="person_add"
                  :label="$t('dashboard.addUser')"
                  @click="$router.push('/users')"
                />
              </div>
              <div class="col-6">
                <q-btn
                  outline
                  color="primary"
                  class="full-width"
                  icon="admin_panel_settings"
                  :label="$t('dashboard.roleManagement')"
                  @click="$router.push('/roles')"
                />
              </div>
              <div class="col-6">
                <q-btn
                  outline
                  color="info"
                  class="full-width"
                  icon="history"
                  :label="$t('dashboard.loginLog')"
                  @click="$router.push('/logs/login')"
                />
              </div>
              <div class="col-6">
                <q-btn
                  outline
                  color="info"
                  class="full-width"
                  icon="track_changes"
                  :label="$t('dashboard.operationLog')"
                  @click="$router.push('/logs/operation')"
                />
              </div>
              <div class="col-6">
                <q-btn
                  outline
                  color="warning"
                  class="full-width"
                  icon="settings"
                  :label="$t('menu.systemSettings')"
                  @click="$router.push('/settings')"
                />
              </div>
              <div class="col-6">
                <q-btn
                  outline
                  color="info"
                  class="full-width"
                  icon="person"
                  :label="$t('dashboard.personalCenter')"
                  @click="$router.push('/profile')"
                />
              </div>
              <div class="col-6">
                <q-btn
                  outline
                  color="purple"
                  class="full-width"
                  icon="notifications"
                  :label="$t('dashboard.notifications')"
                  @click="$router.push('/notifications')"
                />
              </div>
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 最近登录 -->
      <div class="col-12 col-md-8">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('dashboard.recentLogins') }}</div>
            <q-table
              :rows="recentLogins"
              :columns="loginColumns"
              row-key="id"
              flat
              bordered
              :rows-per-page-options="[5]"
              hide-pagination
            >
              <template v-slot:body-cell-status="props">
                <q-td :props="props">
                  <q-badge
                    :color="props.value ? 'positive' : 'negative'"
                    :label="props.value ? $t('loginLog.success') : $t('loginLog.failed')"
                  />
                </q-td>
              </template>
              <template v-slot:body-cell-time="props">
                <q-td :props="props">
                  {{ formatTime(props.value) }}
                </q-td>
              </template>
            </q-table>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 数据可视化区域 - 使用 ECharts -->
    <div class="row q-col-gutter-md q-mt-md">
      <!-- 用户增长趋势 -->
      <div class="col-12 col-md-6">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('dashboard.userGrowthTrend') }}</div>
            <ECharts
              :config="{
                type: 'line',
                data: userGrowthData,
                height: '280px',
                showGrid: true,
                showLegend: false,
                areaFill: true,
              }"
            />
          </q-card-section>
        </q-card>
      </div>

      <!-- 登录类型分布 -->
      <div class="col-12 col-md-6">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('dashboard.loginTypeDistribution') }}</div>
            <ECharts
              :config="{
                type: 'pie',
                data: loginTypeData,
                height: '280px',
                showLegend: true,
                pieRadius: '60%',
              }"
            />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="text-center q-pa-xl">
      <q-spinner-dots size="50px" color="primary" />
      <div class="q-mt-md text-grey-6">{{ $t('common.loading') }}</div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useDashboard } from '@erp-new-frontend-monorepo/composables/src/useDashboard';;
import ECharts from '@erp-new-frontend-monorepo/components/src/ECharts/Main.vue';

const {
  loading, statistics, userGrowthData, loginTypeData, recentLogins, loginColumns,
  handleRefresh, formatTime, initData,
} = useDashboard();

onMounted(() => { void initData(); });
</script>

<style scoped>
/* ============ 现代统计卡片样式 ============ */
.stat-card-modern {
  border-radius: 12px;
  padding: 0;
  position: relative;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.stat-card-modern:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.15);
}

.stat-card-modern .q-card__section {
  padding: 20px;
}

.stat-card-modern__label {
  font-size: 14px;
  color: #666;
  margin-bottom: 8px;
}

.stat-card-modern__value {
  font-size: 32px;
  font-weight: 700;
  line-height: 1.2;
  margin-bottom: 8px;
}

.primary-gradient {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.stat-card-modern__icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  color: white;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.stat-card-modern__trend {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: #666;
}

.stat-card-modern__trend--up {
  color: #51cf66;
}

.stat-card-modern__trend--warning {
  color: #ffc107;
}

.stat-card-modern__accent {
  height: 4px;
  background: linear-gradient(90deg, #667eea 0%, transparent 100%);
}

/* ============ 快捷操作按钮 ============ */
.q-btn--outline {
  border-radius: 8px;
  transition: all 0.25s ease;
}

.q-btn--outline:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* ============ 表格样式 ============ */
.q-table {
  border-radius: 12px;
  overflow: hidden;
}

.q-table thead tr {
  background: #f8f9fa;
}

.body--dark .q-table thead tr {
  background: #252525;
}

/* ============ 暗色主题适配 ============ */
.body--dark .stat-card-modern {
  background: #1e1e1e;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.body--dark .stat-card-modern:hover {
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
}

.body--dark .stat-card-modern__label {
  color: #b0b0b0;
}

.body--dark .stat-card-modern__trend {
  color: #b0b0b0;
}

.body--dark .q-card {
  background: #1e1e1e;
  border-color: #2d2d2d;
}

.body--dark .q-card .text-h6 {
  color: #ffffff;
}

.body--dark .q-table {
  background: #1e1e1e;
}

.body--dark .q-table thead tr {
  background: #252525;
}

.body--dark .q-table thead th {
  color: #ffffff;
}

.body--dark .q-table tbody tr:hover {
  background: rgba(255, 255, 255, 0.05);
}

.body--dark .q-table tbody td {
  color: #b0b0b0;
}

.body--dark .q-btn--outline {
  border-color: #404040;
}

/* ============ 动画 ============ */
@keyframes countUp {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.stat-card-modern {
  animation: fadeInUp 0.5s ease forwards;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.col-12.col-sm-6.col-md-3:nth-child(1) .stat-card-modern {
  animation-delay: 0s;
}

.col-12.col-sm-6.col-md-3:nth-child(2) .stat-card-modern {
  animation-delay: 0.1s;
}

.col-12.col-sm-6.col-md-3:nth-child(3) .stat-card-modern {
  animation-delay: 0.2s;
}

.col-12.col-sm-6.col-md-3:nth-child(4) .stat-card-modern {
  animation-delay: 0.3s;
}
</style>