<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('user.title') }}</div>

    <!-- 批量操作栏 -->
    <BatchActions
      v-model:selected="selectedUsers"
      :show-enable="true"
      :show-disable="true"
      :show-delete="true"
      :show-export="true"
      @batch-enable="handleBatchEnable"
      @batch-disable="handleBatchDisable"
      @batch-delete="handleBatchDelete"
      @export="handleBatchExport"
    >
      <!-- 导入按钮插槽 -->
      <template #actions>
        <q-btn
          flat
          dense
          color="info"
          :label="$t('user.import')"
          icon="upload"
          @click="showImportDialog = true"
        />
        <q-btn
          flat
          dense
          color="secondary"
          :label="$t('user.downloadTemplate')"
          icon="download"
          @click="downloadTemplate"
        />
      </template>
    </BatchActions>

    <!-- 高级搜索（合并版） -->
    <AdvancedSearch
      v-model="filters"
      :show-status="true"
      :show-role="true"
      :show-date-range="true"
      :show-date-shortcuts="true"
      :keyword-placeholder="$t('user.searchPlaceholder')"
      :status-label="$t('user.status')"
      :role-label="$t('user.role')"
      :date-range-label="$t('common.dateRange')"
      :status-options="statusOptionsForSearch"
      :role-options="roleOptionsForSearch"
      :field-options="[]"
      keyword-class="col-12 col-sm-6"
      @search="handleSearch"
      @reset="handleReset"
    />

    <!-- 工具栏（简化版） -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center justify-end">
        <q-btn flat color="grey" icon="view_column" :label="$t('table.selectColumns')" @click="columnSelectorRef?.open()" />
        <q-btn color="positive" :label="$t('user.addUser')" icon="add" class="q-ml-sm" @click="openUserDialog()" data-testid="create-user-button" />
      </q-card-section>
    </q-card>

    <!-- 用户表格 -->
    <q-card bordered>
      <q-table
        v-model:selected="selectedUsers"
        :rows="users"
        :columns="displayedColumns"
        row-key="id"
        :loading="loading"
        :pagination="pagination"
        selection="multiple"
        @request="onTableRequest"
      >
        <!-- 头像列 -->
        <template v-slot:body-cell-avatar="props">
          <q-td :props="props">
            <q-avatar size="36px" color="primary" text-color="white">
              {{ props.row.username?.substring(0, 1).toUpperCase() }}
            </q-avatar>
          </q-td>
        </template>

        <!-- 状态列 -->
        <template v-slot:body-cell-status="props">
          <q-td :props="props">
            <q-chip
              :color="(_getStatusColor as any)(props.value)"
              text-color="white"
              dense
              :icon="(_getStatusIcon as any)(props.value)"
            >
              {{ (_getStatusLabel as any)(props.value) }}
            </q-chip>
          </q-td>
        </template>

        <!-- 角色列 -->
        <template v-slot:body-cell-role="props">
          <q-td :props="props">
            <q-chip
              :color="(_getRoleChipColor as any)(props.value)"
              text-color="white"
              dense
            >
              {{ (_getRoleLabel as any)(props.value) }}
            </q-chip>
          </q-td>
        </template>

        <!-- 注册时间列 -->
        <template v-slot:body-cell-created_at="props">
          <q-td :props="props">
            {{ props.value ? formatUserTime(props.value) : '-' }}
          </q-td>
        </template>

        <!-- 操作列 -->
        <template v-slot:body-cell-actions="props">
          <q-td :props="props">
            <q-btn flat dense color="primary" :label="$t('common.view')" @click="viewUser(props.row)" />
            <q-btn flat dense color="primary" :label="$t('common.edit')" @click="openUserDialog(props.row)" />
            <q-btn flat dense color="negative" :label="$t('common.delete')" @click="handleDelete(props.row)" />
          </q-td>
        </template>

        <!-- 加载状态 -->
        <template v-slot:loading>
          <q-spinner-dots />
        </template>

        <!-- 空状态 -->
        <template v-slot:no-data="{ message }">
          <div class="full-width row justify-center q-pa-xl text-grey-6">
            <q-icon name="inbox" size="64px" class="q-mb-md" />
            <div>{{ message || $t('table.noData') || $t('user.noData') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 用户编辑弹窗 -->
    <UserEditDialog
      v-model="showUserDialog"
      :is-edit="isEdit"
      :form="userForm"
      @save="saveUser"
    />

    <!-- 导入弹窗 -->
    <q-dialog v-model="showImportDialog" persistent maximized>
      <q-card>
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('user.importUsers') || $t('user.importStepFile') }}</div>
          <q-space />
          <q-btn flat round icon="close" @click="handleImportComplete" />
        </q-card-section>

        <q-separator />

        <q-card-section>
          <q-stepper v-model="importStep" vertical color="primary" header-class="hidden">
            <q-step :name="1" :title="$t('common.selectFile')" icon="upload" :done="importStep > 1" class="q-pb-none">
              <q-file
                v-model="importFile"
                accept=".csv"
                :label="$t('user.selectFile')"
                outlined
                @rejected="onFileRejected"
                @update:model-value="handleUploadFile"
              />
              <div class="text-caption text-grey q-mt-sm">
                {{ $t('user.fileFormatHint') }}
              </div>
            </q-step>

            <q-step :name="2" :title="$t('user.importStepUploading')" icon="cloud_upload" />
            <q-step :name="3" :title="$t('user.importStepPreview')" icon="preview" :done="importStep > 3">
              <div class="row q-gutter-sm q-mb-sm">
                <q-chip color="positive" text-color="white" size="sm">
                  {{ $t('user.validRows', { count: validCount }) || `有效: ${validCount}` }}
                </q-chip>
                <q-chip color="negative" text-color="white" size="sm">
                  {{ $t('user.errorRows', { count: errorCount }) || `错误: ${errorCount}` }}
                </q-chip>
              </div>
              <q-table
                :rows="importPreview"
                :columns="importPreviewColumns"
                row-key="rowIndex"
                flat
                dense
                :pagination="{ rowsPerPage: 5 }"
              >
                <template v-slot:body-cell-errors="props">
                  <q-td :props="props">
                    <q-badge
                      v-if="props.value?.length"
                      color="negative"
                      :label="props.value[0]"
                    />
                    <q-icon v-else name="check_circle" color="positive" size="20px" />
                  </q-td>
                </template>
              </q-table>

              <div class="row q-mt-md">
                <q-btn
                  color="positive"
                  :label="$t('user.startImport')"
                  @click="handleStartImport"
                  :disable="importing || selectedImportRows.length === 0"
                />
              </div>
            </q-step>

            <q-step :name="4" :title="$t('user.importStepComplete')" icon="check_circle" :done="importStep > 4">
              <div class="text-center q-pa-lg">
                <q-icon name="check_circle" size="64px" color="positive" />
                <div class="text-h6 q-mt-md">{{ $t('user.importCompleteTitle') }}</div>
                <div class="q-mt-sm" v-if="importResult">
                  <p>{{ $t('user.importResultMsg', { success: importResult.success, failed: importResult.failed }) }}</p>
                </div>
                <q-btn color="primary" :label="$t('common.close')" class="q-mt-md" @click="handleImportComplete" />
              </div>
            </q-step>
          </q-stepper>
        </q-card-section>
      </q-card>
    </q-dialog>

    <!-- 批量操作确认对话框 -->
    <ConfirmDialog
      ref="deleteDialogRef"
      v-model="showDeleteConfirmDialog"
      :title="$t('common.confirm')"
      :message="$t('user.confirmDelete', { username: pendingDeleteUser?.username || '' })"
      icon="warning"
      confirm-color="negative"
      @confirm="doDeleteUser"
    />
  </q-page>
</template>

<script setup lang="ts">
import type TableColumnSelector from '@erp-new-frontend-monorepo/components/src/TableColumnSelector.vue';
import ConfirmDialog from '@erp-new-frontend-monorepo/components/src/ConfirmDialog.vue';
import BatchActions from '@erp-new-frontend-monorepo/components/src/BatchActions.vue';
import AdvancedSearch from '@erp-new-frontend-monorepo/components/src/AdvancedSearch/Main.vue';
import { useUserList } from '@erp-new-frontend-monorepo/composables/src/useUserList';;
import UserEditDialog from '@erp-new-frontend-monorepo/components/src/UserEditDialog.vue';
import { ref } from 'vue';
import { formatDate } from '@/utils/format';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
/** 后端返回 epoch 秒，格式化显示 */
function formatUserTime(value: number | string): string {
  const num = typeof value === 'string' ? Number(value) : value;
  if (!num) return '-';
  return formatDate(num < 1e12 ? num * 1000 : num);
}

const _getStatusLabel = () => '';
const _getStatusColor = () => '';
const _getStatusIcon = () => '';
const _getRoleLabel = () => '';
const _getRoleChipColor = () => 'grey';

const {
  loading, users, selectedUsers, filters, pagination,
  displayedColumns,
  statusOptionsForSearch, roleOptionsForSearch,
  importPreviewColumns, validCount, errorCount,
  showUserDialog, isEdit, userForm,
  showImportDialog, importStep, importFile, importPreview,
  selectedImportRows, importing, importResult,
  showDeleteConfirmDialog, pendingDeleteUser,
  onTableRequest, handleSearch, handleReset,
  openUserDialog, viewUser, saveUser, handleDelete, doDeleteUser,
  downloadTemplate, onFileRejected, handleUploadFile, handleStartImport, handleImportComplete,
  handleBatchEnable, handleBatchDisable, handleBatchDelete, handleBatchExport,
} = useUserList();

const columnSelectorRef = ref<InstanceType<typeof TableColumnSelector> | null>(null);
const deleteDialogRef = ref<InstanceType<typeof ConfirmDialog> | null>(null);
</script>

<style scoped>
/* ============ 暗色主题适配 ============ */

/* 暗色主题页面标题 */
.body--dark .text-h5 {
  color: #ffffff;
}

/* 暗色主题卡片 */
.body--dark .q-card {
  background: #1e1e1e;
  border-color: #2d2d2d;
}

/* 暗色主题输入框 */
.body--dark .q-field__control {
  background: #2d2d2d;
}

.body--dark .q-field__native {
  color: #ffffff;
}

.body--dark .q-field__label {
  color: #b0b0b0;
}

/* 暗色主题表格 */
.body--dark .q-table {
  background: #1e1e1e;
  color: #ffffff;
}

.body--dark .q-table__top,
.body--dark .q-table__bottom,
.body--dark .q-table__head {
  background: #252525;
}

.body--dark .q-table th {
  color: #ffffff;
}

.body--dark .q-table tbody td {
  color: #b0b0b0;
}

.body--dark .q-table tbody tr:hover {
  background: rgba(255, 255, 255, 0.03);
}

/* 暗色主题复选框 */
.body--dark .q-checkbox__inner {
  color: #b0b0b0;
}

/* 暗色主题选择器 */
.body--dark .q-select__dropdown-icon {
  color: #b0b0b0;
}

/* 暗色主题步骤条 */
.body--dark .q-stepper {
  background: transparent;
}
</style>
