<template>
<q-page class="q-pa-md task-schedule-page">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg text-weight-bold">{{ $t('taskSchedule.title') }}</div>

    <!-- 工具栏 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center q-col-gutter-md">
        <div class="col-12 col-sm-4">
          <q-input
            v-model="filters.keyword"
            dense
            outlined
            :placeholder="$t('common.keyword')"
            clearable
            @keyup.enter="handleSearch"
          >
            <template v-slot:prepend>
              <q-icon name="search" />
            </template>
          </q-input>
        </div>
        <div class="col-12 col-sm-2">
          <q-select
            v-model="filters.status"
            :options="statusOptions"
            :label="$t('taskSchedule.status')"
            dense
            outlined
            clearable
            emit-value
            map-options
          />
        </div>
        <q-space />
        <div class="col-auto">
          <q-btn color="positive" icon="add" :label="$t('taskSchedule.add')" @click="openCreateDialog" />
        </div>
      </q-card-section>
    </q-card>

    <!-- 任务列表 -->
    <q-card bordered>
      <q-table
        :rows="store.tasks"
        :columns="columns"
        row-key="id"
        flat
        :loading="store.loading"
        :pagination="pagination"
        @request="onTableRequest"
      >
        <!-- 任务名称列 -->
        <template v-slot:body-cell-name="props">
          <q-td :props="props">
            <div class="text-weight-medium">{{ props.row.name }}</div>
            <div class="text-caption text-grey">{{ props.row.actionType }}</div>
          </q-td>
        </template>

        <!-- 执行周期列 -->
        <template v-slot:body-cell-cron="props">
          <q-td :props="props">
            <q-chip dense size="sm" color="blue-grey-2" text-color="dark">
              {{ props.row.cron }}
            </q-chip>
            <div class="text-caption text-grey q-mt-xs">{{ getCronDescription(props.row.cron) }}</div>
          </q-td>
        </template>

        <!-- 状态列 -->
        <template v-slot:body-cell-status="props">
          <q-td :props="props">
            <q-badge :color="getStatusColor(props.row.status)" :label="getStatusLabel(props.row.status)" />
          </q-td>
        </template>

        <!-- 执行时间列 -->
        <template v-slot:body-cell-lastRun="props">
          <q-td :props="props">
            <div v-if="props.row.lastRunTime">{{ formatDateTime(props.row.lastRunTime) }}</div>
            <div v-else class="text-grey">-</div>
            <div class="text-caption" :class="getDurationClass(props.row.lastRunDuration)">
              {{ props.row.lastRunDuration ? `耗时: ${props.row.lastRunDuration}ms` : '' }}
            </div>
          </q-td>
        </template>

        <!-- 操作列 -->
        <template v-slot:body-cell-actions="props">
          <q-td :props="props">
            <q-btn
              flat
              dense
              color="primary"
              :label="$t('taskSchedule.execute')"
              :disable="props.row.status === 'running'"
              @click="handleExecute(props.row)"
            />
            <q-btn
              flat
              dense
              :color="props.row.status === 'enabled' ? 'warning' : 'positive'"
              :icon="props.row.status === 'enabled' ? 'pause' : 'play_arrow'"
              @click="handleToggleStatus(props.row)"
            >
              <q-tooltip>{{ props.row.status === 'enabled' ? '暂停' : '启用' }}</q-tooltip>
            </q-btn>
            <q-btn flat dense color="primary" icon="edit" @click="openEditDialog(props.row)">
              <q-tooltip>{{ $t('common.edit') }}</q-tooltip>
            </q-btn>
            <q-btn
              flat
              dense
              color="negative"
              icon="delete"
              @click="handleDelete(props.row)"
            >
              <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
            </q-btn>
          </q-td>
        </template>
      </q-table>
    </q-card>

    <!-- 创建/编辑对话框 -->
    <q-dialog v-model="showDialog" persistent :maximized="$q.screen.lt.md">
      <q-card :style="{ width: $q.screen.lt.md ? '100%' : '600px', maxWidth: '600px' }">
        <q-card-section>
          <div class="text-h6">{{ isEdit ? $t('taskSchedule.edit') : $t('taskSchedule.add') }}</div>
        </q-card-section>
        <q-separator />
        <q-card-section style="max-height: 60vh" class="scroll">
          <q-form class="q-gutter-md">
            <q-input
              v-model="form.name"
              :label="$t('taskSchedule.name')"
              outlined
              :rules="[(val) => !!val || $t('validation.required')]"
            />
            <div class="row q-col-gutter-md">
              <div class="col-12 col-sm-6">
                <q-select
                  v-model="form.type"
                  :options="typeOptions"
                  :label="$t('taskSchedule.type')"
                  outlined
                  emit-value
                  map-options
                />
              </div>
              <div class="col-12 col-sm-6">
                <q-input
                  v-model="form.cron"
                  :label="$t('taskSchedule.cron')"
                  outlined
                  placeholder="0 0 * * * ?"
                  :rules="[(val) => !!val || $t('validation.required')]"
                />
              </div>
            </div>
            <q-input
              v-model="form.endpoint"
              :label="$t('taskSchedule.endpoint')"
              outlined
              placeholder="https://api.example.com/task"
              :rules="[(val) => !!val || $t('validation.required')]"
            />
            <q-select
              v-model="form.status"
              :options="statusOptions"
              :label="$t('taskSchedule.status')"
              outlined
              emit-value
              map-options
            />
            <q-input
              v-model="form.description"
              :label="$t('taskSchedule.description')"
              type="textarea"
              outlined
              rows="3"
            />
            <div class="text-caption text-grey">{{ $t('taskSchedule.cronHint') }}</div>
          </q-form>
        </q-card-section>
        <q-separator />
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showDialog = false" />
          <q-btn color="primary" :label="$t('common.save')" @click="handleSave" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 执行日志对话框 -->
    <q-dialog v-model="showLogDialog">
      <q-card style="min-width: 800px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('taskSchedule.executionLog') }}</div>
          <q-space />
          <q-btn flat round dense icon="close" v-close-popup />
        </q-card-section>
        <q-separator />
        <q-card-section style="max-height: 60vh" class="scroll">
          <q-timeline color="primary">
            <q-timeline-entry
              v-for="log in executionLogs"
              :key="log.id"
              :subtitle="formatDateTime(log.executeTime)"
            >
              <div>
                <q-badge
                  :color="log.success ? 'positive' : 'negative'"
                  :label="log.success ? '成功' : '失败'"
                  class="q-mr-sm"
                />
                <span class="text-grey">耗时: {{ log.duration }}ms</span>
              </div>
              <div v-if="log.error" class="text-negative q-mt-sm">{{ log.error }}</div>
            </q-timeline-entry>
          </q-timeline>
        </q-card-section>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file TaskSchedulePage.vue
 * @description 定时任务调度管理页面
 * @date 2026-04-04
 * @update 2026-05-19 实现实际 API 调用
 * @update 2026-07-07 提取 composable useTaskSchedule
 */

import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useTaskSchedule } from '@erp-new-frontend-monorepo/composables/src/useTaskSchedule';

const $q = useQuasar();
const { t: $t } = useI18n();

const {
  store,
  filters,
  statusOptions,
  typeOptions,
  columns,
  pagination,
  showDialog,
  isEdit,
  form,
  showLogDialog,
  executionLogs,
  getStatusColor,
  getStatusLabel,
  getDurationClass,
  getCronDescription,
  formatDateTime,
  onTableRequest,
  handleSearch,
  openCreateDialog,
  openEditDialog,
  handleSave,
  handleToggleStatus,
  handleExecute,
  handleDelete,
} = useTaskSchedule();
</script>

<style scoped>
.task-schedule-page {
  min-height: 100vh;
}
</style>
