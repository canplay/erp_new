/**
 * @file FieldPermissionPanel.vue
 * @description 字段权限配置面板
 * @date 2026-04-04
 * @features
 * - 按实体类型配置字段权限
 * - 支持查看/编辑权限分离
 */

<template>
  <div class="field-permission-panel">
    <q-card flat bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-sm">
          <q-icon name="view_agenda" class="q-mr-sm" color="primary" />
          {{ $t('permission.fieldPermission') }}
        </div>
        <div class="text-caption text-grey q-mb-md">
          控制角色对特定字段的查看和编辑权限
        </div>

        <!-- 字段权限说明 -->
        <q-banner class="q-mb-md" rounded banner-class="bg-orange-1">
          <template v-slot:avatar>
            <q-icon name="info" color="orange" />
          </template>
          <div class="text-body2">
            <strong>字段权限说明：</strong>
            <ul class="q-my-sm" style="padding-left: 20px">
              <li><strong>可查看</strong> - 可以在详情页或列表中看到该字段</li>
              <li><strong>可编辑</strong> - 可以修改该字段的值（需要先有查看权限）</li>
              <li>敏感字段（如手机号、身份证号）建议仅对必要角色开放查看权限</li>
            </ul>
          </div>
        </q-banner>

        <!-- 实体类型选择 -->
        <div class="q-mb-md">
          <q-select
            v-model="selected_entity_type"
            :options="entity_type_options"
            outlined
            dense
            :label="$t('common.selectEntityType')"
            emit-value
            map-options
          />
        </div>

        <!-- 字段权限列表 -->
        <q-table
          :rows="fieldPermissions"
          :columns="columns"
          row-key="field"
          flat
          bordered
          :pagination="{ rowsPerPage: 10 }"
        >
          <!-- 字段名称 -->
          <template v-slot:body-cell-field="props">
            <q-td :props="props">
              <div>
                <div class="text-weight-medium">{{ props.row.field }}</div>
                <div class="text-caption text-grey">{{ props.row.description }}</div>
              </div>
            </q-td>
          </template>

          <!-- 可查看 -->
          <template v-slot:body-cell-canView="props">
            <q-td :props="props">
              <q-toggle
                v-model="props.row.canView"
                color="primary"
                :disable="disabled"
                @update:model-value="handlePermissionChange"
              />
            </q-td>
          </template>

          <!-- 可编辑 -->
          <template v-slot:body-cell-canEdit="props">
            <q-td :props="props">
              <q-toggle
                v-model="props.row.canEdit"
                color="positive"
                :disable="disabled || !props.row.canView"
                @update:model-value="handlePermissionChange"
              />
            </q-td>
          </template>

          <!-- 敏感字段标识 -->
          <template v-slot:body-cell-sensitive="props">
            <q-td :props="props">
              <q-badge
                v-if="props.row.sensitive"
                color="warning"
                text-color="black"
                label="敏感"
              />
            </q-td>
          </template>
        </q-table>
      </q-card-section>

      <!-- 保存按钮 -->
      <q-separator />
      <q-card-actions align="right">
        <q-btn flat color="grey" :label="$t('common.reset')" @click="resetToDefault" />
        <q-btn
          color="primary"
          :label="$t('common.save')"
          :loading="saving"
          @click="saveFieldPermissions"
        />
      </q-card-actions>
    </q-card>
  </div>
</template>

<script setup lang="ts">
/**
 * @file FieldPermissionPanel.vue
 * @description 字段权限配置面板组件
 */

import { ref, watch } from 'vue';
import { logger } from '@/utils/logger';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import type { QTableProps } from 'quasar';
import type { FieldPermission } from '@/types/permission';
import { updateRoleFieldPermissions } from '@/api/permission';

const $q = useQuasar();
const { t: $t } = useI18n();

// ============ Props & Emits ============

const props = defineProps<{
  /** 角色名称 */
  role_name: string;
  /** 初始字段权限配置 */
  initialFieldPermissions?: FieldPermission[];
  /** 是否禁用 */
  disabled?: boolean;
}>();

const emit = defineEmits<{
  /** 保存成功 */
  saved: [fieldPermissions: FieldPermission[]];
  /** 变更 */
  changed: [fieldPermissions: FieldPermission[]];
}>();

// ============ 类型定义 ============

interface FieldPermissionItem extends FieldPermission {
  description?: string;
  sensitive?: boolean;
}

// ============ 状态 ============

/** 选中的实体类型 */
const selected_entity_type = ref('user');

/** 是否正在保存 */
const saving = ref(false);

/** 字段权限列表 */
const fieldPermissions = ref<FieldPermissionItem[]>([]);

// ============ 常量 ============

/** 实体类型选项 */
const entity_type_options = [
  { label: '用户信息', value: 'user' },
  { label: '客户信息', value: 'customer' },
  { label: '订单信息', value: 'order' },
];

/** 预定义的字段配置 */
const fieldConfigs: Record<string, FieldPermissionItem[]> = {
  user: [
    { field: 'id', entity_type: 'user', canView: true, canEdit: false, description: '用户ID', sensitive: false },
    { field: 'username', entity_type: 'user', canView: true, canEdit: true, description: '用户名', sensitive: false },
    { field: 'nickname', entity_type: 'user', canView: true, canEdit: true, description: '昵称', sensitive: false },
    { field: 'email', entity_type: 'user', canView: true, canEdit: true, description: '邮箱', sensitive: false },
    { field: 'phone', entity_type: 'user', canView: true, canEdit: true, description: '手机号', sensitive: true },
    { field: 'idcard', entity_type: 'user', canView: false, canEdit: false, description: '身份证号', sensitive: true },
    { field: 'avatar', entity_type: 'user', canView: true, canEdit: true, description: '头像', sensitive: false },
    { field: 'role', entity_type: 'user', canView: true, canEdit: true, description: '角色', sensitive: false },
    { field: 'status', entity_type: 'user', canView: true, canEdit: true, description: '状态', sensitive: false },
    { field: 'createTime', entity_type: 'user', canView: true, canEdit: false, description: '创建时间', sensitive: false },
  ],
  customer: [
    { field: 'id', entity_type: 'customer', canView: true, canEdit: false, description: '客户ID', sensitive: false },
    { field: 'name', entity_type: 'customer', canView: true, canEdit: true, description: '客户名称', sensitive: false },
    { field: 'contact', entity_type: 'customer', canView: true, canEdit: true, description: '联系人', sensitive: false },
    { field: 'phone', entity_type: 'customer', canView: true, canEdit: true, description: '联系电话', sensitive: true },
    { field: 'address', entity_type: 'customer', canView: true, canEdit: true, description: '地址', sensitive: false },
    { field: 'level', entity_type: 'customer', canView: true, canEdit: true, description: '客户等级', sensitive: false },
  ],
  order: [
    { field: 'id', entity_type: 'order', canView: true, canEdit: false, description: '订单ID', sensitive: false },
    { field: 'orderNo', entity_type: 'order', canView: true, canEdit: false, description: '订单号', sensitive: false },
    { field: 'customerId', entity_type: 'order', canView: true, canEdit: false, description: '客户ID', sensitive: false },
    { field: 'amount', entity_type: 'order', canView: true, canEdit: true, description: '订单金额', sensitive: false },
    { field: 'status', entity_type: 'order', canView: true, canEdit: true, description: '订单状态', sensitive: false },
    { field: 'createTime', entity_type: 'order', canView: true, canEdit: false, description: '下单时间', sensitive: false },
  ],
};

/** 表格列定义 */
const columns: QTableProps['columns'] = [
  { name: 'field', label: '字段', field: 'field', align: 'left', sortable: true },
  { name: 'canView', label: '可查看', field: 'canView', align: 'center' },
  { name: 'canEdit', label: '可编辑', field: 'canEdit', align: 'center' },
  { name: 'sensitive', label: '敏感', field: 'sensitive', align: 'center' },
];

// ============ 方法 ============

/**
 * @brief 初始化字段权限
 */
function initFieldPermissions() {
  const config = fieldConfigs[selected_entity_type.value] || [];
  const initialMap = new Map(
    props.initialFieldPermissions
      ?.filter((fp) => fp.entity_type === selected_entity_type.value)
      .map((fp) => [fp.field, fp]) || []
  );

  fieldPermissions.value = config.map((field) => {
    const initial = initialMap.get(field.field);
    return {
      ...field,
      canView: initial?.canView ?? field.canView,
      canEdit: initial?.canEdit ?? field.canEdit,
    };
  });
}

/**
 * @brief 处理权限变更
 */
function handlePermissionChange() {
  // 如果关闭查看权限，同时关闭编辑权限
  fieldPermissions.value.forEach((field) => {
    if (!field.canView) {
      field.canEdit = false;
    }
  });
  emit('changed', getFieldPermissions());
}

/**
 * @brief 获取字段权限配置
 */
function getFieldPermissions(): FieldPermission[] {
  return fieldPermissions.value
    .filter((f) => !f.canView || !f.canEdit) // 只保存非默认值的
    .map((f) => ({
      field: f.field,
      entity_type: f.entity_type,
      canView: f.canView,
      canEdit: f.canEdit,
    }));
}

/**
 * @brief 重置为默认
 */
function resetToDefault() {
  initFieldPermissions();
  emit('changed', getFieldPermissions());
}

/**
 * @brief 保存字段权限
 */
async function saveFieldPermissions() {
  saving.value = true;
  try {
    const fieldPermissions = getFieldPermissions();
    await updateRoleFieldPermissions(props.role_name, fieldPermissions);
    $q.notify({
      type: 'positive',
      message: '字段权限保存成功',
    });
    emit('saved', fieldPermissions);
  } catch (error) {
    logger.error('【保存字段权限失败】', error);
    $q.notify({
      type: 'negative',
      message: '保存失败，请重试',
    });
  } finally {
    saving.value = false;
  }
}

// ============ 监听器 ============

watch(selected_entity_type, () => {
  initFieldPermissions();
});

watch(
  () => props.initialFieldPermissions,
  () => {
    initFieldPermissions();
  },
  { immediate: true, deep: true }
);
</script>

<style scoped>
.field-permission-panel {
  width: 100%;
}
</style>

