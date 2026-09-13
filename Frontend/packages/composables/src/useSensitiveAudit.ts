/**
 * @file useSensitiveAudit.ts
 * @description 敏感操作审计页面业务逻辑 composable
 */

import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import {
  getSensitiveOperations,
  approveVerification,
  getSensitiveOperationStatistics,
  type SensitiveOperation,
} from '@/api/sensitive-audit';

export function useSensitiveAudit() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  // ── 状态 ──
  const loading = ref(false);
  const detailDialogVisible = ref(false);
  const currentRecord = ref<SensitiveOperation | null>(null);
  const recordList = ref<SensitiveOperation[]>([]);
  const pagination = ref({ page: 1, rowsPerPage: 15, total: 0 });

  // ── 筛选 ──
  const filters = ref<{
    operation_type: string | undefined;
    confirmType: string | undefined;
    confirmStatus: string | undefined;
  }>({
    operation_type: undefined,
    confirmType: undefined,
    confirmStatus: undefined,
  });

  // ── 统计 ──
  const statistics = ref({ pending: 0, success: 0, failed: 0, today: 0 });

  // ── 选项 ──
  const operation_type_options = [
    { label: '删除用户', value: 'user_delete' },
    { label: '角色权限变更', value: 'role_permission' },
    { label: '系统配置修改', value: 'system_config' },
    { label: '敏感数据导出', value: 'data_export' },
    { label: '批量操作', value: 'batch_operation' },
  ];

  const confirmTypeOptions = [
    { label: '密码验证', value: 'password' },
    { label: '短信验证', value: 'sms' },
    { label: '邮箱验证', value: 'email' },
    { label: '身份验证器', value: 'authenticator' },
    { label: '管理员审批', value: 'admin' },
  ];

  const confirmStatusOptions = [
    { label: '待验证', value: 'pending' },
    { label: '验证成功', value: 'success' },
    { label: '验证失败', value: 'failed' },
    { label: '已过期', value: 'expired' },
  ];

  // ── 表格列定义 ──
  const columns = computed(() => [
    { name: 'id', label: 'ID', field: 'id', align: 'center' as const, style: 'width: 60px' },
    { name: 'operation_type', label: $t('sensitiveAudit.operation_type'), field: 'operation_type', align: 'center' as const },
    { name: 'operationDesc', label: $t('sensitiveAudit.operationDesc'), field: 'operationDesc', align: 'left' as const },
    { name: 'userName', label: $t('sensitiveAudit.user'), field: 'userName', align: 'left' as const },
    { name: 'confirmType', label: $t('sensitiveAudit.confirmType'), field: 'confirmType', align: 'left' as const },
    { name: 'confirmStatus', label: $t('sensitiveAudit.status'), field: 'confirmStatus', align: 'center' as const },
    { name: 'operationStatus', label: $t('sensitiveAudit.operationStatus'), field: 'operationStatus', align: 'center' as const },
    { name: 'created_at', label: $t('sensitiveAudit.createTime'), field: 'created_at', align: 'left' as const },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  // ── 数据加载 ──
  async function loadRecords(): Promise<void> {
    loading.value = true;
    try {
      const params: Record<string, unknown> = {
        page: pagination.value.page,
        page_size: pagination.value.rowsPerPage,
      };
      if (filters.value.operation_type) params.operation_type = filters.value.operation_type;
      if (filters.value.confirmType) params.confirmType = filters.value.confirmType;
      if (filters.value.confirmStatus) params.confirmStatus = filters.value.confirmStatus;
      const response = await getSensitiveOperations(params);
      const respData = response as { data?: { list?: SensitiveOperation[]; total?: number } };
      recordList.value = respData.data?.list || [];
      pagination.value.total = respData.data?.total || 0;
    } catch (error) {
      logger.error('【加载敏感操作记录失败】', error);
      $q.notify({ type: 'negative', message: $t('sensitiveAudit.loadFailed') });
    } finally {
      loading.value = false;
    }
  }

  async function loadStatistics(): Promise<void> {
    try {
      const response = await getSensitiveOperationStatistics();
      const respData = response as { data?: { pending: number; success: number; failed: number; today: number } };
      statistics.value = respData.data || { pending: 0, success: 0, failed: 0, today: 0 };
    } catch (error) {
      logger.error('【加载统计数据失败】', error);
    }
  }

  // ── 分页 ──
  interface TableRequestProps {
    pagination: { page: number; rowsPerPage: number };
  }
  function onRequest(props: TableRequestProps): void {
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
    void loadRecords();
  }

  // ── 筛选重置 ──
  function resetFilters(): void {
    filters.value = { operation_type: undefined, confirmType: undefined, confirmStatus: undefined };
    void loadRecords();
  }

  // ── 辅助函数 ──
  function getConfirmTypeIcon(confirmType: string): string {
    const icons: Record<string, string> = {
      password: 'lock', sms: 'sms', email: 'email',
      authenticator: 'authenticator', admin: 'admin_panel_settings',
    };
    return icons[confirmType] || 'help';
  }

  function getConfirmTypeColor(confirmType: string): string {
    const colors: Record<string, string> = {
      password: 'blue', sms: 'green', email: 'orange',
      authenticator: 'purple', admin: 'red',
    };
    return colors[confirmType] || 'grey';
  }

  function getConfirmTypeLabel(confirmType: string): string {
    const labels: Record<string, string> = {
      password: $t('sensitiveAudit.typePassword'),
      sms: $t('sensitiveAudit.typeSms'),
      email: $t('sensitiveAudit.typeEmail'),
      authenticator: $t('sensitiveAudit.typeAuthenticator'),
      admin: $t('sensitiveAudit.typeAdmin'),
    };
    return labels[confirmType] || confirmType;
  }

  function getStatusColor(status: string): string {
    const colors: Record<string, string> = {
      pending: 'warning', success: 'positive', failed: 'negative', expired: 'grey',
    };
    return colors[status] || 'grey';
  }

  function getStatusLabel(status: string): string {
    const labels: Record<string, string> = {
      pending: $t('sensitiveAudit.statusPending'),
      success: $t('sensitiveAudit.statusSuccess'),
      failed: $t('sensitiveAudit.statusFailed'),
      expired: $t('sensitiveAudit.statusExpired'),
    };
    return labels[status] || status;
  }

  // ── 详情 ──
  function viewDetail(row: SensitiveOperation): void {
    currentRecord.value = row;
    detailDialogVisible.value = true;
  }

  // ── 审批 ──
  async function approveRecord(row: SensitiveOperation, approved: boolean): Promise<void> {
    try {
      await approveVerification(row.id, approved);
      $q.notify({
        type: 'positive',
        message: approved ? $t('sensitiveAudit.approveSuccess') : $t('sensitiveAudit.rejectSuccess'),
      });
      await loadRecords();
      await loadStatistics();
    } catch (error) {
      logger.error('【审批失败】', error);
      $q.notify({ type: 'negative', message: $t('sensitiveAudit.approveFailed') });
    }
  }

  // ── 初始化 ──
  async function initData(): Promise<void> {
    await Promise.all([loadRecords(), loadStatistics()]);
  }

  return {
    // 状态
    loading,
    detailDialogVisible,
    currentRecord,
    recordList,
    pagination,
    filters,
    statistics,
    // 选项
    operation_type_options,
    confirmTypeOptions,
    confirmStatusOptions,
    // 列
    columns,
    // 方法
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
    initData,
  };
}
