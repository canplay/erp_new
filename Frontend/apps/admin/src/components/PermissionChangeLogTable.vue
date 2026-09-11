/**
 * @file PermissionChangeLogTable.vue
 * @description 权限变更日志表格组件
 * @date 2026-04-04
 * @features
 * - 展示权限变更历史
 * - 支持时间范围筛选
 * - 支持操作类型筛选
 * - 支持角色和操作人筛选
 */

<template>
  <div class="permission-change-log-table">
    <!-- 筛选工具栏 -->
    <q-card flat bordered class="q-mb-md">
      <q-card-section class="q-py-sm">
        <div class="row q-col-gutter-md items-center">
          <!-- 开始日期 -->
          <div class="col-12 col-sm-6 col-md-3">
            <q-input
              v-model="start_date_str"
              outlined
              dense
              :label="$t('common.startTime')"
              readonly
            >
              <template v-slot:append>
                <q-icon name="event" class="cursor-pointer">
                  <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                    <q-date v-model="start_date_str" />
                  </q-popup-proxy>
                </q-icon>
              </template>
            </q-input>
          </div>

          <!-- 结束日期 -->
          <div class="col-12 col-sm-6 col-md-3">
            <q-input
              v-model="end_date_str"
              outlined
              dense
              label="结束日期"
              readonly
            >
              <template v-slot:append>
                <q-icon name="event" class="cursor-pointer">
                  <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                    <q-date v-model="end_date_str" />
                  </q-popup-proxy>
                </q-icon>
              </template>
            </q-input>
          </div>

          <!-- 角色名称 -->
          <div class="col-12 col-sm-6 col-md-2">
            <q-select
              v-model="filters.role_name"
              :options="roleOptions"
              outlined
              dense
              clearable
              label="角色"
              emit-value
              map-options
            />
          </div>

          <!-- 操作类型 -->
          <div class="col-12 col-sm-6 col-md-2">
            <q-select
              v-model="filters.changeType"
              :options="changeTypeOptions"
              outlined
              dense
              clearable
              label="操作类型"
              emit-value
              map-options
            />
          </div>

          <!-- 搜索按钮 -->
          <div class="col-12 col-md-2">
            <q-btn color="primary" :label="$t('common.search')" icon="search" @click="loadLogs" />
            <q-btn color="grey" :label="$t('common.reset')" icon="refresh" flat class="q-ml-sm" @click="resetFilters" />
          </div>
        </div>
      </q-card-section>
    </q-card>

    <!-- 日志表格 -->
    <q-card flat bordered>
      <q-table
        :rows="logs"
        :columns="columns"
        :loading="loading"
        row-key="id"
        flat
        bordered
        :pagination="pagination"
        @request="onRequest"
      >
        <!-- 加载状态 -->
        <template v-slot:loading>
          <q-inner-loading showing color="primary" />
        </template>

        <!-- 无数据 -->
        <template v-slot:no-data="{ message }">
          <div class="full-width row justify-center q-pa-xl text-grey-6">
            <q-icon name="history" size="64px" class="q-mb-md" />
            <div>{{ message || '没有可用日志' }}</div>
          </div>
        </template>

        <!-- 操作类型 -->
        <template v-slot:body-cell-changeType="props">
          <q-td :props="props">
            <q-badge
              :color="getChangeTypeColor(props.value)"
              text-color="white"
              :label="getChangeTypeLabel(props.value)"
            />
          </q-td>
        </template>

        <!-- 权限 -->
        <template v-slot:body-cell-permission="props">
          <q-td :props="props">
            <code class="text-primary">{{ props.value }}</code>
          </q-td>
        </template>

        <!-- 时间 -->
        <template v-slot:body-cell-created_at="props">
          <q-td :props="props">
            {{ formatDateTime(props.value) }}
          </q-td>
        </template>

        <!-- 操作 -->
        <template v-slot:body-cell-actions="props">
          <q-td :props="props">
            <q-btn
              flat
              dense
              color="primary"
              icon="visibility"
              @click="viewDetail(props.row)"
            >
              <q-tooltip>查看详情</q-tooltip>
            </q-btn>
          </q-td>
        </template>
      </q-table>
    </q-card>

    <!-- 详情对话框 -->
    <q-dialog v-model="showDetailDialog" persistent>
      <q-card style="min-width: 500px">
        <q-card-section class="row items-center">
          <div class="text-h6">权限变更详情</div>
          <q-space />
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-separator />

        <q-card-section v-if="currentLog">
          <q-list separator>
            <q-item>
              <q-item-section avatar>角色</q-item-section>
              <q-item-section>
                <code>{{ currentLog.role_name }}</code>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section avatar>操作人</q-item-section>
              <q-item-section>{{ currentLog.operator }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section avatar>操作类型</q-item-section>
              <q-item-section>
                <q-badge
                  :color="getChangeTypeColor(currentLog.changeType)"
                  text-color="white"
                  :label="getChangeTypeLabel(currentLog.changeType)"
                />
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section avatar>权限</q-item-section>
              <q-item-section>
                <code class="text-primary">{{ currentLog.permission }}</code>
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section avatar>变更时间</q-item-section>
              <q-item-section>{{ formatDateTime(currentLog.created_at) }}</q-item-section>
            </q-item>
          </q-list>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat color="grey" :label="$t('common.close')" v-close-popup />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * @file PermissionChangeLogTable.vue
 * @description 权限变更日志表格组件
 */

import { ref, onMounted } from 'vue';
import { logger } from '@/utils/logger';
import type { QTableProps } from 'quasar';
import type { PermissionChangeLog } from '@/types/permission';
import { listPermissionChangeLogs } from '@/api/permission';

// ============ 类型定义 ============

interface Filters {
  role_name?: string;
  changeType?: string;
  operator?: string;
}

interface Pagination {
  sortBy: string;
  descending: boolean;
  page: number;
  rowsPerPage: number;
  rowsNumber?: number;
}

// ============ 状态 ============

/** 日志列表 */
const logs = ref<PermissionChangeLog[]>([]);

/** 加载状态 */
const loading = ref(false);

/** 分页配置 */
const pagination = ref<Pagination>({
  sortBy: 'created_at',
  descending: true,
  page: 1,
  rowsPerPage: 20,
  rowsNumber: 0,
});

/** 筛选条件 */
const filters = ref<Filters>({});

/** 日期字符串 */
const start_date_str = ref('');
const end_date_str = ref('');

/** 详情对话框 */
const showDetailDialog = ref(false);

/** 当前查看的日志 */
const currentLog = ref<PermissionChangeLog | null>(null);

// ============ 常量 ============

/** 角色选项 */
const roleOptions = [
  { label: '全部', value: '' },
  { label: '管理员', value: 'admin' },
  { label: '普通用户', value: 'user' },
  { label: 'VIP用户', value: 'vip' },
];

/** 操作类型选项 */
const changeTypeOptions = [
  { label: '全部', value: '' },
  { label: '添加', value: 'add' },
  { label: '移除', value: 'remove' },
  { label: '更新', value: 'update' },
];

/** 表格列定义 */
const columns: QTableProps['columns'] = [
  { name: 'id', label: 'ID', field: 'id', align: 'left', sortable: true },
  { name: 'role_name', label: '角色', field: 'role_name', align: 'left', sortable: true },
  { name: 'operator', label: '操作人', field: 'operator', align: 'left' },
  { name: 'changeType', label: '操作类型', field: 'changeType', align: 'center' },
  { name: 'permission', label: '权限', field: 'permission', align: 'left' },
  { name: 'created_at', label: '变更时间', field: 'created_at', align: 'left', sortable: true },
  { name: 'actions', label: '操作', field: 'actions', align: 'center' },
];

// ============ 方法 ============

/**
 * @brief 加载日志列表
 */
async function loadLogs() {
  loading.value = true;
  try {
    const changeTypeValue = filters.value.changeType;
    const params: {
      page?: number;
      page_size?: number;
      role_name?: string;
      change_type?: 'add' | 'remove' | 'update';
      operator?: string;
      start_date?: string;
      end_date?: string;
    } = {
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    };

    if (filters.value.role_name) {
      params.role_name = filters.value.role_name;
    }
    if (changeTypeValue) {
      params.change_type = changeTypeValue as 'add' | 'remove' | 'update';
    }
    if (filters.value.operator) {
      params.operator = filters.value.operator;
    }
    if (start_date_str.value) {
      params.start_date = start_date_str.value;
    }
    if (end_date_str.value) {
      params.end_date = end_date_str.value;
    }

    const response = await listPermissionChangeLogs(params);
    const respData = response as { list?: PermissionChangeLog[]; data?: { list?: PermissionChangeLog[]; data?: PermissionChangeLog[]; total?: number } };
    const dataObj = respData.data;
    logs.value = respData.list || dataObj?.list || dataObj?.data || [];
    pagination.value.rowsNumber = dataObj?.total || 0;
  } catch (error) {
    logger.error('【加载权限变更日志失败】', error);
    logs.value = [];
  } finally {
    loading.value = false;
  }
}

/**
 * @brief 加载日志列表（异步，不处理结果）
 */
function loadLogsAsync() {
  void loadLogs();
}

/**
 * @brief 分页请求
 */
function onRequest(requestProps: Parameters<NonNullable<QTableProps['onRequest']>>[0]) {
  const { pagination: propsPagination } = requestProps;
  pagination.value.page = propsPagination.page;
  pagination.value.rowsPerPage = propsPagination.rowsPerPage;
  pagination.value.sortBy = propsPagination.sortBy || 'created_at';
  pagination.value.descending = propsPagination.descending ?? true;
  void loadLogs();
}

/**
 * @brief 重置筛选条件
 */
function resetFilters() {
  filters.value = {};
  start_date_str.value = '';
  end_date_str.value = '';
  void loadLogs();
}

/**
 * @brief 查看详情
 */
function viewDetail(log: PermissionChangeLog) {
  currentLog.value = log;
  showDetailDialog.value = true;
}

/**
 * @brief 获取操作类型颜色
 */
function getChangeTypeColor(type: string): string {
  const colors: Record<string, string> = {
    add: 'positive',
    remove: 'negative',
    update: 'warning',
  };
  return colors[type] || 'grey';
}

/**
 * @brief 获取操作类型标签
 */
function getChangeTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    add: '添加',
    remove: '移除',
    update: '更新',
  };
  return labels[type] || type;
}

/**
 * @brief 格式化日期时间
 */
function formatDateTime(dateStr: string): string {
  if (!dateStr) return '-';
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

// ============ 生命周期 ============

onMounted(() => {
  loadLogsAsync();
});
</script>

<style scoped>
code {
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.05);
  font-size: 0.9em;
}

.body--dark code {
  background: rgba(255, 255, 255, 0.1);
}
</style>

