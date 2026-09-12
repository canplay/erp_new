/**
 * @file DataPermissionPanel.vue
 * @description 数据权限配置面板
 * @date 2026-04-04
 * @refactored 2026-09-12 - Split into composables
 */

<template>
  <div class="data-permission-panel">
    <q-card flat bordered>
      <q-card-section>
        <div class="text-subtitle1 q-mb-sm">
          <q-icon name="storage" class="q-mr-sm" color="primary" />
          {{ $t('permission.dataPermission') }}
        </div>
        <div class="text-caption text-grey q-mb-md">
          控制角色可以访问的数据范围（部门/租户/创建者）
        </div>

        <!-- 数据权限说明 -->
        <q-banner class="q-mb-md" rounded banner-class="bg-blue-1">
          <template v-slot:avatar>
            <q-icon name="info" color="blue" />
          </template>
          <div class="text-body2">
            <strong>数据权限范围说明：</strong>
            <ul class="q-my-sm" style="padding-left: 20px">
              <li><strong>全部数据</strong> - 可以访问所有数据</li>
              <li><strong>本部门</strong> - 仅能访问本部门创建的数据</li>
              <li><strong>本部门及下级</strong> - 可访问本部门及子部门的数据</li>
              <li><strong>仅本人</strong> - 仅能访问自己创建的数据</li>
              <li><strong>自定义</strong> - 可指定特定部门的数据访问权限</li>
            </ul>
          </div>
        </q-banner>

        <!-- 权限模块列表 -->
        <q-list separator>
          <q-item v-for="item in permissionDataScopes" :key="item.key" clickable>
            <q-item-section avatar>
              <q-icon :name="getModuleIcon(item.module)" color="primary" />
            </q-item-section>

            <q-item-section>
              <q-item-label>{{ item.name }}</q-item-label>
              <q-item-label caption>
                {{ item.description }}
              </q-item-label>
            </q-item-section>

            <q-item-section side>
              <q-select
                v-model="item.scope"
                :options="scopeOptions"
                dense
                outlined
                emit-value
                map-options
                style="min-width: 150px"
                @update:model-value="(val) => handleScopeChange(item, val)"
              />
            </q-item-section>

            <!-- 自定义范围选择 -->
            <q-item-section v-if="item.scope === PermissionScope.CUSTOM" side>
              <q-btn
                flat
                dense
                color="primary"
                label="选择部门"
                icon="corporate_fare"
                @click="openDepartmentSelector(item)"
              />
            </q-item-section>
          </q-item>
        </q-list>

        <!-- 自定义部门已选显示 -->
        <div v-if="hasCustomScope" class="q-mt-md">
          <div class="text-caption text-grey q-mb-sm">已选择的部门：</div>
          <div class="row q-gutter-sm">
            <q-badge
              v-for="dept in selectedDepartments"
              :key="dept.id"
              color="primary"
              text-color="white"
              class="q-pa-sm"
            >
              {{ dept.name }}
              <q-icon
                name="close"
                size="xs"
                class="q-ml-xs cursor-pointer"
                @click="removeDepartment(dept.id)"
              />
            </q-badge>
          </div>
        </div>
      </q-card-section>

      <!-- 数据权限应用范围 -->
      <q-separator />
      <q-card-section>
        <div class="text-subtitle2 q-mb-md">
          <q-icon name="account_tree" class="q-mr-sm" color="primary" />
          数据隔离维度
        </div>

        <q-option-group
          v-model="isolationDimension"
          :options="isolationOptions"
          color="primary"
          inline
        />
      </q-card-section>

      <!-- 保存按钮 -->
      <q-separator />
      <q-card-actions align="right">
        <q-btn flat color="grey" :label="$t('common.reset')" @click="resetToDefault" />
        <q-btn
          color="primary"
          :label="$t('common.save')"
          :loading="saving"
          @click="handleSave"
        />
      </q-card-actions>
    </q-card>

    <!-- 部门选择对话框 -->
    <q-dialog v-model="showDepartmentDialog" persistent>
      <q-card style="min-width: 400px">
        <q-card-section class="row items-center">
          <div class="text-h6">选择部门</div>
          <q-space />
          <q-btn icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-separator />

        <q-card-section style="max-height: 400px; overflow-y: auto">
          <q-tree
            v-model:selected="selectedDepartmentId"
            v-model:ticked="tickedDepartments"
            :nodes="departmentTree"
            node-key="id"
            label-key="name"
            selected-color="primary"
            tick-strategy="leaf"
            default-expand-all
          />
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat color="grey" :label="$t('common.cancel')" v-close-popup />
          <q-btn color="primary" :label="$t('common.confirm')" @click="confirmDepartmentSelection" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { PermissionScope } from '@/types/permission';
import { useDataPermissionPanel } from './composables/useDataPermissionPanel';

// ============ Props & Emits ============

const props = defineProps<{
  role_name: string;
  initialDataPermissions?: Array<{ permission: string; scope: string; custom_scope?: number[] }>;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  saved: [dataPermissions: Array<{ permission: string; scope: string; custom_scope?: number[] }>];
  changed: [dataPermissions: Array<{ permission: string; scope: string; custom_scope?: number[] }>];
}>();

// ============ Composable ============

const {
  permissionDataScopes,
  isolationDimension,
  saving,
  showDepartmentDialog,
  selectedDepartmentId,
  tickedDepartments,
  currentEditingItem,
  departmentTree,
  selectedDepartments,
  scopeOptions,
  isolationOptions,
  hasCustomScope,
  getModuleIcon,
  openDepartmentSelector,
  confirmDepartmentSelection,
  removeDepartment,
  handleScopeChange,
  getDataPermissions,
  resetToDefault,
  saveDataPermissions,
} = useDataPermissionPanel({
  role_name: props.role_name,
  initialDataPermissions: props.initialDataPermissions,
  disabled: props.disabled,
});

// ============ Methods ============

async function handleSave() {
  const result = await saveDataPermissions();
  if (result) {
    emit('saved', result);
  }
}
</script>

<style scoped>
.data-permission-panel {
  width: 100%;
}

.cursor-pointer {
  cursor: pointer;
}
</style>
