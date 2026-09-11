<template>
  <div class="form-builder">
    <!-- 页面标题 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">
        <q-icon name="dynamic_form" class="q-mr-sm" />
        表单生成器
      </div>
      <q-space />
      <q-btn flat color="grey" icon="save" :label="$t('common.save')" @click="handleSave" />
      <q-btn color="primary" icon="preview" :label="$t('common.preview')" @click="handlePreview" />
    </div>

    <div class="row q-col-gutter-md">
      <!-- 左侧：组件面板 -->
      <div class="col-12 col-md-3">
        <q-card flat bordered>
          <q-card-section>
            <div class="text-subtitle1 q-mb-md">组件库</div>
            <q-input
              v-model="componentSearch"
              outlined
              dense
              :placeholder="$t('common.search')"
              class="q-mb-md"
            >
              <template v-slot:prepend>
                <q-icon name="search" />
              </template>
            </q-input>
            <q-list>
              <q-expansion-item
                v-for="category in componentCategories"
                :key="category.name"
                :label="category.name"
                icon="folder"
                default-opened
              >
                <q-list class="q-pl-md">
                  <q-item
                    v-for="comp in category.components"
                    :key="comp.type"
                    clickable
                    v-ripple
                    draggable="true"
                    @dragstart="handleDragStart($event, comp)"
                  >
                    <q-item-section avatar>
                      <q-icon :name="comp.icon" :color="comp.color" />
                    </q-item-section>
                    <q-item-section>{{ comp.label }}</q-item-section>
                  </q-item>
                </q-list>
              </q-expansion-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>

      <!-- 中间：表单设计区 -->
      <div class="col-12 col-md-6">
        <q-card flat bordered class="form-canvas">
          <q-card-section>
            <div class="row items-center q-mb-md">
              <q-input
                v-model="formConfig.name"
                outlined
                dense
                placeholder="表单名称"
                class="col"
              />
              <q-btn flat round icon="more_vert">
                <q-menu>
                  <q-list style="min-width: 150px">
                    <q-item clickable v-close-popup @click="handleCopyConfig">
                      <q-item-section avatar><q-icon name="content_copy" /></q-item-section>
                      <q-item-section>复制表单</q-item-section>
                    </q-item>
                    <q-item clickable v-close-popup @click="handleImportConfig">
                      <q-item-section avatar><q-icon name="upload" /></q-item-section>
                      <q-item-section>导入配置</q-item-section>
                    </q-item>
                    <q-item clickable v-close-popup @click="handleExportConfig">
                      <q-item-section avatar><q-icon name="download" /></q-item-section>
                      <q-item-section>导出配置</q-item-section>
                    </q-item>
                  </q-list>
                </q-menu>
              </q-btn>
            </div>

            <!-- 工具栏 -->
            <div class="row items-center q-mb-md">
              <q-btn flat round icon="undo" size="sm" @click="handleUndo" />
              <q-btn flat round icon="redo" size="sm" @click="handleRedo" />
              <q-separator vertical class="q-mx-sm" />
              <q-btn flat round icon="add" size="sm" @click="handleAddGroup" />
            </div>

            <!-- 表单设计区域 -->
            <div
              class="form-design-area"
              @dragover.prevent
              @drop="handleDrop"
            >
              <!-- 空状态 -->
              <div v-if="formConfig.groups.length === 0" class="empty-state">
                <q-icon name="dynamic_form" size="64px" color="grey-4" />
                <div class="text-grey-6 q-mt-md">从左侧拖入组件或添加分组</div>
              </div>

              <!-- 分组列表 -->
              <div v-else>
                <div
                  v-for="(group, groupIndex) in formConfig.groups"
                  :key="group.id"
                  class="form-group"
                >
                  <!-- 分组头 -->
                  <div class="row items-center q-pa-sm bg-grey-1">
                    <q-icon name="folder" class="q-mr-sm" />
                    <span class="text-subtitle2">{{ group.name }}</span>
                    <q-space />
                    <q-btn flat round icon="add_circle" size="sm" @click="handleAddField(groupIndex)">
                      <q-tooltip>添加字段</q-tooltip>
                    </q-btn>
                    <q-btn flat round icon="edit" size="sm" @click="handleEditGroup(groupIndex)">
                      <q-tooltip>编辑分组</q-tooltip>
                    </q-btn>
                    <q-btn flat round icon="delete" size="sm" @click="handleDeleteGroup(groupIndex)">
                      <q-tooltip>删除分组</q-tooltip>
                    </q-btn>
                  </div>

                  <!-- 分组内容 -->
                  <div class="group-fields">
                    <!-- 空分组提示 -->
                    <div
                      v-if="group.fields.length === 0"
                      class="text-grey-5 text-center q-pa-md"
                      style="border: 1px dashed #e0e0e0; border-radius: 8px;"
                    >
                      拖入组件到此分组
                    </div>

                    <!-- 字段列表 -->
                    <div
                      v-for="(field, fieldIndex) in group.fields"
                      :key="field.id"
                      class="form-field-item"
                      :class="{ 'form-field-item--selected': selectedField?.id === field.id }"
                      @click="handleSelectField(field)"
                    >
                      <div class="field-drag-handle">
                        <q-icon name="drag_indicator" />
                      </div>
                      <div class="field-content">
                        <div class="field-label">{{ field.label || field.name }}</div>
                        <div class="field-type">
                          <q-badge :color="getFieldColor(field.type)" :label="field.type" />
                          <q-badge v-if="field.required" color="negative" label="必填" class="q-ml-xs" />
                        </div>
                      </div>
                      <div class="field-actions">
                        <q-btn flat round icon="content_copy" size="sm" @click.stop="handleCopyField(groupIndex, fieldIndex)">
                          <q-tooltip>复制字段</q-tooltip>
                        </q-btn>
                        <q-btn flat round icon="delete" size="sm" color="negative" @click.stop="handleDeleteField(groupIndex, fieldIndex)">
                          <q-tooltip>删除字段</q-tooltip>
                        </q-btn>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </q-card-section>
        </q-card>
      </div>

      <!-- 右侧：属性配置 -->
      <div class="col-12 col-md-3">
        <q-card flat bordered>
          <q-card-section>
            <div class="text-subtitle1 q-mb-md">属性配置</div>

            <!-- 未选中提示 -->
            <div v-if="!selectedField" class="text-grey-5 text-center q-pa-md">
              请选择一个字段进行配置
            </div>

            <!-- 字段配置表单 -->
            <div v-else class="field-config">
              <q-input v-model="selectedField.name" label="字段名称" outlined dense class="q-mb-sm" />
              <q-input v-model="selectedField.label" label="标签文本" outlined dense class="q-mb-sm" />
              <q-input v-model="selectedField.placeholder" label="占位提示" outlined dense class="q-mb-sm" />

              <q-select
                v-model="selectedField.type"
                :options="fieldTypeOptions"
                label="字段类型"
                outlined
                dense
                class="q-mb-sm"
                emit-value
                map-options
              />

              <q-input
                v-model.number="selectedField.span"
                label="栅格宽度"
                type="number"
                outlined
                dense
                class="q-mb-sm"
              />

              <q-toggle v-model="selectedField.required" label="必填" class="q-mb-sm" />
              <q-toggle v-model="selectedField.disabled" label="禁用" />
            </div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 预览对话框 -->
    <q-dialog v-model="showPreviewDialog" maximized>
      <q-card>
        <q-card-section class="row items-center">
          <div class="text-h6">表单预览</div>
          <q-space />
          <q-btn flat round icon="close" v-close-popup />
        </q-card-section>
        <q-separator />
        <q-card-section style="max-width: 800px; margin: 0 auto;">
          <DynamicForm :config="formConfig" @submit="handleFormSubmit" @reset="handleFormReset" />
        </q-card-section>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * @file FormBuilder.vue
 * @description 表单生成器组件
 * @date 2026-04-04
 */

import { onMounted } from 'vue';
import DynamicForm from './DynamicForm.vue';
import { useFormBuilder } from '@/composables/useFormBuilder';

const {
  componentSearch,
  showPreviewDialog,
  formConfig,
  componentCategories,
  fieldTypeOptions,
  selectedField,
  getFieldColor,
  handleDragStart,
  handleDrop,
  handleAddGroup,
  handleEditGroup,
  handleDeleteGroup,
  handleAddField,
  handleSelectField,
  handleCopyField,
  handleDeleteField,
  handleSave,
  handlePreview,
  handleFormSubmit,
  handleFormReset,
  handleUndo,
  handleRedo,
  handleCopyConfig,
  handleImportConfig,
  handleExportConfig,
  initData,
} = useFormBuilder();

onMounted(() => {
  initData();
});
</script>

<style scoped>
.form-builder {
  padding: 16px;
}
.form-canvas {
  min-height: 600px;
}
.form-design-area {
  min-height: 400px;
  border: 2px dashed #e0e0e0;
  border-radius: 8px;
  padding: 16px;
  background: #fafafa;
}
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
}
.form-group {
  margin-bottom: 16px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  overflow: hidden;
}
.group-fields {
  padding: 16px;
  background: white;
}
.form-field-item {
  display: flex;
  align-items: center;
  padding: 12px;
  margin-bottom: 8px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}
.form-field-item:hover {
  border-color: #1976d2;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
.form-field-item--selected {
  border-color: #1976d2;
  border-width: 2px;
  background: #e3f2fd;
}
.field-drag-handle {
  cursor: grab;
  padding: 0 8px;
  color: #9e9e9e;
}
.field-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.field-label {
  font-weight: 500;
  font-size: 14px;
}
.field-type {
  margin-top: 4px;
}
.field-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}
.form-field-item:hover .field-actions {
  opacity: 1;
}
.field-config {
  max-height: calc(100vh - 200px);
  overflow-y: auto;
}
</style>
