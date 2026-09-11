<template>
  <q-page class="q-pa-md">
    <div class="row q-col-gutter-md">
      <!-- 搜索栏 -->
      <div class="col-12">
        <q-card>
          <q-card-section class="row q-gutter-md items-center">
            <q-input
              v-model="searchKeyword"
              dense
              outlined
              :placeholder="$t('common.searchPlaceholder')"
              style="width: 300px"
              @keyup.enter="handleSearch"
            >
              <template #prepend>
                <q-icon name="search" />
              </template>
            </q-input>

            <q-select
              v-model="filterType"
              :options="typeOptions"
              dense
              outlined
              clearable
              emit-value
              map-options
              style="width: 150px"
              label="类型"
            />

            <q-select
              v-model="filterStatus"
              :options="statusOptions"
              dense
              outlined
              clearable
              emit-value
              map-options
              style="width: 150px"
              :label="$t('common.status')"
            />

            <q-space />

            <q-btn
              color="positive"
              icon="add"
              label="新建报表"
              @click="handleCreate"
            />
          </q-card-section>
        </q-card>
      </div>

      <!-- 报表列表 -->
      <div class="col-12">
        <q-table
          :rows="reportList"
          :columns="columns"
          :loading="loading"
          :pagination="pagination"
          row-key="id"
          @request="onTableRequest"
        >
          <template #body-cell-reportType="props">
            <q-td :props="props">
              <q-icon
                :name="getTypeIcon(props.row.reportType)"
                size="sm"
                class="q-mr-xs"
              />
              {{ getTypeLabel(props.row.reportType) }}
            </q-td>
          </template>

          <template #body-cell-status="props">
            <q-td :props="props">
              <q-badge
                :color="getStatusColor(props.row.status)"
                :label="getStatusLabel(props.row.status)"
              />
            </q-td>
          </template>

          <template #body-cell-created_at="props">
            <q-td :props="props">
              {{ props.value ? formatTs(props.value) : '-' }}
            </q-td>
          </template>

          <template #body-cell-actions="props">
            <q-td :props="props">
              <q-btn-group flat>
                <q-btn
                  flat
                  dense
                  color="primary"
                  icon="bar_chart"
                  @click="handlePreview(props.row)"
                >
                  <q-tooltip>预览</q-tooltip>
                </q-btn>
                <q-btn
                  flat
                  dense
                  color="positive"
                  icon="play_arrow"
                  @click="handleExecute(props.row)"
                >
                  <q-tooltip>执行</q-tooltip>
                </q-btn>
                <q-btn
                  flat
                  dense
                  color="primary"
                  icon="download"
                  @click="handleExport(props.row)"
                >
                  <q-tooltip>导出</q-tooltip>
                </q-btn>
                <q-btn
                  flat
                  dense
                  color="negative"
                  icon="delete"
                  @click="handleDelete(props.row)"
                >
                  <q-tooltip>删除</q-tooltip>
                </q-btn>
              </q-btn-group>
            </q-td>
          </template>

          <template #no-data>
            <EmptyState
              icon="assessment"
              message="暂无报表"
              action-text="新建报表"
              @action="handleCreate"
            />
          </template>
        </q-table>
      </div>
    </div>

    <!-- 新建报表对话框 -->
    <q-dialog v-model="showDialog" persistent>
      <q-card style="min-width: 500px; max-width: 800px">
        <q-card-section>
          <div class="text-h6">{{ isEdit ? t('report.editReport') : t('report.createReport') }}</div>
        </q-card-section>

        <q-card-section>
          <q-form>
            <q-input
              v-model="formData.name"
              :label="$t('report.reportName')"
              outlined
              :rules="[val => !!val || $t('report.enterReportName')]"
            />

            <q-input
              v-model="formData.description"
              :label="$t('common.description')"
              outlined
              type="textarea"
              rows="3"
              class="q-mt-md"
            />

            <q-select
              v-model="formData.reportType"
              :options="typeOptions"
              emit-value
              map-options
              :label="$t('report.reportType')"
              outlined
              class="q-mt-md"
              :disable="isEdit"
            />

            <div v-if="isEdit" class="q-mt-md">
              <q-select
                v-model="formData.status"
                :options="statusOptions"
                emit-value
                map-options
                :label="$t('common.status')"
                outlined
              />
            </div>
          </q-form>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn color="primary" :label="$t('common.save')" @click="handleSubmit" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 导出选项对话框 -->
    <q-dialog v-model="showExportDialog">
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">{{ $t('report.exportReport') }}</div>
        </q-card-section>

        <q-card-section>
          <q-form>
            <q-select
              v-model="exportFormat"
              :options="exportOptions"
              emit-value
              map-options
              :label="$t('report.exportFormat')"
              outlined
            />
          </q-form>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn color="primary" :label="$t('common.export')" @click="confirmExport" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 确认删除对话框 -->
    <ConfirmDialog
      v-model="showDeleteDialog"
      :title="$t('report.confirmDeleteReport')"
      :message="$t('report.deleteReportMessage')"
      @confirm="confirmDelete"
    />
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { useReportStore } from '@/stores/report';
import type { Report, ReportStatus } from '@/api/report';
import { exportReport } from '@/api/report';
import { formatDate } from '@/utils/format';
import EmptyState from '@/components/EmptyState.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';

const { t } = useI18n();
const $q = useQuasar();
const router = useRouter();
const reportStore = useReportStore();
const loading = computed(() => reportStore.loading);
const reportList = computed(() => reportStore.reports);

const searchKeyword = ref('');
const filterType = ref<string | null>(null);
const filterStatus = ref<ReportStatus | null>(null);

const showDialog = ref(false);
const showExportDialog = ref(false);
const showDeleteDialog = ref(false);
const isEdit = ref(false);
const currentReport = ref<Report | null>(null);
const exportFormat = ref('excel');

const formData = ref({
  name: '',
  description: '',
  reportType: 'table',
  status: 'draft',
});

// ============ 列表配置 ============
const columns = [
  { name: 'name', label: '名称', field: 'name', align: 'left' as const, sortable: true },
  { name: 'reportType', label: '类型', field: 'reportType', align: 'center' as const },
  { name: 'description', label: '描述', field: 'description', align: 'left' as const },
  { name: 'status', label: '状态', field: 'status', align: 'center' as const },
  { name: 'created_at', label: '创建时间', field: 'created_at', align: 'center' as const, sortable: true },
  { name: 'actions', label: '操作', field: 'actions', align: 'center' as const },
];

const pagination = ref({
  page: 1,
  rowsPerPage: 20,
  rowsNumber: 0,
});

// ============ 选项 ============
const typeOptions = [
  { label: '表格', value: 'table' },
  { label: '图表', value: 'chart' },
  { label: '仪表盘', value: 'dashboard' },
];

const statusOptions = [
  { label: '草稿', value: 'draft' },
  { label: '已发布', value: 'published' },
  { label: '已归档', value: 'archived' },
];

const exportOptions = [
  { label: 'Excel', value: 'excel' },
  { label: 'CSV', value: 'csv' },
  { label: 'PDF', value: 'pdf' },
];

// ============ 函数 ============
function getTypeIcon(type: string): string {
  const icons: Record<string, string> = {
    table: 'table_chart',
    chart: 'bar_chart',
    dashboard: 'dashboard',
  };
  return icons[type] || 'assessment';
}

function getTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    table: '表格',
    chart: '图表',
    dashboard: '仪表盘',
  };
  return labels[type] || type;
}

function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    draft: 'grey',
    published: 'positive',
    archived: 'blue-grey',
  };
  return colors[status] || 'grey';
}

function getStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    draft: '草稿',
    published: '已发布',
    archived: '已归档',
  };
  return labels[status] || status;
}

/** 时间戳格式化（兼容 epoch 秒与毫秒/ISO） */
function formatTs(value: string | number): string {
  if (!value) return '-';
  // ISO 字符串（非纯数字）直接解析
  if (typeof value === 'string' && !/^\d+$/.test(value.trim())) {
    const d = new Date(value);
    return d.getTime() ? formatDate(d.getTime()) : String(value);
  }
  const num = typeof value === 'string' ? Number(value) : value;
  if (Number.isNaN(num)) return String(value);
  return formatDate(num < 1e12 ? num * 1000 : num);
}

async function onTableRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  const params: Record<string, unknown> = {
    page: props.pagination.page,
    page_size: props.pagination.rowsPerPage,
  };
  if (filterType.value) params.reportType = filterType.value;
  if (filterStatus.value) params.status = filterStatus.value;
  await reportStore.fetchReports(params);
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  pagination.value.rowsNumber = reportStore.total;
}

function handleSearch() {
  pagination.value.page = 1;
  void onTableRequest({ pagination: pagination.value });
}

function handleCreate() {
  isEdit.value = false;
  formData.value = { name: '', description: '', reportType: 'table', status: 'draft' };
  showDialog.value = true;
}


function handlePreview(row: Report) {
  void router.push({
    name: 'ReportView',
    params: { id: row.id },
    query: { name: row.name },
  });
}

async function handleExecute(row: Report) {
  try {
    await reportStore.runReport(row.id);
    $q.notify({ type: 'positive', message: '报表执行已启动' });
  } catch {
    $q.notify({ type: 'negative', message: '执行失败' });
  }
}

function handleExport(row: Report) {
  currentReport.value = row;
  exportFormat.value = 'excel';
  showExportDialog.value = true;
}

async function confirmExport() {
  if (!currentReport.value) return;
  try {
    await exportReport(currentReport.value.id, exportFormat.value as 'csv' | 'excel' | 'pdf');
    $q.notify({ type: 'positive', message: '导出任务已启动' });
    showExportDialog.value = false;
  } catch {
    $q.notify({ type: 'negative', message: '导出失败' });
  }
}

function handleDelete(row: Report) {
  currentReport.value = row;
  showDeleteDialog.value = true;
}

async function handleSubmit() {
  try {
    if (isEdit.value && currentReport.value) {
      await reportStore.editReport(currentReport.value.id, {
        name: formData.value.name,
        description: formData.value.description,
        status: formData.value.status as 'draft' | 'published' | 'archived',
      });
      $q.notify({ type: 'positive', message: '更新成功' });
    } else {
      await reportStore.addReport({
        name: formData.value.name,
        description: formData.value.description,
        reportType: formData.value.reportType as 'table' | 'chart' | 'dashboard',
      });
      $q.notify({ type: 'positive', message: t('report.reportCreated') });
    }
    showDialog.value = false;
    void onTableRequest({ pagination: pagination.value });
  } catch {
    $q.notify({ type: 'negative', message: t('common.saveFailed') });
  }
}

async function confirmDelete() {
  if (!currentReport.value) return;
  try {
    await reportStore.removeReport(currentReport.value.id);
    $q.notify({ type: 'positive', message: t('report.reportDeleted') });
    showDeleteDialog.value = false;
    void onTableRequest({ pagination: pagination.value });
  } catch {
    $q.notify({ type: 'negative', message: t('report.reportDeleteFailed') });
  }
}

// ============ 生命周期 ============
onMounted(() => {
  void onTableRequest({ pagination: pagination.value });
});
</script>
