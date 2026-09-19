/**
 * @file useApiKey.ts
 * @description API 密钥管理页面业务逻辑 composable
 */

import { ref, computed, reactive, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { unwrapData } from '@/utils/unwrap';
import {
  listApiKeys,
  createApiKey,
  updateApiKey,
  deleteApiKey,
  enableApiKey,
  disableApiKey,
  type ApiKey,
  type CreateApiKeyRequest,
} from '@/api/api-key';

export function useApiKey() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  // 状态
  const loading = ref(false);
  const apiKeys = ref<ApiKey[]>([]);
  const pagination = ref({ page: 1, rowsPerPage: 10, rowsNumber: 0 });
  const showDialog = ref(false);
  const showKeyDialog = ref(false);
  const isEdit = ref(false);
  const newCreatedKey = ref('');

  // 筛选
  const filters = reactive({
    keyword: '',
    status: null as string | null,
  });

  // 表单
  const form = reactive({
    id: 0,
    name: '',
    description: '',
    permissions: [] as string[],
    rateLimit: 60,
    status: 'active',
    expires_at: '',
    ipWhitelist: '',
  });

  // 选项
  const statusOptions = [
    { label: '活跃', value: 'active' },
    { label: '已禁用', value: 'disabled' },
  ];

  const permissionOptions = [
    { label: '只读权限', value: 'read', description: '仅允许读取数据' },
    { label: '读写权限', value: 'write', description: '允许读取和写入数据' },
    { label: '管理员权限', value: 'admin', description: '完全控制权限' },
  ];

  // 表格列
  const columns = computed(() => [
    { name: 'name', label: $t('apiKey.name'), field: 'name', align: 'left' as const },
    { name: 'key', label: $t('apiKey.key'), field: 'key', align: 'left' as const },
    { name: 'permissions', label: $t('apiKey.permissions'), field: 'permissions', align: 'left' as const },
    { name: 'rateLimit', label: $t('apiKey.rateLimit'), field: 'rateLimit', align: 'center' as const },
    { name: 'status', label: $t('apiKey.status'), field: 'status', align: 'center' as const },
    { name: 'usage', label: $t('apiKey.usage'), field: 'usageCount', align: 'center' as const },
    { name: 'lastUsedAt', label: $t('apiKey.lastUsedAt'), field: 'lastUsedAt', align: 'left' as const },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  // 方法
  function formatNumber(num: number): string {
    if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
    if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
    return num.toString();
  }

  async function loadApiKeys() {
    loading.value = true;
    try {
      const listApiKeysParams: Record<string, unknown> = {
        page: pagination.value.page,
        page_size: pagination.value.rowsPerPage,
      };
      if (filters.keyword) listApiKeysParams.keyword = filters.keyword;
      if (filters.status) listApiKeysParams.status = filters.status;
      const response = await listApiKeys(listApiKeysParams);

      if (response.data) {
        apiKeys.value = response.data.records.map((item: ApiKey) => ({
          ...item,
          key: item.key_prefix,
          rateLimit: item.rate_limit || 60,
          permissions: [item.permission_level],
          usageCount: 0,
          usagePercent: 0,
        }));
        pagination.value.rowsNumber = response.data.total;
      }
    } catch (error) {
      if (import.meta.env.DEV) console.error('【加载 API 密钥失败】', error);
      $q.notify({ type: 'negative', message: '加载 API 密钥失败' });
    } finally {
      loading.value = false;
    }
  }

  function onTableRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
    void loadApiKeys();
  }

  function handleSearch() {
    pagination.value.page = 1;
    void loadApiKeys();
  }

  function openCreateDialog() {
    isEdit.value = false;
    Object.assign(form, {
      id: 0,
      name: '',
      description: '',
      permissions: [],
      rateLimit: 60,
      status: 'active',
      expires_at: '',
      ipWhitelist: '',
    });
    showDialog.value = true;
  }

  function openEditDialog(apiKey: ApiKey) {
    isEdit.value = true;
    Object.assign(form, {
      id: apiKey.id,
      name: apiKey.name,
      description: apiKey.description || '',
      permissions: [apiKey.permission_level],
      rateLimit: apiKey.rate_limit || 60,
      status: apiKey.status === 'active' ? 'active' : 'disabled',
      expires_at: apiKey.expires_at ? apiKey.expires_at.slice(0, 16) : '',
      ipWhitelist: apiKey.allowed_ips?.join(', ') || '',
    });
    showDialog.value = true;
  }

  async function handleSave() {
    try {
      if (!isEdit.value) {
        const request: Record<string, unknown> = {
          name: form.name,
          description: form.description,
          permission_level: (form.permissions[0]) || 'read',
          rate_limit: form.rateLimit,
        };
        if (form.ipWhitelist) request.allowed_ips = form.ipWhitelist.split(',').map((s) => s.trim());
        if (form.expires_at) request.expires_at = form.expires_at;

        const response = await createApiKey(request as CreateApiKeyRequest);
        const response_data = unwrapData<{ data: { key_id: string; key_secret: string } }>(response)?.data;
        newCreatedKey.value = `${response_data.key_id}:${response_data.key_secret}`;
        showDialog.value = false;
        showKeyDialog.value = true;
      } else {
        const request: Record<string, unknown> = {
          name: form.name,
          description: form.description,
          permission_level: (form.permissions[0]) || 'read',
          rate_limit: form.rateLimit,
          status: form.status,
        };
        if (form.ipWhitelist) request.allowed_ips = form.ipWhitelist.split(',').map((s) => s.trim());
        if (form.expires_at) request.expires_at = form.expires_at;

        await updateApiKey(form.id.toString(), request);
        showDialog.value = false;
      }
      await loadApiKeys();
      $q.notify({ type: 'positive', message: $t('common.success') });
    } catch (error) {
      if (import.meta.env.DEV) console.error('【保存 API 密钥失败】', error);
      $q.notify({ type: 'negative', message: '保存 API 密钥失败' });
    }
  }

  async function handleToggleStatus(apiKey: ApiKey) {
    try {
      if (apiKey.status === 'active') {
        await disableApiKey(apiKey.id);
      } else {
        await enableApiKey(apiKey.id);
      }
      await loadApiKeys();
      $q.notify({ type: 'positive', message: $t('common.success') });
    } catch (error) {
      if (import.meta.env.DEV) console.error('【切换 API 密钥状态失败】', error);
      $q.notify({ type: 'negative', message: '切换 API 密钥状态失败' });
    }
  }

  async function handleDelete(apiKey: ApiKey) {
    const confirmed = await new Promise<boolean>((resolve) => {
      $q.dialog({
        title: $t('common.confirm'),
        message: $t('apiKey.deleteConfirm', { name: apiKey.name }),
        cancel: true,
        persistent: true,
      }).onOk(() => resolve(true)).onCancel(() => resolve(false));
    });

    if (!confirmed) return;

    try {
      await deleteApiKey(apiKey.id);
      await loadApiKeys();
      $q.notify({ type: 'positive', message: $t('common.success') });
    } catch (error) {
      if (import.meta.env.DEV) console.error('【删除 API 密钥失败】', error);
      $q.notify({ type: 'negative', message: '删除 API 密钥失败' });
    }
  }

  async function handleCopyKey(key: string) {
    await navigator.clipboard.writeText(key);
    $q.notify({ type: 'positive', message: $t('apiKey.copied') });
  }

  onMounted(() => {
    void loadApiKeys();
  });

  return {
    loading,
    apiKeys,
    pagination,
    showDialog,
    showKeyDialog,
    isEdit,
    newCreatedKey,
    filters,
    form,
    statusOptions,
    permissionOptions,
    columns,
    formatNumber,
    loadApiKeys,
    onTableRequest,
    handleSearch,
    openCreateDialog,
    openEditDialog,
    handleSave,
    handleToggleStatus,
    handleDelete,
    handleCopyKey,
  };
}
