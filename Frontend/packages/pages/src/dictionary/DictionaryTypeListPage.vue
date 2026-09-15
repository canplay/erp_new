<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('dictionary.typeTitle') }}</div>

    <!-- 批量操作栏 -->
    <BatchActions
      v-model:selected="selectedTypes"
      :show-enable="true"
      :show-disable="true"
      :show-delete="true"
      :show-export="true"
      @batch-enable="handleBatchEnable"
      @batch-disable="handleBatchDisable"
      @batch-delete="handleBatchDelete"
      @export="handleBatchExport"
    />

    <!-- 高级搜索（合并版） -->
    <AdvancedSearch
      v-model="filters"
      :show-status="true"
      :keyword-placeholder="$t('dictionary.searchPlaceholder')"
      :status-label="$t('dictionary.status')"
      :status-options="statusOptionsForSearch"
      :field-options="[]"
      keyword-class="col-12 col-sm-6"
      @search="handleSearch"
      @reset="handleReset"
    />

    <!-- 工具栏（简化版） -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center justify-end">
        <q-btn
          color="positive"
          :label="$t('dictionary.addType')"
          icon="add"
          @click="openTypeDialog()"
        />
      </q-card-section>
    </q-card>

    <!-- 字典类型表格 -->
    <q-card bordered>
      <q-table
        v-model:selected="selectedTypes"
        :rows="dictionaryStore.dictionaryTypes"
        :columns="typeColumns"
        row-key="id"
        :loading="dictionaryStore.isLoading"
        selection="multiple"
        :pagination="(pagination as any)"
        @request="onTableRequest"
      >
        <template v-slot:body-cell-status="props">
          <q-td :props="props">
            <q-chip
              :color="props.value ? 'positive' : 'negative'"
              text-color="white"
              dense
            >
              {{ props.value ? $t('dictionary.enabled') : $t('dictionary.disabled') }}
            </q-chip>
          </q-td>
        </template>

        <template v-slot:body-cell-actions="props">
          <q-td :props="props">
            <q-btn flat dense color="primary" :label="$t('dictionary.manageItems')" @click="manageItems(props.row)" />
            <q-btn flat dense color="primary" icon="edit" @click="openTypeDialog(props.row)" />
            <q-btn flat dense color="negative" icon="delete" @click="handleDeleteType(props.row)" />
          </q-td>
        </template>

        <template v-slot:no-data="{ message }">
          <div class="full-width row justify-center q-pa-xl text-grey-6">
            <q-icon name="inbox" size="64px" class="q-mb-md" />
            <div>{{ message || $t('table.noData') || $t('dictionary.noData') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 字典类型编辑弹窗 -->
    <DictionaryTypeDialog
      v-model="showTypeDialog"
      :is-edit="isEditType"
      :form="typeForm"
      @update:code="typeForm.code = $event"
      @update:name="typeForm.name = $event"
      @update:description="typeForm.description = $event"
      @update:sort="typeForm.sort = $event"
      @update:status="typeForm.status = $event"
      @save="saveType"
    />

    <!-- 确认删除对话框 -->
    <ConfirmDialog
      ref="deleteTypeDialogRef"
      v-model="showDeleteConfirmDialog"
      :title="$t('common.confirm')"
      :message="$t('dictionary.deleteTypeConfirmMessage')"
      icon="warning"
      confirm-color="negative"
      @confirm="doDeleteType"
    />

    <!-- 字典项管理弹窗 -->
    <q-dialog v-model="showItemsDialog" persistent maximized>
      <q-card>
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('dictionary.manageItemsFor', { name: currentType?.name || '' }) }}</div>
          <q-space />
          <q-btn flat dense color="primary" :label="$t('dictionary.addItem')" icon="add" @click="openItemDialog()" />
          <q-btn flat dense color="primary" :label="$t('dictionary.batchAdd')" @click="openBatchAddDialog()" />
          <q-btn flat round icon="close" @click="showItemsDialog = false" />
        </q-card-section>
        <q-separator />
        <q-card-section>
          <!-- 筛选 -->
          <div class="row q-gutter-sm q-mb-md">
            <q-input v-model="itemFilters.keyword" dense outlined :placeholder="$t('dictionary.searchItemPlaceholder')" style="min-width: 200px" />
            <q-select v-model="itemFilters.status" :options="statusOptionsForSearch" dense outlined :label="$t('dictionary.status')" emit-value map-options style="min-width: 120px" clearable />
          </div>
          <q-table :rows="filteredItems" :columns="itemColumns" row-key="id" flat dense :pagination="{ rowsPerPage: 10 }">
            <template v-slot:body-cell-default="props">
              <q-td :props="props">
                <q-icon :name="props.value ? 'check_circle' : 'radio_button_unchecked'" :color="props.value ? 'positive' : 'grey'" />
              </q-td>
            </template>
            <template v-slot:body-cell-status="props">
              <q-td :props="props">
                <q-chip :color="props.value ? 'positive' : 'negative'" text-color="white" dense>{{ props.value ? $t('dictionary.enabled') : $t('dictionary.disabled') }}</q-chip>
              </q-td>
            </template>
            <template v-slot:body-cell-actions="props">
              <q-td :props="props">
                <q-btn flat dense icon="arrow_upward" size="sm" :disable="props.pageIndex === 0" @click="moveUp(props.pageIndex)" />
                <q-btn flat dense icon="arrow_downward" size="sm" :disable="props.pageIndex >= filteredItems.length - 1" @click="moveDown(props.pageIndex)" />
                <q-btn flat dense icon="edit" color="primary" size="sm" @click="openItemDialog(props.row)" />
                <q-btn flat dense icon="delete" color="negative" size="sm" @click="handleDeleteItem(props.row)" />
              </q-td>
            </template>
          </q-table>
        </q-card-section>
      </q-card>
    </q-dialog>

    <!-- 字典项编辑弹窗 -->
    <DictionaryItemDialog
      v-model="showItemDialog"
      :is-edit="isEditItem"
      :form="itemForm"
      @update:label="itemForm.label = $event"
      @update:value="itemForm.value = $event"
      @update:sort="itemForm.sort = $event"
      @update:status="itemForm.status = $event"
      @update:remark="itemForm.remark = $event"
      @update:is_default="itemForm.is_default = $event"
      @save="saveItem"
    />

    <!-- 批量添加弹窗 -->
    <q-dialog v-model="showBatchAddDialog" persistent>
      <q-card style="min-width: 500px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('dictionary.batchAdd') }}</div>
          <q-space />
          <q-btn flat round icon="close" @click="showBatchAddDialog = false" />
        </q-card-section>
        <q-separator />
        <q-card-section>
          <div class="text-caption q-mb-sm">{{ $t('dictionary.batchAddHint') }}</div>
          <q-input v-model="batchAddText" outlined type="textarea" :label="$t('dictionary.batchAddInput')" :input-style="{ minHeight: '150px', fontFamily: 'monospace' }" />
        </q-card-section>
        <q-separator />
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn color="primary" :label="$t('common.confirm')" @click="handleBatchAdd" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 隐藏的文件输入 -->
    <input ref="fileInputRef" type="file" accept=".csv,.json" style="display:none" @change="handleFileImport" />
  </q-page>
</template>

<script setup lang="ts">
import ConfirmDialog from '@erp-new-frontend-monorepo/components/src/ConfirmDialog.vue';
import { useExport } from '@erp-new-frontend-monorepo/composables/src/useExport';
import BatchActions from '@erp-new-frontend-monorepo/components/src/BatchActions.vue';
import AdvancedSearch from '@erp-new-frontend-monorepo/components/src/AdvancedSearch/Main.vue';
import DictionaryTypeDialog from '@erp-new-frontend-monorepo/components/src/DictionaryTypeDialog.vue';
import DictionaryItemDialog from '@erp-new-frontend-monorepo/components/src/DictionaryItemDialog.vue';
import { useDictionaryTypeList } from '@erp-new-frontend-monorepo/composables/src/useDictionaryTypeList';
import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import type { DictionaryType } from '@/api/dictionary';

const { t } = useI18n();
const $q = useQuasar();

const {
  dictionaryStore,
  selectedTypes, filters, pagination, itemFilters,
  showTypeDialog, isEditType, currentType, typeForm,
  showItemsDialog, showItemDialog, isEditItem, itemForm,
  showBatchAddDialog, batchAddText,
  showDeleteConfirmDialog,
  statusOptionsForSearch,
  typeColumns, itemColumns, filteredItems,
  onTableRequest, handleSearch, handleReset,
  openTypeDialog, saveType, handleDeleteType, doDeleteType,
  manageItems, openItemDialog, saveItem, handleDeleteItem,
  openBatchAddDialog, handleBatchAdd,
  handleBatchEnable, handleBatchDisable, handleBatchDelete,
} = useDictionaryTypeList();

const { exportToCSV } = useExport();

const deleteTypeDialogRef = ref<InstanceType<typeof ConfirmDialog> | null>(null);
const fileInputRef = ref<HTMLInputElement | null>(null);

// 页面专用：导出（不在 composable 中保留 useExport 依赖）
function handleBatchExport(items: unknown[]) {
  const typesToExport = items as DictionaryType[];
  void exportToCSV(typesToExport, [], 'dictionary-types.csv');
}

// 页面专用：排序（仅前端重新排列，无实际 API 调用）
function moveUp(index: number) {
  if (index <= 0 || !currentType.value) return;
  const items = dictionaryStore.dictionaryItems;
  const temp = items[index];
  if (items[index - 1] !== undefined && temp !== undefined) {
    items[index] = items[index - 1]!;
    items[index - 1] = temp;
  }
}

function moveDown(index: number) {
  if (!currentType.value) return;
  const items = dictionaryStore.dictionaryItems;
  if (index < items.length - 1) {
    const temp = items[index];
    if (items[index + 1] !== undefined && temp !== undefined) {
      items[index] = items[index + 1]!;
      items[index + 1] = temp;
    }
  }
}

// 页面专用：文件导入
async function handleFileImport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input?.files?.[0];
  if (!file) return;

  const type_id = currentType.value?.id;
  if (!type_id) return;

  try {
    const text = await file.text();
    const lines = text.split('\n').filter((l) => l.trim());
    if (lines.length < 2) return;

    const items = lines.slice(1)
      .map((line) => {
        const parts = line.split(',').map((p) => p.trim());
        const label = String(parts[0] ?? '');
        const value = String(parts[1] ?? '');
        return { label, value };
      })
      .filter((item) => item.label && item.value);

    if (items.length > 0) {
      await dictionaryStore.batchAddDictionaryItems(type_id, items);
      $q.notify({ type: 'positive', message: t('batchActions.importSuccess', { count: items.length }) });
    }
  } catch {
    $q.notify({ type: 'negative', message: t('common.error') });
  }
  if (input) input.value = '';
}
</script>

<style scoped>
.drag-handle {
  cursor: move;
}
</style>
