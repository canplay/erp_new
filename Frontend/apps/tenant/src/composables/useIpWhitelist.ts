/**
 * @file useIpWhitelist.ts
 * @description IP白名单页面业务逻辑 composable
 */

import { ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import {
  getIpWhitelist,
  createIpWhitelist,
  updateIpWhitelist,
  deleteIpWhitelist,
  enableIpWhitelist,
  disableIpWhitelist,
  type IpWhitelistRule,
  type IpWhitelistCreateParams,
} from '@/api/ip-whitelist';

export function useIpWhitelist() {
  const $q = useQuasar();
  const { t } = useI18n();

  // ─── 表格状态 ───
  const loading = ref(false);
  const submitting = ref(false);
  const ruleList = ref<IpWhitelistRule[]>([]);
  const searchKeyword = ref('');
  const pagination = ref({ page: 1, rowsPerPage: 15, total: 0 });

  // ─── 对话框状态 ───
  const dialogVisible = ref(false);
  const isEdit = ref(false);
  const currentEditId = ref<number | null>(null);
  const formData = ref<IpWhitelistCreateParams>({
    name: '',
    ipType: 'ip',
    ipStart: '',
    ipEnd: '',
    targetType: 'all',
    effectStartTime: '',
    effectEndTime: '',
    priority: 100,
    status: 1,
    description: '',
  });

  // ─── 选项数据 ───
  const ipTypeOptions = computed(() => [
    { label: t('ipWhitelist.typeIp'), value: 'ip' },
    { label: t('ipWhitelist.typeCidr'), value: 'cidr' },
    { label: t('ipWhitelist.typeRange'), value: 'range' },
  ]);

  const targetTypeOptions = computed(() => [
    { label: t('ipWhitelist.targetAll'), value: 'all' },
    { label: t('ipWhitelist.targetRole'), value: 'role' },
    { label: t('ipWhitelist.targetUser'), value: 'user' },
  ]);

  // ─── 计算属性 ───
  const ipStartLabel = computed(() => {
    const labels: Record<string, string> = {
      ip: t('ipWhitelist.ipAddress'),
      cidr: t('ipWhitelist.cidr'),
      range: t('ipWhitelist.ipStart'),
    };
    return labels[formData.value.ipType] || t('ipWhitelist.ipAddress');
  });

  const ipStartPlaceholder = computed(() => {
    const placeholders: Record<string, string> = {
      ip: '例：192.0.2.1',
      cidr: '例：192.0.2.0/24',
      range: '例：192.0.2.1',
    };
    return placeholders[formData.value.ipType] || '';
  });

  const columns = computed(() => [
    { name: 'id', label: 'ID', field: 'id', align: 'center' as const, style: 'width: 60px' },
    { name: 'name', label: t('ipWhitelist.ruleName'), field: 'name', align: 'left' as const },
    { name: 'ipType', label: t('ipWhitelist.ipType'), field: 'ipType', align: 'center' as const },
    { name: 'ipRange', label: t('ipWhitelist.ipRange'), field: 'ipRange', align: 'left' as const },
    { name: 'targetType', label: t('ipWhitelist.targetType'), field: 'targetType', align: 'center' as const },
    { name: 'effectTime', label: t('ipWhitelist.effectTime'), field: 'effectTime', align: 'left' as const },
    { name: 'priority', label: t('ipWhitelist.priority'), field: 'priority', align: 'center' as const },
    { name: 'status', label: t('ipWhitelist.status'), field: 'status', align: 'center' as const },
    { name: 'actions', label: t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  // ─── 方法 ───

  function getIpTypeColor(ipType: string) {
    const colors: Record<string, string> = {
      ip: 'primary',
      cidr: 'info',
      range: 'warning',
    };
    return colors[ipType] || 'grey';
  }

  function getIpTypeLabel(ipType: string) {
    const labels: Record<string, string> = {
      ip: t('ipWhitelist.typeIp'),
      cidr: t('ipWhitelist.typeCidr'),
      range: t('ipWhitelist.typeRange'),
    };
    return labels[ipType] || ipType;
  }

  function getTargetTypeLabel(targetType: string) {
    const labels: Record<string, string> = {
      all: t('ipWhitelist.targetAll'),
      role: t('ipWhitelist.targetRole'),
      user: t('ipWhitelist.targetUser'),
    };
    return labels[targetType] || targetType;
  }

  function formatIpRange(row: IpWhitelistRule) {
    if (row.ipType === 'cidr') {
      return row.ipStart;
    } else if (row.ipType === 'range') {
      return `${row.ipStart} ~ ${row.ipEnd || '-'}`;
    }
    return row.ipStart;
  }

  function onIpTypeChange() {
    formData.value.ipStart = '';
    formData.value.ipEnd = '';
  }

  async function loadRules() {
    loading.value = true;
    try {
      const response = await getIpWhitelist({
        keyword: searchKeyword.value,
        page: pagination.value.page,
        page_size: pagination.value.rowsPerPage,
      }) as { list?: unknown[]; data?: { list?: unknown[]; total?: number }; total?: number };
      const data = response.data ?? response;
      ruleList.value = (data.list ?? []) as IpWhitelistRule[];
      pagination.value.total = data.total ?? 0;
    } catch (error) {
      logger.error('【加载IP白名单失败】', error);
      $q.notify({
        type: 'negative',
        message: t('ipWhitelist.loadFailed'),
      });
    } finally {
      loading.value = false;
    }
  }

  function onRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
    void loadRules();
  }

  function openCreateDialog() {
    isEdit.value = false;
    currentEditId.value = null;
    formData.value = {
      name: '',
      ipType: 'ip',
      ipStart: '',
      ipEnd: '',
      targetType: 'all',
      effectStartTime: '',
      effectEndTime: '',
      priority: 100,
      status: 1,
      description: '',
    };
    dialogVisible.value = true;
  }

  function openEditDialog(row: IpWhitelistRule) {
    isEdit.value = true;
    currentEditId.value = row.id;
    formData.value = {
      name: row.name,
      ipType: row.ipType,
      ipStart: row.ipStart,
      ipEnd: row.ipEnd || '',
      targetType: row.targetType,
      effectStartTime: row.effectStartTime || '',
      effectEndTime: row.effectEndTime || '',
      priority: row.priority,
      status: row.status,
      description: row.description || '',
    };
    dialogVisible.value = true;
  }

  async function handleSubmit(): Promise<void> {
    if (!formData.value.name || !formData.value.ipStart) {
      $q.notify({
        type: 'warning',
        message: t('common.fillRequiredFields'),
      });
      return;
    }

    submitting.value = true;
    try {
      if (isEdit.value && currentEditId.value !== null) {
        await updateIpWhitelist(currentEditId.value, {
          ...formData.value,
          id: currentEditId.value,
        });
      } else {
        await createIpWhitelist(formData.value);
      }
      $q.notify({
        type: 'positive',
        message: t('common.success'),
      });
      dialogVisible.value = false;
      await loadRules();
    } catch (error) {
      logger.error('【保存IP白名单失败】', error);
      $q.notify({
        type: 'negative',
        message: t('common.error'),
      });
    } finally {
      submitting.value = false;
    }
  }

  async function deleteRule(id: number) {
    try {
      await deleteIpWhitelist(id);
      $q.notify({
        type: 'positive',
        message: t('common.success'),
      });
      await loadRules();
    } catch (error) {
      logger.error('【删除IP白名单失败】', error);
      $q.notify({
        type: 'negative',
        message: t('common.error'),
      });
    }
  }

  async function toggleStatus(row: IpWhitelistRule) {
    try {
      if (row.status === 1) {
        await disableIpWhitelist(row.id);
      } else {
        await enableIpWhitelist(row.id);
      }
      $q.notify({
        type: 'positive',
        message: t('common.success'),
      });
      await loadRules();
    } catch (error) {
      logger.error('【切换状态失败】', error);
      $q.notify({
        type: 'negative',
        message: t('common.error'),
      });
    }
  }

  return {
    // 状态
    loading,
    submitting,
    ruleList,
    searchKeyword,
    pagination,
    dialogVisible,
    isEdit,
    formData,
    // 计算属性
    ipTypeOptions,
    targetTypeOptions,
    ipStartLabel,
    ipStartPlaceholder,
    columns,
    // 方法
    getIpTypeColor,
    getIpTypeLabel,
    getTargetTypeLabel,
    formatIpRange,
    onIpTypeChange,
    loadRules,
    onRequest,
    openCreateDialog,
    openEditDialog,
    handleSubmit,
    deleteRule,
    toggleStatus,
  };
}
