/**
 * @file PermissionChangeLogPage.vue
 * @description 权限变更日志页面
 * @date 2026-04-04
 * @features
 * - 展示权限变更历史
 * - 支持多种筛选条件
 * - 支持导出功能
 */

<template>
  <q-page class="q-pa-md permission-change-log-page">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg text-weight-bold">
      <q-icon name="history" class="q-mr-sm" />
      权限变更日志
    </div>

    <!-- 权限变更日志表格 -->
    <PermissionChangeLogTable />

    <!-- 统计卡片 -->
    <div class="row q-mt-md q-col-gutter-md">
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered>
          <q-card-section class="text-center">
            <div class="text-h4 text-primary">{{ stats.total }}</div>
            <div class="text-caption text-grey">总变更次数</div>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered>
          <q-card-section class="text-center">
            <div class="text-h4 text-positive">{{ stats.add }}</div>
            <div class="text-caption text-grey">权限添加</div>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered>
          <q-card-section class="text-center">
            <div class="text-h4 text-negative">{{ stats.remove }}</div>
            <div class="text-caption text-grey">权限移除</div>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-6 col-md-3">
        <q-card flat bordered>
          <q-card-section class="text-center">
            <div class="text-h4 text-warning">{{ stats.update }}</div>
            <div class="text-caption text-grey">权限更新</div>
          </q-card-section>
        </q-card>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file PermissionChangeLogPage.vue
 * @description 权限变更日志页面组件
 */

import { ref, onMounted } from 'vue';
import PermissionChangeLogTable from '@erp-new-frontend-monorepo/components/src/PermissionChangeLogTable.vue';
import { listPermissionChangeLogs } from '@/api/permission';
import type { PermissionChangeLog } from '@/types/permission';
import { logger } from '@/utils/logger';

// ============ 状态 ============

/** 统计数据 */
const stats = ref({
  total: 0,
  add: 0,
  remove: 0,
  update: 0,
});

// ============ 方法 ============

/**
 * @brief 加载统计数据
 */
async function loadStats() {
  try {
    // 获取全部数据用于统计（实际应该用单独的API）
    const response = await listPermissionChangeLogs({
      page: 1,
      page_size: 1000,
    });

    const respData = response as { list?: unknown[]; data?: { list?: unknown[]; data?: unknown[] } };
    const dataObj = respData.data as { list?: unknown[]; data?: unknown[] } | undefined;
    const logs = respData.list || dataObj?.list || dataObj?.data || [];

    stats.value = {
      total: logs.length,
      add: (logs as PermissionChangeLog[]).filter((l) => l.changeType === 'add').length,
      remove: (logs as PermissionChangeLog[]).filter((l) => l.changeType === 'remove').length,
      update: (logs as PermissionChangeLog[]).filter((l) => l.changeType === 'update').length,
    };
  } catch (error) {
    logger.error('【加载统计数据失败】', error);
  }
}

/**
 * @brief 加载统计数据（异步，不处理结果）
 */
function loadStatsAsync() {
  void loadStats();
}

// ============ 生命周期 ============

onMounted(() => {
  loadStatsAsync();
});
</script>

<style scoped>
.permission-change-log-page {
  max-width: 1600px;
  margin: 0 auto;
}
</style>

