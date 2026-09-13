<template>
  <div class="import-template-manager">
    <!-- 页面标题 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">
        <q-icon name="description" class="q-mr-sm" />
        {{ $t('importMod.templateManagement') || '导入模板管理' }}
      </div>
      <q-space />
      <q-btn color="primary" icon="add" :label="$t('common.newTemplate')" @click="openCreateDialog" />
    </div>

    <!-- 搜索和筛选 -->
    <div class="row q-col-gutter-md q-mb-md">
      <div class="col-12 col-sm-4">
        <q-input
          v-model="searchQuery"
          outlined
          dense
          :placeholder="$t('importMod.searchTemplate') || '搜索模板...'"
        >
          <template v-slot:prepend>
            <q-icon name="search" />
          </template>
          <template v-slot:append v-if="searchQuery">
            <q-icon name="close" class="cursor-pointer" @click="searchQuery = ''" />
          </template>
        </q-input>
      </div>
      <div class="col-12 col-sm-3">
        <q-select
          v-model="entity_type_filter"
          :options="entity_type_options"
          outlined
          dense
          clearable
          :placeholder="$t('importMod.filterByEntity') || '按实体类型筛选'"
          emit-value
          map-options
        />
      </div>
    </div>

    <!-- 模板列表 -->
    <q-table
      :rows="filteredTemplates"
      :columns="columns"
      row-key="id"
      flat
      bordered
      :loading="loading"
      :pagination="{ rowsPerPage: 10 }"
    >
      <!-- 模板名称 -->
      <template v-slot:body-cell-name="props">
        <q-td :props="props">
          <div class="row items-center">
            <q-icon name="description" color="primary" size="20px" class="q-mr-sm" />
            <div>
              <div class="text-weight-medium">{{ props.row.name }}</div>
              <div class="text-caption text-grey">{{ props.row.description }}</div>
            </div>
            <q-badge
              v-if="props.row.is_default"
              color="warning"
              text-color="black"
              :label="$t('common.default')"
              class="q-ml-sm"
            />
          </div>
        </q-td>
      </template>

      <!-- 实体类型 -->
      <template v-slot:body-cell-entity_type="props">
        <q-td :props="props">
          <q-badge color="secondary" :label="getEntityLabel(props.row.entity_type)" />
        </q-td>
      </template>

      <!-- 列数 -->
      <template v-slot:body-cell-columns="props">
        <q-td :props="props">
          <div class="row items-center">
            <q-icon name="view_column" size="16px" class="q-mr-xs" />
            <span>{{ props.row.columns?.length || 0 }} 列</span>
          </div>
        </q-td>
      </template>

      <!-- 操作 -->
      <template v-slot:body-cell-actions="props">
        <q-td :props="props">
          <q-btn flat dense color="primary" icon="visibility" @click="previewTemplate(props.row)">
            <q-tooltip>{{ $t('common.preview') }}</q-tooltip>
          </q-btn>
          <q-btn flat dense color="info" icon="download" @click="downloadTemplate(props.row)">
            <q-tooltip>{{ $t('common.downloadTemplate') }}</q-tooltip>
          </q-btn>
          <q-btn flat dense color="primary" icon="edit" @click="openEditDialog(props.row)">
            <q-tooltip>{{ $t('common.edit') }}</q-tooltip>
          </q-btn>
          <q-btn flat dense color="negative" icon="delete" @click="deleteTemplate(props.row)">
            <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
          </q-btn>
        </q-td>
      </template>

      <!-- 加载状态 -->
      <template v-slot:loading>
        <q-inner-loading showing color="primary" />
      </template>

      <!-- 空状态 -->
      <template v-slot:no-data>
        <div class="text-center q-pa-xl">
          <q-icon name="folder_open" size="64px" color="grey-5" />
          <div class="text-h6 q-mt-md text-grey">{{ $t('common.noImportTemplate') }}</div>
          <q-btn color="primary" :label="$t('common.createFirstTemplate')" class="q-mt-md" @click="openCreateDialog" />
        </div>
      </template>
    </q-table>

    <!-- 模板预览对话框 -->
    <q-dialog v-model="showPreviewDialog" persistent>
      <q-card style="min-width: 800px">
        <q-card-section class="row items-center">
          <div class="text-h6">模板预览 - {{ previewTemplateData?.name }}</div>
          <q-space />
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-separator />

        <q-card-section style="max-height: 60vh; overflow-y: auto">
          <div v-if="previewTemplateData" class="template-preview">
            <!-- 基本信息 -->
            <div class="q-mb-lg">
              <div class="text-subtitle2 q-mb-sm">{{ $t('common.basicInfo') }}</div>
              <q-list dense bordered separator>
                <q-item>
                  <q-item-section>{{ $t('common.templateName') }}</q-item-section>
                  <q-item-section side>{{ previewTemplateData.name }}</q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>{{ $t('common.entityType') }}</q-item-section>
                  <q-item-section side>{{
                    getEntityLabel(previewTemplateData.entity_type)
                  }}</q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>{{ $t('common.description') }}</q-item-section>
                  <q-item-section side>{{ previewTemplateData.description || '-' }}</q-item-section>
                </q-item>
                <q-item>
                  <q-item-section>{{ $t('common.columnCount') }}</q-item-section>
                  <q-item-section side>{{
                    previewTemplateData.columns?.length || 0
                  }}</q-item-section>
                </q-item>
              </q-list>
            </div>

            <!-- 列配置 -->
            <div class="q-mb-lg">
              <div class="text-subtitle2 q-mb-sm">{{ $t('common.columnConfig') }}</div>
              <q-table
                :rows="previewTemplateData.columns || []"
                :columns="columnConfigColumns"
                row-key="field"
                flat
                bordered
                dense
              >
                <template v-slot:body-cell-required="props">
                  <q-td :props="props">
                    <q-icon
                      :name="props.row.required ? 'check_circle' : 'radio_button_unchecked'"
                      :color="props.row.required ? 'positive' : 'grey'"
                    />
                  </q-td>
                </template>
                <template v-slot:body-cell-options="props">
                  <q-td :props="props">
                    <template v-if="props.row.options?.length">
                      <q-badge
                        v-for="opt in props.row.options.slice(0, 2)"
                        :key="String(opt.value)"
                        color="secondary"
                        :label="String(opt.label)"
                        class="q-mr-xs"
                      />
                      <q-badge
                        v-if="props.row.options.length > 2"
                        color="grey"
                        :label="`+${props.row.options.length - 2}`"
                      />
                    </template>
                    <span v-else class="text-grey">-</span>
                  </q-td>
                </template>
              </q-table>
            </div>

            <!-- 示例数据 -->
            <div>
              <div class="text-subtitle2 q-mb-sm">{{ $t('common.sampleData') }}</div>
              <q-table
                :rows="previewSampleData"
                :columns="previewColumns"
                row-key="index"
                flat
                bordered
                dense
                hide-bottom
                :pagination="{ rowsPerPage: 0 }"
              />
            </div>
          </div>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.close')" v-close-popup />
          <q-btn
            color="primary"
            icon="download"
            :label="$t('common.downloadTemplate')"
            @click="downloadTemplate(previewTemplateData)"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 创建/编辑模板对话框 -->
    <q-dialog v-model="showEditDialog" persistent>
      <q-card style="min-width: 900px; max-width: 1000px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ isEditMode ? '编辑模板' : '新建模板' }}</div>
          <q-space />
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-separator />

        <q-card-section style="max-height: 70vh; overflow-y: auto">
          <q-form class="q-gutter-md">
            <!-- 基本信息 -->
            <div class="text-subtitle2">{{ $t('common.basicInfo') }}</div>
            <div class="row q-col-gutter-md">
              <div class="col-12 col-sm-6">
                <q-input
                  v-model="formData.name"
                  outlined
                  label="模板名称 *"
                  :rules="[(val) => !!val || '请输入模板名称']"
                />
              </div>
              <div class="col-12 col-sm-6">
                <q-select
                  v-model="formData.entity_type"
                  :options="entity_type_options"
                  outlined
                  label="实体类型 *"
                  emit-value
                  map-options
                  :rules="[(val) => !!val || '请选择实体类型']"
                />
              </div>
            </div>
            <div class="col-12">
              <q-input
                v-model="formData.description"
                outlined
                :label="$t('common.description')"
                type="textarea"
                rows="2"
              />
            </div>

            <q-separator />

            <!-- 列配置 -->
            <div class="row items-center q-mb-sm">
              <div class="text-subtitle2">{{ $t('common.columnConfig') }}</div>
              <q-space />
              <q-btn flat dense color="primary" icon="add" :label="$t('common.addColumn')" @click="addColumn" />
            </div>

            <div class="columns-editor">
              <div
                v-for="(col, index) in formData.columns"
                :key="index"
                class="column-item q-pa-md q-mb-sm rounded-borders"
              >
                <div class="row q-col-gutter-md items-center">
                  <div class="col-12 col-sm-2">
                    <q-input v-model="col.header" outlined dense label="表头名称 *" />
                  </div>
                  <div class="col-12 col-sm-2">
                    <q-input v-model="col.field" outlined dense label="字段名 *" />
                  </div>
                  <div class="col-12 col-sm-2">
                    <q-select
                      v-model="col.dataType"
                      :options="dataTypeOptions"
                      outlined
                      dense
                      :label="$t('common.dataType')"
                    />
                  </div>
                  <div class="col-12 col-sm-2">
                    <q-toggle v-model="col.required" :label="$t('common.required')" />
                  </div>
                  <div class="col-12 col-sm-3">
                    <q-select
                      v-if="col.dataType === 'select'"
                      v-model="col.options"
                      multiple
                      use-chips
                      outlined
                      dense
                      :label="$t('common.options')"
                      :options="[]"
                      new-value-mode="add-unique"
                    />
                  </div>
                  <div class="col-12 col-sm-1">
                    <q-btn flat dense color="negative" icon="delete" @click="removeColumn(index)" />
                  </div>
                </div>
              </div>

              <div v-if="formData.columns.length === 0" class="text-center q-pa-lg text-grey">
                {{ $t('common.noColumnConfig') }}
              </div>
            </div>
          </q-form>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn
            color="primary"
            :label="isEditMode ? '保存修改' : '创建模板'"
            @click="handleSave"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
/**
 * @file ImportTemplateManager.vue
 * @description 导入模板管理组件 — 业务逻辑已提取至 useImportTemplateManager composable
 * @date 2026-04-04
 */

import { onMounted } from 'vue';
import { useImportTemplateManager } from '@erp-new-frontend-monorepo/composables/src/useImportTemplateManager';;

const {
  // 常量
  entity_type_options,
  dataTypeOptions,
  // 状态
  loading,
  searchQuery,
  entity_type_filter,
  showPreviewDialog,
  showEditDialog,
  isEditMode,
  previewTemplateData,
  previewSampleData,
  // 表格列
  columns,
  columnConfigColumns,
  previewColumns,
  // 表单
  formData,
  // 计算属性
  filteredTemplates,
  // 方法
  getEntityLabel,
  previewTemplate,
  downloadTemplate,
  openCreateDialog,
  openEditDialog,
  addColumn,
  removeColumn,
  handleSave,
  deleteTemplate,
  initData,
} = useImportTemplateManager();

onMounted(() => {
  void initData();
});
</script>

<style scoped>
.import-template-manager {
  padding: 16px;
}

.rounded-borders {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
}

.body--dark .rounded-borders {
  border-color: #404040;
}

.column-item {
  background: #f5f5f5;
}

.body--dark .column-item {
  background: #2d2d2d;
}
</style>
