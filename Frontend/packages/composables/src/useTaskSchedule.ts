/**
 * @file useTaskSchedule.ts
 * @description 定时任务调度管理页面业务逻辑 composable
 */

import { ref, computed, reactive, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import { useScheduledTaskStore } from '@/stores/report';
import {
  createScheduledTask,
  updateScheduledTask,
  deleteScheduledTask,
  triggerScheduledTask,
  pauseScheduledTask,
  resumeScheduledTask,
  type ScheduledTask,
  type ScheduledTaskStatus,
  type ActionType,
} from '@/api/scheduled-task';

// 类型定义
interface ExecutionLog {
  id: number;
  taskId: number;
  executeTime: string;
  duration: number;
  success: boolean;
  error?: string;
}

export function useTaskSchedule() {
  const $q = useQuasar();
  const { t: $t } = useI18n();
  const store = useScheduledTaskStore();

  // ============ 状态 ============
  const pagination = ref({ page: 1, rowsPerPage: 10, rowsNumber: 0 });
  const showDialog = ref(false);
  const showLogDialog = ref(false);
  const isEdit = ref(false);
  const executionLogs = ref<ExecutionLog[]>([]);

  // ============ 筛选 ============
  const filters = reactive({
    keyword: '',
    status: null as 'active' | 'paused' | 'stopped' | null,
  });

  // ============ 表单 ============
  const form = reactive({
    id: '',
    name: '',
    type: 'script',
    cron: '0 0 * * * ?',
    endpoint: '',
    status: 'active',
    description: '',
  });

  // ============ 选项 ============
  const statusOptions = [
    { label: '启用', value: 'active' },
    { label: '暂停', value: 'paused' },
    { label: '停止', value: 'stopped' },
  ];

  const typeOptions = [
    { label: '脚本执行', value: 'script' },
    { label: '工作流', value: 'workflow' },
    { label: '报表', value: 'report' },
    { label: 'Webhook', value: 'webhook' },
  ];

  // ============ 表格列 ============
  const columns = computed(() => [
    { name: 'name', label: $t('taskSchedule.name'), field: 'name', align: 'left' as const },
    { name: 'type', label: $t('taskSchedule.type'), field: 'type', align: 'center' as const },
    { name: 'cron', label: $t('taskSchedule.cron'), field: 'cron', align: 'left' as const },
    { name: 'status', label: $t('taskSchedule.status'), field: 'status', align: 'center' as const },
    { name: 'lastRun', label: $t('taskSchedule.lastRun'), field: 'lastRunTime', align: 'left' as const },
    { name: 'created_at', label: $t('taskSchedule.created_at'), field: 'created_at', align: 'left' as const },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  // ============ 工具函数 ============
  function getStatusColor(status: string): string {
    const colors: Record<string, string> = {
      enabled: 'positive',
      disabled: 'grey',
      running: 'warning',
      active: 'positive',
      paused: 'warning',
      stopped: 'grey',
    };
    return colors[status] || 'grey';
  }

  function getStatusLabel(status: string): string {
    const labels: Record<string, string> = {
      enabled: '启用',
      disabled: '禁用',
      running: '运行中',
      active: '启用',
      paused: '暂停',
      stopped: '停止',
    };
    return labels[status] || status;
  }

  function getDurationClass(duration?: number): string {
    if (!duration) return '';
    if (duration < 1000) return 'text-positive';
    if (duration < 5000) return 'text-warning';
    return 'text-negative';
  }

  function getCronDescription(cron: string): string {
    if (!cron) return '';
    const parts = cron.split(' ');
    if (parts.length !== 5) return cron;
    const min = parts[0];
    const hour = parts[1];
    const dow = parts[4];

    if (min === '*' && hour === '*') return '每分钟执行';
    if (hour === '*') return `每小时第 ${min} 分钟执行`;
    if (min !== undefined && hour !== undefined && min !== '*' && hour !== '*') {
      if (dow === '*' || dow === '?') return `每天 ${hour.padStart(2, '0')}:${min.padStart(2, '0')} 执行`;
      const weekMap: Record<string, string> = { '1': '一', '2': '二', '3': '三', '4': '四', '5': '五', '6': '六', '7': '日' };
      const weekDay = dow != null ? (weekMap[dow] || dow) : '';
      return `每周${weekDay} ${hour.padStart(2, '0')}:${min.padStart(2, '0')} 执行`;
    }
    return cron;
  }

  function formatDateTime(dateStr: string): string {
    if (!dateStr) return '-';
    return new Date(dateStr).toLocaleString('zh-CN');
  }

  // ============ 数据加载 ============
  async function loadTasks() {
    try {
      const params = {
        page: pagination.value.page,
        page_size: pagination.value.rowsPerPage,
        keyword: filters.keyword || undefined,
        status: filters.status || undefined,
      };
      await store.fetchTasks(params);
      pagination.value.rowsNumber = store.total;
    } catch (error) {
      logger.error('【加载任务列表失败】', error);
      $q.notify({ type: 'negative', message: $t('common.loadFailed') });
    }
  }

  function onTableRequest(props: { pagination: { page: number; rowsPerPage: number; rowsNumber?: number } }) {
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
    void loadTasks();
  }

  function handleSearch() {
    pagination.value.page = 1;
    void loadTasks();
  }

  // ============ 弹窗操作 ============
  function openCreateDialog() {
    isEdit.value = false;
    Object.assign(form, {
      id: '',
      name: '',
      type: 'script',
      cron: '0 0 * * * ?',
      endpoint: '',
      status: 'active',
      description: '',
    });
    showDialog.value = true;
  }

  function openEditDialog(task: ScheduledTask) {
    isEdit.value = true;
    Object.assign(form, {
      id: task.id,
      name: task.name,
      type: task.actionType,
      cron: task.cronExpression,
      endpoint: (task.actionParams?.endpoint as string) || '',
      status: task.status,
      description: (task.actionParams?.description as string) || '',
    });
    showDialog.value = true;
  }

  async function handleSave() {
    try {
      if (isEdit.value) {
        await updateScheduledTask(form.id, {
          name: form.name,
          cronExpression: form.cron,
          status: form.status as ScheduledTaskStatus,
          actionParams: {
            endpoint: form.endpoint,
            description: form.description,
          },
        });
      } else {
        await createScheduledTask({
          name: form.name,
          actionType: form.type as ActionType,
          cronExpression: form.cron,
          actionParams: {
            endpoint: form.endpoint,
            description: form.description,
          },
        });
      }
      $q.notify({ type: 'positive', message: $t('common.operationSuccess') });
      showDialog.value = false;
      await loadTasks();
    } catch (error) {
      logger.error('【保存任务失败】', error);
      $q.notify({ type: 'negative', message: $t('common.operationFailed') });
    }
  }

  // ============ 任务操作 ============
  async function handleToggleStatus(task: ScheduledTask) {
    try {
      if (task.status === 'active') {
        await pauseScheduledTask(task.id);
      } else {
        await resumeScheduledTask(task.id);
      }
      $q.notify({ type: 'positive', message: $t('common.operationSuccess') });
      await loadTasks();
    } catch (error) {
      logger.error('【切换任务状态失败】', error);
      $q.notify({ type: 'negative', message: $t('common.operationFailed') });
    }
  }

  async function handleExecute(task: ScheduledTask) {
    try {
      await triggerScheduledTask(task.id);
      $q.notify({ type: 'positive', message: $t('common.operationSuccess') });
      await loadTasks();
    } catch (error) {
      logger.error('【执行任务失败】', error);
      $q.notify({ type: 'negative', message: $t('common.operationFailed') });
    }
  }

  function handleDelete(task: ScheduledTask) {
    $q.dialog({
      title: $t('common.confirmDelete'),
      message: $t('taskSchedule.confirmDeleteTask', { name: task.name }),
      cancel: { label: $t('common.cancel'), flat: true },
      ok: { label: $t('common.confirm'), color: 'negative' },
      persistent: true,
    }).onOk(() => {
      void (async () => {
        try {
          await deleteScheduledTask(task.id);
          $q.notify({ type: 'positive', message: $t('common.operationSuccess') });
          await loadTasks();
        } catch (error) {
          logger.error('【删除任务失败】', error);
          $q.notify({ type: 'negative', message: $t('common.operationFailed') });
        }
      })();
    });
  }

  // ============ 生命周期 ============
  onMounted(() => {
    void loadTasks();
  });

  return {
    // 状态
    pagination,
    showDialog,
    showLogDialog,
    isEdit,
    executionLogs,
    filters,
    form,
    store,

    // 选项
    statusOptions,
    typeOptions,

    // 计算属性
    columns,

    // 工具函数
    getStatusColor,
    getStatusLabel,
    getDurationClass,
    getCronDescription,
    formatDateTime,

    // 数据加载
    loadTasks,
    onTableRequest,
    handleSearch,

    // 弹窗操作
    openCreateDialog,
    openEditDialog,
    handleSave,

    // 任务操作
    handleToggleStatus,
    handleExecute,
    handleDelete,
  };
}
