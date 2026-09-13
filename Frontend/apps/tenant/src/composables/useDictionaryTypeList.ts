/**
 * @file useDictionaryTypeList.ts
 * @description DictionaryTypeListPage 业务逻辑 composable
 */

import { ref, reactive, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useDictionaryStore } from '@/stores/system';
import type { DictionaryType, DictionaryItem } from '@/api/dictionary';
import { logger } from '@/utils/logger';
import type { AdvancedFilters } from '@/types/advanced-search';

export function useDictionaryTypeList() {
  const $q = useQuasar();
  const { t: $t } = useI18n();
  const dictionaryStore = useDictionaryStore();

  // ============ 状态 ============

  const selectedTypes = ref<DictionaryType[]>([]);

  const filters = ref<AdvancedFilters>({
    keyword: '',
    start_date: '',
    end_date: '',
    status: null,
  });

  const showTypeDialog = ref(false);
  const isEditType = ref(false);
  const currentType = ref<DictionaryType | null>(null);
  const typeForm = reactive({
    id: 0,
    code: '',
    name: '',
    description: '',
    sort: 0,
    status: 1,
  });

  const showItemsDialog = ref(false);
  const showItemDialog = ref(false);
  const isEditItem = ref(false);
  const itemForm = reactive({
    id: 0,
    type_id: 0,
    label: '',
    value: '',
    sort: 0,
    status: 1,
    is_default: false,
    remark: '',
  });

  const showBatchAddDialog = ref(false);
  const batchAddText = ref('');
  const pendingDeleteType = ref<DictionaryType | null>(null);
  const showDeleteConfirmDialog = ref(false);

  const pagination = ref({
    page: 1,
    rowsPerPage: 20,
    rowsNumber: 0,
  });

  const itemFilters = reactive({
    keyword: '',
    status: null as number | null,
  });

  // ============ 计算属性 ============

  // Status options — static data, not dependent on reactive state
  const statusOptionsForSearch = [
    { label: $t('dictionary.statusOptions.all'), value: null },
    { label: $t('dictionary.enabled'), value: 1 },
    { label: $t('dictionary.disabled'), value: 0 },
  ] as const;

  const statusOptionsForDialog = [
    { label: $t('dictionary.enabled'), value: 1 },
    { label: $t('dictionary.disabled'), value: 0 },
  ] as const;

  const typeColumns = computed(() => [
    { name: 'code', label: $t('dictionary.typeCode'), field: 'code', align: 'left' as const, sortable: true },
    { name: 'name', label: $t('dictionary.typeName'), field: 'name', align: 'left' as const },
    { name: 'description', label: $t('dictionary.description'), field: 'description', align: 'left' as const },
    { name: 'sort', label: $t('dictionary.sort'), field: 'sort', align: 'center' as const },
    { name: 'status', label: $t('dictionary.status'), field: 'status', align: 'center' as const },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  const itemColumns = computed(() => [
    { name: 'sort', label: $t('dictionary.sort'), field: 'sort', align: 'center' as const },
    { name: 'label', label: $t('dictionary.itemLabel'), field: 'label', align: 'left' as const },
    { name: 'value', label: $t('dictionary.itemValue'), field: 'value', align: 'left' as const },
    { name: 'default', label: $t('dictionary.is_default'), field: 'default', align: 'center' as const },
    { name: 'status', label: $t('dictionary.status'), field: 'status', align: 'center' as const },
    { name: 'remark', label: $t('dictionary.remark'), field: 'remark', align: 'left' as const },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' as const },
  ]);

  const filteredItems = computed(() => {
    let items = dictionaryStore.dictionaryItems;
    if (itemFilters.keyword) {
      const kw = itemFilters.keyword.toLowerCase();
      items = items.filter(
        (i) => i.label.toLowerCase().includes(kw) || i.value.toLowerCase().includes(kw),
      );
    }
    if (itemFilters.status !== null) {
      items = items.filter((i) => i.status === itemFilters.status);
    }
    return items;
  });

  // ============ 列表加载 ============

  function handleSearch() {
    pagination.value.page = 1;
    void loadTypes();
  }

  function handleReset() {
    filters.value = {
      keyword: '',
      start_date: '',
      end_date: '',
      status: null,
    };
    pagination.value.page = 1;
    void loadTypes();
  }

  async function loadTypes() {
    await dictionaryStore.fetchDictionaryTypes();
    pagination.value.rowsNumber = dictionaryStore.pagination.total;
  }

  function onTableRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
    pagination.value.page = props.pagination.page;
    pagination.value.rowsPerPage = props.pagination.rowsPerPage;
    void loadTypes();
  }

  // ============ 字典类型 CRUD ============

  function openTypeDialog(type?: DictionaryType) {
    if (type) {
      isEditType.value = true;
      Object.assign(typeForm, type);
    } else {
      isEditType.value = false;
      Object.assign(typeForm, {
        id: 0, code: '', name: '', description: '', sort: 0, status: 1,
      });
    }
    showTypeDialog.value = true;
  }

  async function saveType() {
    try {
      if (isEditType.value) {
        await dictionaryStore.editDictionaryType(typeForm.id, {
          name: typeForm.name,
          description: typeForm.description,
          sort: typeForm.sort,
          status: typeForm.status,
        });
      } else {
        await dictionaryStore.addDictionaryType({
          code: typeForm.code,
          name: typeForm.name,
          description: typeForm.description,
          sort: typeForm.sort,
        });
      }
      $q.notify({ type: 'positive', message: $t('common.success') });
      showTypeDialog.value = false;
      void loadTypes();
    } catch (error) {
      logger.error('【保存字典类型失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  function handleDeleteType(type: DictionaryType) {
    pendingDeleteType.value = type;
  }

  async function doDeleteType() {
    if (!pendingDeleteType.value) return;
    try {
      await dictionaryStore.removeDictionaryType(pendingDeleteType.value.id);
      $q.notify({ type: 'positive', message: $t('common.success') });
      pendingDeleteType.value = null;
      void loadTypes();
    } catch (error) {
      logger.error('【删除字典类型失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  // ============ 字典项 CRUD ============

  async function manageItems(type: DictionaryType) {
    currentType.value = type;
    showItemsDialog.value = true;
    await dictionaryStore.fetchDictionaryItems(type.id);
  }

  function openItemDialog(item?: DictionaryItem) {
    if (item) {
      isEditItem.value = true;
      itemForm.id = item.id;
      itemForm.type_id = item.type_id;
      itemForm.label = item.label;
      itemForm.value = item.value;
      itemForm.sort = item.sort;
      itemForm.status = item.status;
      itemForm.is_default = item.is_default ?? false;
      itemForm.remark = item.remark ?? '';
    } else {
      isEditItem.value = false;
      Object.assign(itemForm, {
        id: 0, type_id: currentType.value?.id || 0,
        label: '', value: '', sort: 0, status: 1, is_default: false, remark: '',
      });
    }
    showItemDialog.value = true;
  }

  async function saveItem() {
    try {
      if (isEditItem.value) {
        await dictionaryStore.editDictionaryItem(itemForm.id, {
          label: itemForm.label,
          value: itemForm.value,
          sort: itemForm.sort,
          status: itemForm.status,
          is_default: itemForm.is_default,
          remark: itemForm.remark,
        });
      } else {
        await dictionaryStore.addDictionaryItem({
          type_id: itemForm.type_id,
          label: itemForm.label,
          value: itemForm.value,
          sort: itemForm.sort,
          status: itemForm.status,
        });
      }
      $q.notify({ type: 'positive', message: $t('common.success') });
      showItemDialog.value = false;
      if (currentType.value) {
        await dictionaryStore.fetchDictionaryItems(currentType.value.id);
      }
    } catch (error) {
      logger.error('【保存字典项失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  function handleDeleteItem(item: DictionaryItem) {
    void doDeleteItem(item);
  }

  async function doDeleteItem(item: DictionaryItem) {
    try {
      await dictionaryStore.removeDictionaryItem(item.id);
      $q.notify({ type: 'positive', message: $t('common.success') });
      if (currentType.value) {
        await dictionaryStore.fetchDictionaryItems(currentType.value.id);
      }
    } catch (error) {
      logger.error('【删除字典项失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  // ============ 批量添加 ============

  function openBatchAddDialog() {
    batchAddText.value = '';
    showBatchAddDialog.value = true;
  }

  async function handleBatchAdd() {
    if (!currentType.value || !batchAddText.value.trim()) {
      $q.notify({ type: 'warning', message: $t('dictionary.batchAddRequired') });
      return;
    }

    const lines = batchAddText.value.trim().split('\n').filter((l) => l.trim());
    let successCount = 0;
    for (const line of lines) {
      const parts = line.split(/[,=:，＝：]/).map((s) => s.trim());
      if (parts.length < 2) continue;
      const labelStr: string = String(parts[0] ?? '');
      const valueStr: string = String(parts[1] ?? '');
      const tid = currentType.value?.id;
      if (tid === undefined) continue;
      try {
        await dictionaryStore.addDictionaryItem({
          type_id: tid,
          label: labelStr,
          value: valueStr,
          sort: 0,
          status: 1,
        });
        successCount++;
      } catch (error) {
        logger.warn('【批量添加字典项失败，跳过错误项】', error);
      }
    }
    $q.notify({ type: 'positive', message: $t('dictionary.batchAddSuccess', { count: successCount }) });
    showBatchAddDialog.value = false;
    if (currentType.value) {
      await dictionaryStore.fetchDictionaryItems(currentType.value.id);
    }
  }

  // ============ 批量操作 ============

  async function handleBatchEnable(items: unknown[]): Promise<void> {
    const ids = (items as DictionaryType[]).map((t) => t.id);
    try {
      await Promise.all(ids.map((id) => dictionaryStore.editDictionaryType(id, { status: 1 })));
      $q.notify({ type: 'positive', message: $t('batchActions.enableSuccess', { count: ids.length }) });
      selectedTypes.value = [];
      await loadTypes();
    } catch (error) {
      console.error('批量启用失败', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
      throw error;
    }
  }

  async function handleBatchDisable(items: unknown[]): Promise<void> {
    const ids = (items as DictionaryType[]).map((t) => t.id);
    try {
      await Promise.all(ids.map((id) => dictionaryStore.editDictionaryType(id, { status: 0 })));
      $q.notify({ type: 'positive', message: $t('batchActions.disableSuccess', { count: ids.length }) });
      selectedTypes.value = [];
      await loadTypes();
    } catch (error) {
      console.error('批量禁用失败', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
      throw error;
    }
  }

  async function handleBatchDelete(items: unknown[]): Promise<void> {
    const ids = (items as DictionaryType[]).map((t) => t.id);
    try {
      await Promise.all(ids.map((id) => dictionaryStore.removeDictionaryType(id)));
      $q.notify({ type: 'positive', message: $t('batchActions.deleteSuccess', { count: ids.length }) });
      selectedTypes.value = [];
      await loadTypes();
    } catch (error) {
      console.error('批量删除失败', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
      throw error;
    }
  }

  // ============ 生命周期 ============

  onMounted(() => {
    void loadTypes();
  });

  // ============ 返回 ============

  return {
    // Store
    dictionaryStore,
    // 状态
    selectedTypes, filters, pagination, itemFilters,
    showTypeDialog, isEditType, currentType, typeForm,
    showItemsDialog, showItemDialog, isEditItem, itemForm,
    showBatchAddDialog, batchAddText,
    pendingDeleteType, showDeleteConfirmDialog,
    // 计算属性
    statusOptionsForSearch, statusOptionsForDialog,
    typeColumns, itemColumns, filteredItems,
    // 列表
    loadTypes, onTableRequest, handleSearch, handleReset,
    // 字典类型 CRUD
    openTypeDialog, saveType, handleDeleteType, doDeleteType,
    // 字典项 CRUD
    manageItems, openItemDialog, saveItem, handleDeleteItem,
    // 批量添加
    openBatchAddDialog, handleBatchAdd,
    // 批量操作
    handleBatchEnable, handleBatchDisable, handleBatchDelete,
  };
}
