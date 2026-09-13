<template>
  <q-page class="q-pa-md role-permission-page">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg text-weight-bold">{{ $t('role.permissions') }}</div>

    <!-- 角色选择 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section>
        <div class="row items-center q-col-gutter-md">
          <div class="col-12 col-sm-4">
            <q-select
              v-model="selectedRole"
              :options="roleOptions"
              :label="$t('role.role_name')"
              outlined
              dense
              emit-value
              map-options
              @update:model-value="loadRolePermissions"
            />
          </div>
          <div class="col-12 col-sm-8 text-right">
            <q-btn
              v-if="hasChanges"
              color="warning"
              flat
              :label="$t('common.reset')"
              @click="resetChanges"
            />
            <q-btn
              v-if="hasChanges"
              color="primary"
              :label="$t('common.save')"
              :loading="saving"
              @click="savePermissions"
            />
          </div>
        </div>
      </q-card-section>
    </q-card>

    <template v-if="selectedRole">
      <q-card bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-md">{{ $t('role.assignPermissions') }}</div>
          <q-expansion-item
            v-for="group in permissionGroups"
            :key="group.module"
            :label="group.label"
            icon="security"
            default-opened
            class="permission-group"
          >
            <q-card flat bordered class="q-ma-sm">
              <q-card-section>
                <div class="row items-center q-mb-md">
                  <q-checkbox v-model="group.selected" :label="$t('table.selectAll')" dense @update:model-value="(val: unknown) => toggleGroup(group, val as boolean)" />
                  <q-space />
                  <span class="text-caption text-grey">{{ getSelectedCount(group) }} / {{ group.permissions.length }}</span>
                </div>
                <div class="row q-col-gutter-sm">
                  <div v-for="permission in group.permissions" :key="permission.key" class="col-12 col-sm-6 col-md-4">
                    <q-checkbox v-model="permission.selected" :label="permission.name" :disable="is_read_only" dense class="permission-checkbox">
                      <q-tooltip>{{ permission.description || permission.name }}</q-tooltip>
                    </q-checkbox>
                  </div>
                </div>
              </q-card-section>
            </q-card>
          </q-expansion-item>
        </q-card-section>
      </q-card>
    </template>

    <!-- 权限项管理 -->
    <q-card class="q-mt-lg" bordered>
      <q-card-section>
        <div class="row items-center q-mb-md">
          <div class="text-subtitle1">{{ $t('role.permissionDefs') }}</div>
          <q-space />
          <q-btn color="positive" icon="add" :label="$t('common.add')" @click="openPermissionDialog()" />
        </div>
        <q-table
          :rows="permissionDefinitions"
          :columns="permissionColumns"
          row-key="key"
          flat
          :loading="loadingPermissions"
          :pagination="permissionPagination"
          @request="onPermissionRequest"
        >
          <template v-slot:body-cell-module="props">
            <q-td :props="props">{{ getModuleLabel(props.value) }}</q-td>
          </template>
          <template v-slot:body-cell-category="props">
            <q-td :props="props"><q-chip :color="getCategoryColor(props.value)" text-color="white" dense>{{ getCategoryLabel(props.value) }}</q-chip></q-td>
          </template>
          <template v-slot:body-cell-sensitive="props">
            <q-td :props="props">
              <q-icon
                :name="props.value === true ? 'warning' : 'check_circle'"
                :color="props.value === true ? 'warning' : 'positive'"
              />
            </q-td>
          </template>
          <template v-slot:body-cell-actions="props">
            <q-td :props="props">
              <q-btn flat dense icon="edit" color="primary" size="sm" @click="openPermissionDialog(props.row)" />
              <q-btn flat dense icon="delete" color="negative" size="sm" @click="confirmDeletePermission(props.row)" />
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>

    <!-- 权限项编辑弹窗 -->
    <q-dialog v-model="showPermissionDialog" persistent>
      <q-card style="min-width: 400px">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ editingPermission ? $t('role.editPermission') : $t('role.addPermission') }}</div>
          <q-space />
          <q-btn flat round icon="close" @click="showPermissionDialog = false" />
        </q-card-section>
        <q-separator />
        <q-card-section>
          <q-form class="q-gutter-md">
            <q-input v-model="permissionForm.name" :label="$t('role.permissionName')" outlined />
            <q-select v-model="permissionForm.module" :options="moduleOptions" :label="$t('role.module')" outlined emit-value map-options />
            <q-select v-model="permissionForm.category" :options="categoryOptions" :label="$t('role.category')" outlined emit-value map-options />
            <q-input v-model="permissionForm.description" :label="$t('common.description')" outlined type="textarea" />
            <q-checkbox v-model="permissionForm.sensitive" :label="$t('role.isSensitive')" />
          </q-form>
        </q-card-section>
        <q-separator />
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn color="primary" :label="$t('common.save')" :loading="savingPermission" @click="handleSavePermission" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 删除确认弹窗 -->
    <q-dialog v-model="showDeleteDialog" persistent>
      <q-card>
        <q-card-section class="row items-center">
          <div class="text-h6">{{ $t('common.confirm') }}</div>
        </q-card-section>
        <q-card-section>{{ $t('role.deletePermissionConfirm') }}</q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn color="negative" :label="$t('common.delete')" @click="doDeletePermission" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { useRolePermissionPage } from '@erp-new-frontend-monorepo/composables/src/useRolePermissionPage';

const {
  selectedRole, saving, permissionGroups, permissionDefinitions, loadingPermissions,
  permissionPagination, showPermissionDialog, editingPermission, permissionForm, savingPermission,
  showDeleteDialog, roleOptions, hasChanges, is_read_only, permissionColumns,
  moduleOptions, categoryOptions,
  loadRolePermissions, getSelectedCount, toggleGroup, resetChanges, savePermissions,
  onPermissionRequest, getModuleLabel, getCategoryColor, getCategoryLabel,
  openPermissionDialog, handleSavePermission, confirmDeletePermission, doDeletePermission,
} = useRolePermissionPage();
</script>

<style scoped>
.permission-group { margin-bottom: 8px; }
.permission-group :deep(.q-expansion-item__container) { border-radius: 8px; }
.permission-checkbox { width: 100%; }
</style>
