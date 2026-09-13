<template>
  <div class="api-log-viewer">
    <!-- 页面标题 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">
        <q-icon name="api" class="q-mr-sm" />
        {{ $t('apiLog.pageTitle') }}
      </div>
      <q-space />
      <q-btn flat color="grey" icon="refresh" @click="handleRefresh" />
      <q-btn color="primary" icon="download" :label="$t('common.export')" @click="handleExport" />
    </div>

    <!-- 统计概览 -->
    <div class="row q-col-gutter-md q-mb-md">
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="stat-card">
          <q-card-section class="text-center">
            <div class="text-h4 text-primary">{{ stats.totalCalls }}</div>
            <div class="text-caption">{{ $t('apiLog.totalCalls') }}</div>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="stat-card">
          <q-card-section class="text-center">
            <div class="text-h4 text-positive">{{ stats.successCalls }}</div>
            <div class="text-caption">{{ $t('apiLog.success') }}</div>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="stat-card">
          <q-card-section class="text-center">
            <div class="text-h4 text-negative">{{ stats.failedCalls }}</div>
            <div class="text-caption">{{ $t('apiLog.failed') }}</div>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="stat-card">
          <q-card-section class="text-center">
            <div class="text-h4" :class="errorRateClass">{{ stats.errorRate }}%</div>
            <div class="text-caption">{{ $t('apiLog.errorRate') }}</div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 响应时间统计 -->
    <div class="row q-col-gutter-md q-mb-md">
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="stat-card">
          <q-card-section class="text-center">
            <div class="text-h5">{{ stats.avg_response_time }}ms</div>
            <div class="text-caption">{{ $t('apiLog.avgResponse') }}</div>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="stat-card">
          <q-card-section class="text-center">
            <div class="text-h5 text-info">{{ stats.p95_response_time }}ms</div>
            <div class="text-caption">{{ $t('apiLog.p95Response') }}</div>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="stat-card">
          <q-card-section class="text-center">
            <div class="text-h5 text-warning">{{ stats.p99ResponseTime }}ms</div>
            <div class="text-caption">{{ $t('apiLog.p99Response') }}</div>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered class="stat-card">
          <q-card-section class="text-center">
            <div class="text-h5">{{ stats.qps.toFixed(2) }}</div>
            <div class="text-caption">{{ $t('apiLog.qps') }}</div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 筛选条件 -->
    <q-card flat bordered class="q-mb-md">
      <q-card-section>
        <div class="row q-col-gutter-md items-center">
          <div class="col-12 col-sm-6 col-md-2">
            <q-select
              v-model="filter.method"
              :options="methodOptions"
              outlined
              dense
              clearable
              emit-value
              map-options
              :label="$t('common.httpMethod')"
            />
          </div>
          <div class="col-12 col-sm-6 col-md-3">
            <q-input
              v-model="filter.pathKeyword"
              outlined
              dense
              clearable
              :label="$t('common.pathKeyword')"
            />
          </div>
          <div class="col-12 col-sm-6 col-md-2">
            <q-select
              v-model="filter.status_code"
              :options="statusOptions"
              outlined
              dense
              clearable
              emit-value
              map-options
              :label="$t('common.statusCode')"
            />
          </div>
          <div class="col-12 col-sm-6 col-md-2">
            <q-input
              v-model.number="filter.min_response_time"
              outlined
              dense
              type="number"
              :label="$t('common.minResponse')"
            />
          </div>
          <div class="col-12 col-sm-6 col-md-2">
            <q-input
              v-model.number="filter.max_response_time"
              outlined
              dense
              type="number"
              :label="$t('common.maxResponse')"
            />
          </div>
          <div class="col-12 col-md-1">
            <q-btn
              color="primary"
              icon="search"
              :label="$t('common.filter')"
              @click="handleFilter"
            />
          </div>
        </div>

        <div class="row q-col-gutter-md items-center q-mt-md">
          <div class="col-12 col-sm-6 col-md-3">
            <q-input
              v-model="filter.start_time"
              outlined
              dense
              type="datetime-local"
              :label="$t('common.startTime')"
            />
          </div>
          <div class="col-12 col-sm-6 col-md-3">
            <q-input
              v-model="filter.end_time"
              outlined
              dense
              type="datetime-local"
              :label="$t('common.endTime')"
            />
          </div>
          <div class="col-12 col-md-3">
            <q-checkbox v-model="filter.errorsOnly" :label="$t('common.showErrorsOnly')" />
          </div>
          <div class="col-12 col-md-3">
            <q-btn flat color="grey" icon="clear" :label="$t('common.clearFilter')" @click="handleClearFilter" />
          </div>
        </div>
      </q-card-section>
    </q-card>

    <!-- 日志列表 -->
    <q-card flat bordered>
      <q-card-section class="q-pa-none">
        <q-table
          :rows="filteredLogs"
          :columns="columns"
          row-key="id"
          flat
          :pagination="{ rowsPerPage: 15 }"
          :loading="loading"
          @row-click="handleRowClick"
        >
          <!-- 方法 -->
          <template v-slot:body-cell-method="props">
            <q-td :props="props">
              <q-badge
                :color="getMethodColor(props.row.method)"
                text-color="white"
                :label="props.row.method"
              />
            </q-td>
          </template>

          <!-- 路径 -->
          <template v-slot:body-cell-path="props">
            <q-td :props="props">
              <div class="ellipsis" style="max-width: 300px;">
                {{ props.row.path }}
              </div>
            </q-td>
          </template>

          <!-- 状态 -->
          <template v-slot:body-cell-status_code="props">
            <q-td :props="props">
              <q-badge
                :color="getStatusColor(props.row.status_code)"
                text-color="white"
                :label="props.row.status_code"
              />
            </q-td>
          </template>

          <!-- 响应时间 -->
          <template v-slot:body-cell-response_time="props">
            <q-td :props="props">
              <span :class="getResponseTimeClass(props.row.response_time)">
                {{ props.row.response_time }}ms
              </span>
            </q-td>
          </template>

          <!-- 用户 -->
          <template v-slot:body-cell-user="props">
            <q-td :props="props">
              <span v-if="props.row.username">{{ props.row.username }}</span>
              <span v-else class="text-grey">-</span>
            </q-td>
          </template>

          <!-- 时间 -->
          <template v-slot:body-cell-timestamp="props">
            <q-td :props="props">
              {{ formatTime(props.row.timestamp) }}
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>

    <!-- 日志详情对话框 -->
    <q-dialog v-model="showDetailDialog" maximized>
      <q-card>
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('apiLog.dialogTitle') }}</div>
          <q-space />
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-separator />

        <q-card-section v-if="selectedLog" class="log-detail-content">
          <!-- 基本信息 -->
          <div class="text-subtitle1 q-mb-sm">{{ $t('apiLog.basicInfo') }}</div>
          <q-card flat bordered class="q-mb-md">
            <q-card-section>
              <div class="row q-col-gutter-md">
                <div class="col-6 col-md-3">
                  <div class="text-caption text-grey">{{ $t('apiLog.requestMethod') }}</div>
                  <q-badge :color="getMethodColor(selectedLog.method)" text-color="white">
                    {{ selectedLog.method }}
                  </q-badge>
                </div>
                <div class="col-6 col-md-3">
                  <div class="text-caption text-grey">{{ $t('apiLog.statusCode') }}</div>
                  <q-badge :color="getStatusColor(selectedLog.status_code)" text-color="white">
                    {{ selectedLog.status_code }}
                  </q-badge>
                </div>
                <div class="col-6 col-md-3">
                  <div class="text-caption text-grey">{{ $t('apiLog.responseTime') }}</div>
                  <div class="text-h6">{{ selectedLog.response_time }}ms</div>
                </div>
                <div class="col-6 col-md-3">
                  <div class="text-caption text-grey">{{ $t('apiLog.user') }}</div>
                  <div>{{ selectedLog.username || '-' }}</div>
                </div>
                <div class="col-6 col-md-3">
                  <div class="text-caption text-grey">{{ $t('apiLog.clientIp') }}</div>
                  <div>{{ selectedLog.clientIp }}</div>
                </div>
                <div class="col-6 col-md-3">
                  <div class="text-caption text-grey">{{ $t('apiLog.requestId') }}</div>
                  <div class="text-caption">{{ selectedLog.requestId }}</div>
                </div>
                <div class="col-12 col-md-6">
                  <div class="text-caption text-grey">{{ $t('apiLog.requestPath') }}</div>
                  <div class="text-body2">{{ selectedLog.path }}</div>
                </div>
              </div>
            </q-card-section>
          </q-card>

          <!-- 请求信息 -->
          <div class="text-subtitle1 q-mb-sm">{{ $t('apiLog.requestInfo') }}</div>
          <q-card flat bordered class="q-mb-md">
            <q-card-section>
              <div class="text-caption text-grey q-mb-xs">{{ $t('apiLog.requestHeaders') }}</div>
              <q-card class="bg-grey-9" flat>
                <q-card-section>
                  <pre class="text-white text-caption" style="white-space: pre-wrap;">{{ formatJson(selectedLog.headers) }}</pre>
                </q-card-section>
              </q-card>

              <div class="text-caption text-grey q-mb-xs q-mt-md">{{ $t('apiLog.requestBody') }}</div>
              <q-card class="bg-grey-9" flat>
                <q-card-section>
                  <pre class="text-white text-caption" style="white-space: pre-wrap;">{{ selectedLog.request_body || $t('apiLog.empty') }}</pre>
                </q-card-section>
              </q-card>

              <div class="row q-mt-md">
                <div class="col-6">
                  <div class="text-caption text-grey">{{ $t('apiLog.requestSize') }}</div>
                  <div>{{ formatBytes(selectedLog.requestSize) }}</div>
                </div>
                <div class="col-6">
                  <div class="text-caption text-grey">{{ $t('apiLog.queryParams') }}</div>
                  <div class="text-caption">{{ formatJson(selectedLog.queryParams) || $t('apiLog.noParams') }}</div>
                </div>
              </div>
            </q-card-section>
          </q-card>

          <!-- 响应信息 -->
          <div class="text-subtitle1 q-mb-sm">{{ $t('apiLog.responseInfo') }}</div>
          <q-card flat bordered class="q-mb-md">
            <q-card-section>
              <div class="text-caption text-grey q-mb-xs">{{ $t('apiLog.responseHeaders') }}</div>
              <q-card class="bg-grey-9" flat>
                <q-card-section>
                  <pre class="text-white text-caption" style="white-space: pre-wrap;">{{ formatJson(selectedLog.responseHeaders) }}</pre>
                </q-card-section>
              </q-card>

              <div class="text-caption text-grey q-mb-xs q-mt-md">{{ $t('apiLog.responseBody') }}</div>
              <q-card class="bg-grey-9" flat>
                <q-card-section>
                  <pre class="text-white text-caption" style="white-space: pre-wrap; max-height: 300px; overflow: auto;">{{ selectedLog.response_body || $t('apiLog.empty') }}</pre>
                </q-card-section>
              </q-card>

              <div class="row q-mt-md">
                <div class="col-6">
                  <div class="text-caption text-grey">{{ $t('apiLog.responseSize') }}</div>
                  <div>{{ formatBytes(selectedLog.responseSize) }}</div>
                </div>
                <div class="col-6" v-if="selectedLog.error">
                  <div class="text-caption text-grey">{{ $t('apiLog.errorMessage') }}</div>
                  <div class="text-negative">{{ selectedLog.error }}</div>
                </div>
              </div>
            </q-card-section>
          </q-card>
        </q-card-section>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { useApiLogViewer } from '@erp-new-frontend-monorepo/composables/src/useApiLogViewer';

const {
  methodOptions, statusOptions,
  loading, showDetailDialog, selectedLog, filter, stats,
  filteredLogs, errorRateClass, columns,
  getMethodColor, getStatusColor, getResponseTimeClass, formatTime, formatBytes, formatJson,
  handleRefresh, handleFilter, handleClearFilter, handleRowClick, handleExport,
} = useApiLogViewer();
</script>

<style scoped>
.api-log-viewer {
  padding: 16px;
}

.stat-card {
  border-radius: 8px;
  transition: all 0.3s;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.log-detail-content {
  max-height: calc(100vh - 150px);
  overflow-y: auto;
}
</style>
