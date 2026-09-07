/**
 * @file useExportTaskManager.ts
 * @description 导出任务管理页面业务逻辑 composable
 * @date 2026-07-08
 */

import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { logger } from '@/utils/logger';
import { httpClient } from '@/utils/alova';
import type { ExportTask } from '@/types/importExport';

export function useExportTaskManager() {
  const $q = useQuasar();

  // ── 状态 ──
  const loading = ref(false);
  const tasks = ref<ExportTask[]>([]);
  const statusFilter = ref<string | null>(null);
  const viewMode = ref<'all' | 'processing' | 'completed'>('all');

  // ── 筛选选项 ──
  const statusOptions = [
    { label: '等待中', value: 'pending' },
    { label: '处理中', value: 'processing' },
    { label: '已完成', value: 'completed' },
    { label: '失败', value: 'failed' },
    { label: '已取消', value: 'cancelled' },
  ];

  // ── 表格列定义 ──
  const columns = [
    { name: 'name', label: '任务名称', field: 'name', align: 'left' as const, sortable: true },
    { name: 'status', label: '状态', field: 'status', align: 'center' as const, sortable: true },
    { name: 'progress', label: '进度', field: 'progress', align: 'center' as const, sortable: true },
    { name: 'format', label: '格式', field: 'format', align: 'center' as const },
    { name: 'fileSize', label: '文件大小', field: 'fileSize', align: 'right' as const, sortable: true },
    { name: 'created_at', label: '创建时间', field: 'created_at', align: 'left' as const, sortable: true },
    { name: 'actions', label: '操作', field: 'actions', align: 'center' as const },
  ];

  // ── 计算属性 ──
  const filteredTasks = computed(() => {
    let result = tasks.value;
    if (viewMode.value === 'processing') {
      result = result.filter((t) => t.status === 'pending' || t.status === 'processing');
    } else if (viewMode.value === 'completed') {
      result = result.filter((t) => t.status === 'completed' || t.status === 'failed' || t.status === 'cancelled');
    }
    if (statusFilter.value) {
      result = result.filter((t) => t.status === statusFilter.value);
    }
    return result;
  });

  const completedTasks = computed(() => tasks.value.filter((t) => t.status === 'completed'));

  // ── 工具函数 ──
  function getFormatIcon(format: string): string {
    const icons: Record<string, string> = {
      xlsx: 'table_chart',
      csv: 'grid_on',
      json: 'data_object',
    };
    return icons[format] ?? 'description';
  }

  function getStatusColor(status: string): string {
    const colors: Record<string, string> = {
      pending: 'grey',
      processing: 'blue',
      completed: 'green',
      failed: 'red',
      cancelled: 'orange',
    };
    return colors[status] ?? 'grey';
  }

  function getStatusLabel(status: string): string {
    const labels: Record<string, string> = {
      pending: '等待中',
      processing: '处理中',
      completed: '已完成',
      failed: '失败',
      cancelled: '已取消',
    };
    return labels[status] ?? status;
  }

  function formatFileSize(bytes?: number): string {
    if (!bytes) return '-';
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  function formatDateTime(dateStr?: string): string {
    if (!dateStr) return '-';
    return new Date(dateStr).toLocaleString('zh-CN');
  }

  // ── 业务操作 ──
  function loadData() {
    loading.value = true;
    try {
      // 修复 (fix-plan-20260806 P20): 已对接真实 API
      httpClient.get<ExportTask[]>('/export/tasks/list').then((resp) => {
        tasks.value = resp?.data || [];
      }).catch((error) => {
        logger.error('【加载导出任务失败】', error);
        $q.notify({ type: 'negative', message: '加载导出任务失败' });
      }).finally(() => {
        loading.value = false;
      });
    } catch (error) {
      logger.error('【加载导出任务失败】', error);
      $q.notify({ type: 'negative', message: '加载导出任务失败' });
      loading.value = false;
    }
  }

  function handleRefresh() {
    loadData();
    $q.notify({ type: 'positive', message: '刷新成功' });
  }

  function handleDownload(task: ExportTask) {
    if (task.downloadUrl) {
      window.open(task.downloadUrl, '_blank');
    } else {
      $q.notify({ type: 'warning', message: '文件暂不可用' });
    }
  }

  function handleCancel(task: ExportTask) {
    $q.dialog({
      title: '确认取消',
      message: `确定要取消任务「${task.name}」吗？`,
      cancel: { label: '取消', flat: true },
      ok: { label: '确定', color: 'negative' },
    }).onOk(() => {
      // 修复 (fix-plan-20260806 P20): 已对接真实 API
      httpClient.post<void>(`/export/tasks/${task.id}/cancel`).then(() => {
        task.status = 'cancelled';
        $q.notify({ type: 'positive', message: '任务已取消' });
      }).catch((error) => {
        logger.error('【取消任务失败】', error);
        $q.notify({ type: 'negative', message: '取消任务失败' });
      });
    });
  }

  function handleDelete(task: ExportTask) {
    $q.dialog({
      title: '确认删除',
      message: `确定要删除任务「${task.name}」吗？此操作不可撤销。`,
      cancel: { label: '取消', flat: true },
      ok: { label: '删除', color: 'negative' },
    }).onOk(() => {
      // 修复 (fix-plan-20260806 P20): 已对接真实 API
      httpClient.delete<void>(`/export/tasks/${task.id}`).then(() => {
        tasks.value = tasks.value.filter((t) => t.id !== task.id);
        $q.notify({ type: 'positive', message: '任务已删除' });
      }).catch((error) => {
        logger.error('【删除任务失败】', error);
        $q.notify({ type: 'negative', message: '删除任务失败' });
      });
    });
  }

  function handleClearCompleted() {
    $q.dialog({
      title: '确认清空',
      message: '确定要清空所有已完成的任务吗？此操作不可撤销。',
      cancel: { label: '取消', flat: true },
      ok: { label: '清空', color: 'negative' },
    }).onOk(() => {
      // 修复 (fix-plan-20260806 P20): 已对接真实 API
      httpClient.post<void>('/export/tasks/cleanup').then(() => {
        tasks.value = tasks.value.filter((t) => t.status !== 'completed');
        $q.notify({ type: 'positive', message: '已清空完成的任务' });
      }).catch((error) => {
        logger.error('【清空任务失败】', error);
        $q.notify({ type: 'negative', message: '清空任务失败' });
      });
    });
  }

  // ── 对外暴露 ──
  return {
    // 状态
    loading,
    tasks,
    statusFilter,
    viewMode,
    // 选项/列
    statusOptions,
    columns,
    // 计算属性
    filteredTasks,
    completedTasks,
    // 工具函数
    getFormatIcon,
    getStatusColor,
    getStatusLabel,
    formatFileSize,
    formatDateTime,
    // 业务操作
    loadData,
    handleRefresh,
    handleDownload,
    handleCancel,
    handleDelete,
    handleClearCompleted,
  };
}
