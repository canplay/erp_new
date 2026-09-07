<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('sensitiveAudit.title') }}</div>

    <!-- 统计卡片 -->
    <div class="row q-mb-md q-gutter-md">
      <q-card flat bordered class="col" style="min-width: 180px">
        <q-card-section class="row items-center">
          <q-icon name="pending_actions" size="40px" color="warning" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('sensitiveAudit.pending') }}</div>
            <div class="text-h6">{{ statistics.pending }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 180px">
        <q-card-section class="row items-center">
          <q-icon name="check_circle" size="40px" color="positive" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('sensitiveAudit.success') }}</div>
            <div class="text-h6">{{ statistics.success }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 180px">
        <q-card-section class="row items-center">
          <q-icon name="error" size="40px" color="negative" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('sensitiveAudit.failed') }}</div>
            <div class="text-h6">{{ statistics.failed }}</div>
          </div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col" style="min-width: 180px">
        <q-card-section class="row items-center">
          <q-icon name="history" size="40px" color="info" class="q-mr-md" />
          <div>
            <div class="text-caption text-grey">{{ $t('sensitiveAudit.today') }}</div>
            <div class="text-h6">{{ statistics.today }}</div>
          </div>
        </q-card-section>
      </q-card>
    </div>

    <!-- 筛选栏 -->
    <q-card flat bordered class="q-mb-md">
      <q-card-section class="row q-gutter-md">
        <q-select
          v-model="filters.operation_type"
          :options="operation_type_options"
          :label="$t('sensitiveAudit.operation_type')"
          dense
          outlined
          clearable
          emit-value
          map-options
          style="min-width: 150px"
          @update:model-value="loadRecords"
        />
        <q-select
          v-model="filters.confirmType"
          :options="confirmTypeOptions"
          :label="$t('sensitiveAudit.confirmType')"
          dense
          outlined
          clearable
          emit-value
          map-options
          style="min-width: 150px"
          @update:model-value="loadRecords"
        />
        <q-select
          v-model="filters.confirmStatus"
          :options="confirmStatusOptions"
          :label="$t('sensitiveAudit.status')"
          dense
          outlined
          clearable
          emit-value
          map-options
          style="min-width: 150px"
          @update:model-value="loadRecords"
        />
        <q-space />
        <q-btn flat :icon="matRefresh" :label="$t('common.refresh')" @click="resetFilters" />
      </q-card-section>
    </q-card>

    <!-- 操作记录列表 -->
    <q-card flat bordered>
      <q-table
        :rows="recordList"
        :columns="columns"
        row-key="id"
        :loading="loading"
        flat
        :pagination="{ rowsPerPage: 15 }"
        @request="onRequest"
      >
        <!-- 操作类型 -->
        <template #body-cell-operation_type="{ row }">
          <q-td>
            <q-badge color="primary" :label="row.operation_type" />
          </q-td>
        </template>

        <!-- 确认方式 -->
        <template #body-cell-confirmType="{ row }">
          <q-td>
            <q-icon
              :name="getConfirmTypeIcon(row.confirmType)"
              :color="getConfirmTypeColor(row.confirmType)"
              size="20px"
            />
            {{ getConfirmTypeLabel(row.confirmType) }}
          </q-td>
        </template>

        <!-- 确认状态 -->
        <template #body-cell-confirmStatus="{ row }">
          <q-td>
            <q-badge :color="getStatusColor(row.confirmStatus)" :label="getStatusLabel(row.confirmStatus)" />
          </q-td>
        </template>

        <!-- 操作状态 -->
        <template #body-cell-operationStatus="{ row }">
          <q-td>
            <q-badge
              :color="row.operationStatus === 'success' ? 'positive' : 'negative'"
              :label="row.operationStatus === 'success' ? $t('sensitiveAudit.operationSuccess') : $t('sensitiveAudit.operationFailed')"
            />
          </q-td>
        </template>

        <!-- 操作时间 -->
        <template #body-cell-created_at="{ row }">
          <q-td>
            <div class="text-body2">{{ row.created_at }}</div>
          </q-td>
        </template>

        <!-- 操作 -->
        <template #body-cell-actions="{ row }">
          <q-btn flat dense round :icon="matVisibility" @click="viewDetail(row)">
            <q-tooltip>{{ $t('common.view') }}</q-tooltip>
          </q-btn>
          <q-btn
            v-if="row.confirmType === 'admin' && row.confirmStatus === 'pending'"
            flat
            dense
            round
            :icon="matCheck"
            color="positive"
            @click="approveRecord(row, true)"
          >
            <q-tooltip>{{ $t('sensitiveAudit.approve') }}</q-tooltip>
          </q-btn>
          <q-btn
            v-if="row.confirmType === 'admin' && row.confirmStatus === 'pending'"
            flat
            dense
            round
            :icon="matClose"
            color="negative"
            @click="approveRecord(row, false)"
          >
            <q-tooltip>{{ $t('sensitiveAudit.reject') }}</q-tooltip>
          </q-btn>
        </template>

        <!-- 加载状态 -->
        <template #loading>
          <q-inner-loading showing color="primary" />
        </template>

        <!-- 空状态 -->
        <template #no-data>
          <div class="full-width row flex-center text-grey-6 q-pa-lg">
            <q-icon name="security" size="48px" class="q-mb-sm" />
            <div>{{ $t('sensitiveAudit.noRecords') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 详情对话框 -->
    <q-dialog v-model="detailDialogVisible">
      <q-card style="min-width: 500px; max-width: 700px">
        <q-card-section>
          <div class="text-h6">{{ $t('sensitiveAudit.detail') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section v-if="currentRecord">
          <q-list dense>
            <q-item>
              <q-item-section>{{ $t('sensitiveAudit.operation_type') }}:</q-item-section>
              <q-item-section side>
                <q-badge color="primary" :label="currentRecord.operation_type" />
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('sensitiveAudit.operationDesc') }}:</q-item-section>
              <q-item-section side>{{ currentRecord.operationDesc || '-' }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('sensitiveAudit.user') }}:</q-item-section>
              <q-item-section side>{{ currentRecord.userName }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('sensitiveAudit.confirmType') }}:</q-item-section>
              <q-item-section side>{{ getConfirmTypeLabel(currentRecord.confirmType) }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('sensitiveAudit.confirmStatus') }}:</q-item-section>
              <q-item-section side>
                <q-badge :color="getStatusColor(currentRecord.confirmStatus)" :label="getStatusLabel(currentRecord.confirmStatus)" />
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('sensitiveAudit.operationStatus') }}:</q-item-section>
              <q-item-section side>
                <q-badge :color="currentRecord.operationStatus === 'success' ? 'positive' : 'negative'" :label="currentRecord.operationStatus" />
              </q-item-section>
            </q-item>
            <q-item v-if="currentRecord.targetResource">
              <q-item-section>{{ $t('sensitiveAudit.targetResource') }}:</q-item-section>
              <q-item-section side>{{ currentRecord.targetResourceName || currentRecord.targetResource }}</q-item-section>
            </q-item>
            <q-item v-if="currentRecord.ipAddress">
              <q-item-section>IP:</q-item-section>
              <q-item-section side>{{ currentRecord.ipAddress }}</q-item-section>
            </q-item>
            <q-item v-if="currentRecord.failureReason">
              <q-item-section>{{ $t('sensitiveAudit.failureReason') }}:</q-item-section>
              <q-item-section side class="text-negative">{{ currentRecord.failureReason }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>{{ $t('sensitiveAudit.createTime') }}:</q-item-section>
              <q-item-section side>{{ currentRecord.created_at }}</q-item-section>
            </q-item>
          </q-list>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.close')" v-close-popup />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file SensitiveAuditPage.vue
 * @description 敏感操作审计页面
 * @date 2026-04-06
 */

import { onMounted } from 'vue';
import { useSensitiveAudit } from '@/composables/useSensitiveAudit';

// Material Icons 常量（仅模板使用）
const matRefresh = 'refresh';
const matVisibility = 'visibility';
const matCheck = 'check';
const matClose = 'close';

// 从 composable 解构所有业务逻辑
const {
  loading,
  detailDialogVisible,
  currentRecord,
  recordList,
  filters,
  statistics,
  operation_type_options,
  confirmTypeOptions,
  confirmStatusOptions,
  columns,
  loadRecords,
  loadStatistics,
  onRequest,
  resetFilters,
  getConfirmTypeIcon,
  getConfirmTypeColor,
  getConfirmTypeLabel,
  getStatusColor,
  getStatusLabel,
  viewDetail,
  approveRecord,
} = useSensitiveAudit();

// 生命周期
onMounted(() => {
  void loadRecords();
  void loadStatistics();
});
</script>
