<template>
  <div class="export-task-manager">
    <!-- 页面标题 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">
        <q-icon name="cloud_download" class="q-mr-sm" />
        {{ $t('exportMod.taskManagement') || '导出任务管理' }}
      </div>
      <q-space />
      <q-btn flat color="grey" icon="refresh" :label="$t('common.refresh')" @click="handleRefresh" />
    </div>

    <!-- 筛选选项 -->
    <div class="row q-col-gutter-md q-mb-md">
      <div class="col-12 col-sm-3">
        <q-select
          v-model="statusFilter"
          :options="statusOptions"
          outlined
          dense
          clearable
          emit-value
          map-options
          :placeholder="$t('common.filterByStatus')"
        />
      </div>
      <div class="col-12 col-sm-3">
        <q-btn-toggle
          v-model="viewMode"
          toggle-color="primary"
          :options="[
            { label: '全部', value: 'all' },
            { label: '进行中', value: 'processing' },
            { label: '已完成', value: 'completed' },
          ]"
          unelevated
        />
      </div>
      <div class="col-12 col-sm-6 text-right">
        <q-btn
          flat
          dense
          color="negative"
          icon="delete_sweep"
          :label="$t('common.clearCompleted')"
          :disable="completedTasks.length === 0"
          @click="handleClearCompleted"
        />
      </div>
    </div>

    <!-- 任务列表 -->
    <q-table
      :rows="filteredTasks"
      :columns="columns"
      row-key="id"
      flat
      bordered
      :loading="loading"
      :pagination="{ rowsPerPage: 10 }"
    >
      <!-- 任务名称 -->
      <template v-slot:body-cell-name="props">
        <q-td :props="props">
          <div class="row items-center">
            <q-icon :name="getFormatIcon(props.row.format)" color="primary" size="20px" class="q-mr-sm" />
            <div>
              <div class="text-weight-medium">{{ props.row.name }}</div>
              <div class="text-caption text-grey">{{ props.row.format.toUpperCase() }} 格式</div>
            </div>
          </div>
        </q-td>
      </template>

      <!-- 状态 -->
      <template v-slot:body-cell-status="props">
        <q-td :props="props">
          <q-badge
            :color="getStatusColor(props.row.status)"
            :label="getStatusLabel(props.row.status)"
          />
          <q-linear-progress
            v-if="props.row.status === 'processing'"
            :value="props.row.progress / 100"
            color="primary"
            class="q-mt-xs"
            track-color="grey-3"
          />
        </q-td>
      </template>

      <!-- 进度 -->
      <template v-slot:body-cell-progress="props">
        <q-td :props="props">
          <div v-if="props.row.status === 'processing' || props.row.status === 'completed'">
            <div class="text-weight-medium">{{ props.row.progress }}%</div>
            <div class="text-caption text-grey">
              {{ props.row.exportedCount }} / {{ props.row.totalCount }}
            </div>
          </div>
          <div v-else class="text-grey">-</div>
        </q-td>
      </template>

      <!-- 格式 -->
      <template v-slot:body-cell-format="props">
        <q-td :props="props">
          <q-chip dense size="sm" :color="getStatusColor(props.row.status)" text-color="white">
            {{ props.row.format.toUpperCase() }}
          </q-chip>
        </q-td>
      </template>

      <!-- 文件大小 -->
      <template v-slot:body-cell-fileSize="props">
        <q-td :props="props">
          {{ formatFileSize(props.row.fileSize) }}
        </q-td>
      </template>

      <!-- 创建时间 -->
      <template v-slot:body-cell-created_at="props">
        <q-td :props="props">
          <div class="text-caption">{{ formatDateTime(props.row.created_at) }}</div>
          <div v-if="props.row.completed_at" class="text-caption text-grey">
            完成: {{ formatDateTime(props.row.completed_at) }}
          </div>
        </q-td>
      </template>

      <!-- 操作 -->
      <template v-slot:body-cell-actions="props">
        <q-td :props="props">
          <div class="row q-gutter-xs justify-center">
            <q-btn
              v-if="props.row.status === 'completed'"
              flat
              dense
              round
              icon="download"
              color="positive"
              size="sm"
              @click="handleDownload(props.row)">
              <q-tooltip>下载文件</q-tooltip>
            </q-btn>
            <q-btn
              v-if="props.row.status === 'pending' || props.row.status === 'processing'"
              flat
              dense
              round
              icon="cancel"
              color="warning"
              size="sm"
              @click="handleCancel(props.row)">
              <q-tooltip>取消任务</q-tooltip>
            </q-btn>
            <q-btn
              v-if="props.row.status === 'completed' || props.row.status === 'failed' || props.row.status === 'cancelled'"
              flat
              dense
              round
              icon="delete"
              color="negative"
              size="sm"
              @click="handleDelete(props.row)">
              <q-tooltip>删除任务</q-tooltip>
            </q-btn>
          </div>
        </q-td>
      </template>

      <!-- 空状态 -->
      <template v-slot:no-data>
        <div class="full-width row flex-center text-grey q-pa-lg">
          <q-icon name="cloud_download" size="48px" class="q-mr-sm" />
          <div>
            <div class="text-h6">暂无导出任务</div>
            <div class="text-caption">创建导出任务后将在此显示</div>
          </div>
        </div>
      </template>
    </q-table>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useExportTaskManager } from '@/composables/useExportTaskManager';

const {
  loading,
  statusFilter,
  viewMode,
  statusOptions,
  columns,
  filteredTasks,
  completedTasks,
  getFormatIcon,
  getStatusColor,
  getStatusLabel,
  formatFileSize,
  formatDateTime,
  loadData,
  handleRefresh,
  handleDownload,
  handleCancel,
  handleDelete,
  handleClearCompleted,
} = useExportTaskManager();

onMounted(() => {
  loadData();
});
</script>

<style scoped>
.export-task-manager {
  padding: 24px;
}
</style>
