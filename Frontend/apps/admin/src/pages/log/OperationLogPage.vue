<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('log.operationLog') }}</div>

    <!-- 搜索筛选 -->
    <AdvancedSearch
      v-model="filters"
      :show-user="true"
      :show-module="true"
      :show-action="true"
      :show-date-range="true"
      :show-date-shortcuts="true"
      :keyword-placeholder="$t('log.searchPlaceholder')"
      :user-label="$t('log.operator')"
      :module-label="$t('log.module')"
      :action-label="$t('log.action')"
      :date-range-label="$t('common.dateRange')"
      :user-options="userOptions"
      :module-options="moduleOptions"
      :field-options="[]"
      :action-options="actionOptions"
      keyword-class="col-12 col-sm-4"
      @search="handleSearch"
      @reset="handleReset"
    />

    <!-- 工具栏 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center justify-between">
        <div class="text-subtitle1">
          {{ $t('log.totalRecords', { count: pagination.rowsNumber }) }}
        </div>
        <div class="q-gutter-sm">
          <q-btn flat :label="$t('common.refresh')" icon="refresh" @click="loadLogs" />
          <q-btn flat color="primary" :label="$t('common.export')" icon="download" @click="handleExport" />
        </div>
      </q-card-section>
    </q-card>

    <!-- 日志表格 -->
    <q-card bordered>
      <q-table
        :rows="logs"
        :columns="columns"
        row-key="id"
        :loading="loading"
        :pagination="pagination"
        @request="onTableRequest"
      >
        <!-- 操作者 -->
        <template v-slot:body-cell-operator="props">
          <q-td :props="props">
            <div class="flex items-center">
              <q-avatar size="28px" color="primary" text-color="white" class="q-mr-sm">
                {{ props.row.operator?.substring(0, 1).toUpperCase() || '?' }}
              </q-avatar>
              <span>{{ props.row.operator || '-' }}</span>
            </div>
          </q-td>
        </template>

        <!-- 模块 -->
        <template v-slot:body-cell-module="props">
          <q-td :props="props">
            <q-badge color="secondary" :label="props.row.module" />
          </q-td>
        </template>

        <!-- 操作 -->
        <template v-slot:body-cell-action="props">
          <q-td :props="props">
            <q-badge :color="getActionColor(props.row.action)" :label="props.row.action" />
          </q-td>
        </template>

        <!-- 资源 -->
        <template v-slot:body-cell-resource="props">
          <q-td :props="props">
            <code class="text-grey-7">{{ props.row.resource || '-' }}</code>
          </q-td>
        </template>

        <!-- IP -->
        <template v-slot:body-cell-ip="props">
          <q-td :props="props">
            <code class="text-grey-7">{{ props.row.ip || '-' }}</code>
          </q-td>
        </template>

        <!-- 详情 -->
        <template v-slot:body-cell-detail="props">
          <q-td :props="props">
            <q-btn flat dense size="sm" color="primary" :label="$t('common.view')" @click="viewDetail(props.row)" />
          </q-td>
        </template>

        <!-- 时间 -->
        <template v-slot:body-cell-created_at="props">
          <q-td :props="props">
            {{ formatDateTime(props.row.created_at) }}
          </q-td>
        </template>

        <!-- 加载状态 -->
        <template v-slot:loading>
          <q-spinner-dots />
        </template>

        <!-- 空状态 -->
        <template v-slot:no-data="{ message }">
          <div class="full-width row justify-center q-pa-xl text-grey-6">
            <q-icon name="assignment" size="64px" class="q-mb-md" />
            <div>{{ message || $t('table.noData') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 详情弹窗 -->
    <q-dialog v-model="showDetailDialog">
      <q-card style="min-width: 600px; max-width: 800px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('log.operationDetail') }}</div>
          <q-space />
          <q-btn flat round icon="close" @click="showDetailDialog = false" />
        </q-card-section>

        <q-separator />

        <q-card-section v-if="currentLog">
          <q-list dense>
            <q-item>
              <q-item-section>
                <q-item-label caption>{{ $t('log.operator') }}</q-item-label>
                <q-item-label class="text-body1">{{ currentLog.operator }}</q-item-label>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>
                <q-item-label caption>{{ $t('log.module') }}</q-item-label>
                <q-item-label class="text-body1">{{ currentLog.module }}</q-item-label>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>
                <q-item-label caption>{{ $t('log.action') }}</q-item-label>
                <q-item-label class="text-body1">
                  <q-badge :color="getActionColor(currentLog.action)" :label="currentLog.action" />
                </q-item-label>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>
                <q-item-label caption>{{ $t('log.resource') }}</q-item-label>
                <q-item-label class="text-body1"><code>{{ currentLog.resource }}</code></q-item-label>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>
                <q-item-label caption>{{ $t('log.ip') }}</q-item-label>
                <q-item-label class="text-body1"><code>{{ currentLog.ip }}</code></q-item-label>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>
                <q-item-label caption>{{ $t('log.time') }}</q-item-label>
                <q-item-label class="text-body1">{{ formatDateTime(currentLog.created_at) }}</q-item-label>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>
                <q-item-label caption>{{ $t('log.request_params') }}</q-item-label>
                <q-item-label class="text-body1">
                  <pre class="code-block">{{ formatJson(currentLog.request_params) }}</pre>
                </q-item-label>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>
                <q-item-label caption>{{ $t('log.response_result') }}</q-item-label>
                <q-item-label class="text-body1">
                  <pre class="code-block">{{ formatJson(currentLog.response_result) }}</pre>
                </q-item-label>
              </q-item-section>
            </q-item>
          </q-list>
        </q-card-section>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import AdvancedSearch from '@/components/AdvancedSearch/Main.vue';
import { useOperationLog } from '@/composables/useOperationLog';

const {
  loading,
  logs,
  showDetailDialog,
  currentLog,
  filters,
  pagination,
  userOptions,
  moduleOptions,
  actionOptions,
  columns,
  getActionColor,
  formatDateTime,
  formatJson,
  loadLogs,
  onTableRequest,
  handleSearch,
  handleReset,
  viewDetail,
  handleExport,
} = useOperationLog();
</script>

<style scoped>
/* 暗色主题适配 */
.body--dark .q-card {
  background: #1e1e1e;
  border-color: #2d2d2d;
}

.body--dark .q-table {
  background: #1e1e1e;
  color: #ffffff;
}

code {
  background: rgba(255, 255, 255, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
}

.body--dark code {
  background: rgba(255, 255, 255, 0.1);
}

.code-block {
  background: rgba(0, 0, 0, 0.2);
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
  font-size: 12px;
  margin: 8px 0;
  white-space: pre-wrap;
  word-break: break-all;
}

.body--dark .code-block {
  background: rgba(0, 0, 0, 0.4);
}
</style>
