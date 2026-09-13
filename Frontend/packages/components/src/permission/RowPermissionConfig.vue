/**
 * @file RowPermissionConfig.vue
 * @description 行级权限配置组件 - 主文件（拆分后 行）
 * @date 2026-04-04
 */

<template>
  <div class="row-permission-config">
    <q-card flat bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-sm">
          <q-icon name="rule" class="q-mr-sm" color="primary" />
          行级权限配置
        </div>
        <div class="text-caption text-grey q-mb-md">
          配置行级数据访问规则，细粒度控制角色可访问的数据行
        </div>
        <div class="row q-mb-md items-center">
          <q-select v-model="selected_entity_type" :options="entity_type_options" outlined dense
            :label="$t('common.entityType')" emit-value map-options style="min-width: 200px" class="q-mr-md" />
          <q-select v-model="selectedRoleName" :options="roleOptions" outlined dense
            :label="$t('common.role')" emit-value map-options clearable style="min-width: 150px" />
          <q-space />
          <q-btn color="primary" icon="add" :label="$t('common.newRule')" @click="openCreateDialog" />
        </div>
        <q-table :rows="rowPermissions" :columns="columns" row-key="id" flat bordered
          :loading="loading" :pagination="{ rowsPerPage: 10 }">
          <template v-slot:body-cell-name="props">
            <q-td :props="props">
              <div class="row items-center">
                <q-icon name="vpn_key" color="primary" size="20px" class="q-mr-sm" />
                <div>
                  <div class="text-weight-medium">{{ props.row.name }}</div>
                  <div class="text-caption text-grey">{{ props.row.permission }}</div>
                </div>
              </div>
            </q-td>
          </template>
          <template v-slot:body-cell-filter="props">
            <q-td :props="props">
              <RowPermissionPreview
                :rules="props.row.filterGroup?.rules || []"
                :logic="props.row.filterGroup?.logic || 'and'"
                :available-fields="getAvailableFields(props.row.entity_type)"
              />
            </q-td>
          </template>
          <template v-slot:body-cell-allowCustomize="props">
            <q-td :props="props">
              <q-icon :name="props.row.allowCustomize ? 'check_circle' : 'cancel'"
                :color="props.row.allowCustomize ? 'positive' : 'grey'" size="20px" />
            </q-td>
          </template>
          <template v-slot:body-cell-enabled="props">
            <q-td :props="props">
              <q-toggle v-model="props.row.enabled" color="primary"
                @update:model-value="handleToggleEnabled(props.row)" />
            </q-td>
          </template>
          <template v-slot:body-cell-actions="props">
            <q-td :props="props">
              <q-btn flat dense color="primary" icon="edit" @click="openEditDialog(props.row)">
                <q-tooltip>{{ $t('common.edit') }}</q-tooltip>
              </q-btn>
              <q-btn flat dense color="info" icon="content_copy" @click="duplicateRule(props.row)">
                <q-tooltip>复制</q-tooltip>
              </q-btn>
              <q-btn flat dense color="negative" icon="delete" @click="handleDelete(props.row)">
                <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
              </q-btn>
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>
    <q-dialog v-model="showDialog" persistent>
      <q-card style="min-width: 700px; max-width: 900px">
        <q-card-section>
          <div class="text-h6">{{ isEditMode ? '编辑行级权限规则' : '新建行级权限规则' }}</div>
        </q-card-section>
        <q-separator />
        <q-card-section style="max-height: 70vh; overflow-y: auto">
          <q-form class="q-gutter-md">
            <div class="text-subtitle2 q-mb-sm">{{ $t('common.basicInfo') }}</div>
            <div class="row q-col-gutter-md">
              <div class="col-12 col-sm-6">
                <q-input v-model="formData.name" outlined label="规则名称 *"
                  :rules="[(val) => !!val || '请输入规则名称']" />
              </div>
              <div class="col-12 col-sm-6">
                <q-input v-model="formData.permission" outlined label="关联权限标识 *"
                  placeholder="如: user:view"
                  :rules="[(val) => !!val || '请输入权限标识']" />
              </div>
            </div>
            <div class="row q-col-gutter-md">
              <div class="col-12 col-sm-6">
                <q-select v-model="formData.entity_type" :options="entity_type_options"
                  outlined label="实体类型 *" emit-value map-options
                  :rules="[(val) => !!val || '请选择实体类型']" />
              </div>
              <div class="col-12 col-sm-6">
                <q-toggle v-model="formData.enabled" :label="$t('common.enableNow')" color="primary" />
                <q-toggle v-model="formData.allowCustomize" :label="$t('common.allowUserCustomize')" color="positive" />
              </div>
            </div>
            <q-separator />
            <RowConditionEditor v-model="formData.filterGroup"
              :available-fields="getAvailableFields(formData.entity_type)"
              @saved="handleFilterSaved" />
          </q-form>
        </q-card-section>
        <q-separator />
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" color="grey" v-close-popup />
          <q-btn color="primary" :label="isEditMode ? '保存修改' : '创建规则'" @click="handleSave" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { logger } from '@/utils/logger';
import { useQuasar } from 'quasar';
import type { QTableProps } from 'quasar';
import type { RowPermission, FilterGroup, FilterRule } from '@/types/rowPermission';
import { FilterOperator, LogicalOperator, createEmptyFilterGroup, OPERATOR_OPTIONS } from '@/types/rowPermission';
import RowConditionEditor from './RowConditionEditor.vue';
import RowPermissionPreview from './RowPermissionPreview.vue';
import { EntityTypeOptions, RoleOptions, entityFields, defaultPermissions } from './RowPermissionConfig';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
const $q = useQuasar();

const props = defineProps<{
  role_name?: string;
  initialRowPermissions?: RowPermission[];
}>();

const emit = defineEmits<{
  saved: [permissions: RowPermission[]];
  deleted: [id: number];
}>();

const loading = ref(false);
const selected_entity_type = ref('user');
const selectedRoleName = ref('');
const rowPermissions = ref<RowPermission[]>([]);
const showDialog = ref(false);
const isEditMode = ref(false);
const formData = reactive<{
  id?: number; name: string; permission: string; entity_type: string;
  filterGroup: FilterGroup; enabled: boolean; allowCustomize: boolean;
}>({
  name: '', permission: '', entity_type: 'user',
  filterGroup: createEmptyFilterGroup(), enabled: true, allowCustomize: false,
});
let idCounter = 1000;

const columns: QTableProps['columns'] = [
  { name: 'name', label: '规则名称', field: 'name', align: 'left', sortable: true },
  { name: 'filter', label: '过滤条件', field: 'filterGroup', align: 'left' },
  { name: 'allowCustomize', label: '允许自定义', field: 'allowCustomize', align: 'center' },
  { name: 'enabled', label: '启用', field: 'enabled', align: 'center' },
  { name: 'actions', label: '操作', field: 'actions', align: 'center' },
];

const entity_type_options = computed(() => EntityTypeOptions);
const roleOptions = computed(() => RoleOptions);

function getFieldLabel(field: string): string {
  const fields = entityFields[selected_entity_type.value] || [];
  const config = fields.find(f => f.field === field);
  return config?.label || field;
}

function getOperatorLabel(operator: FilterOperator): string {
  const option = OPERATOR_OPTIONS.find(o => o.value === operator);
  return option?.label || operator;
}

function formatRuleValue(rule: FilterRule): string {
  const val = rule.value;
  if (Array.isArray(val)) return `(${val.join(', ')})`;
  if (val === null || val === undefined) return '';
  if (rule.operator === FilterOperator.BETWEEN) {
    const v1 = val != null ? String(val) : '';
    const v2 = rule.value2 != null ? String(rule.value2) : '';
    return `${v1} ~ ${v2}`;
  }
  return String(val);
}

function getAvailableFields(entity_type: string) {
  return entityFields[entity_type] || [];
}

function openCreateDialog() {
  isEditMode.value = false;
  Object.assign(formData, {
    id: undefined, name: '', permission: '', entity_type: selected_entity_type.value,
    filterGroup: createEmptyFilterGroup(), enabled: true, allowCustomize: false,
  });
  showDialog.value = true;
}

function openEditDialog(row: RowPermission) {
  isEditMode.value = true;
  Object.assign(formData, {
    id: row.id, name: row.name, permission: row.permission,
    entity_type: row.entity_type, filterGroup: { ...row.filterGroup },
    enabled: row.enabled, allowCustomize: row.allowCustomize,
  });
  showDialog.value = true;
}

function duplicateRule(row: RowPermission) {
  const newId = ++idCounter;
  const duplicated: RowPermission = {
    ...row, id: newId, name: `${row.name} (副本)`,
    created_at: new Date().toISOString(), updated_at: new Date().toISOString(),
  };
  rowPermissions.value.push(duplicated);
  $q.notify({ type: 'positive', message: '规则已复制' });
}

function handleToggleEnabled(row: RowPermission) {
  logger.info('【行级权限】切换启用状态：', row.id, row.enabled);
}

function handleDelete(row: RowPermission) {
  $q.dialog({
    title: '确认删除',
    message: `确定要删除规则「${row.name}」吗？此操作不可恢复。`,
    ok: { label: '删除', color: 'negative' },
    cancel: { label: '取消', flat: true },
  }).onOk(() => {
    rowPermissions.value = rowPermissions.value.filter(r => r.id !== row.id);
    emit('deleted', row.id);
    $q.notify({ type: 'positive', message: '规则已删除' });
  });
}

function handleFilterSaved(_filterGroup: FilterGroup) {
  logger.info('【行级权限】过滤条件已保存：', _filterGroup);
}

function handleSave() {
  if (!formData.name || !formData.permission) {
    $q.notify({ type: 'warning', message: '请填写必填字段' });
    return;
  }
  const now = new Date().toISOString();
  if (isEditMode.value && formData.id) {
    const index = rowPermissions.value.findIndex(r => r.id === formData.id);
    if (index > -1) {
      const existing = rowPermissions.value[index]!;
      rowPermissions.value[index] = {
        ...existing, name: formData.name, permission: formData.permission,
        entity_type: formData.entity_type, filterGroup: { ...formData.filterGroup },
        enabled: formData.enabled, allowCustomize: formData.allowCustomize, updated_at: now,
      };
    }
  } else {
    const newPermission: RowPermission = {
      id: ++idCounter, name: formData.name, permission: formData.permission,
      entity_type: formData.entity_type, filterGroup: { ...formData.filterGroup },
      enabled: formData.enabled, allowCustomize: formData.allowCustomize,
      created_at: now, updated_at: now,
    };
    rowPermissions.value.push(newPermission);
  }
  showDialog.value = false;
  emit('saved', rowPermissions.value);
  $q.notify({ type: 'positive', message: isEditMode.value ? '规则已更新' : '规则已创建' });
}

function loadRowPermissions() {
  loading.value = true;
  setTimeout(() => {
    rowPermissions.value = props.initialRowPermissions || defaultPermissions;
    loading.value = false;
  }, 500);
}

onMounted(() => {
  if (props.role_name) selectedRoleName.value = props.role_name;
  loadRowPermissions();
});
</script>

<style scoped>
.row-permission-config { width: 100%; }
.filter-preview { display: flex; flex-wrap: wrap; gap: 4px; align-items: center; }
</style>
