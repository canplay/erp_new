<template>
  <div class="permission-matrix">
    <!-- 页面标题和操作 -->
    <div class="row items-center q-mb-lg">
      <div class="text-h5 text-weight-bold">
        <q-icon name="grid_on" class="q-mr-sm" />
        {{ $t('permission.matrix') || '权限矩阵' }}
      </div>
      <q-space />
      <q-btn flat color="grey" icon="refresh" @click="handleRefresh" />
      <q-btn color="primary" icon="save" :label="$t('common.saveConfig')" @click="handleSave" />
    </div>

    <!-- 筛选和搜索 -->
    <div class="row q-col-gutter-md q-mb-md">
      <div class="col-12 col-sm-3">
        <q-select
          v-model="moduleFilter"
          :options="moduleOptions"
          outlined
          dense
          clearable
          emit-value
          map-options
          :placeholder="$t('common.filterByModule')"
        />
      </div>
      <div class="col-12 col-sm-3">
        <q-select
          v-model="categoryFilter"
          :options="categoryOptions"
          outlined
          dense
          clearable
          emit-value
          map-options
          :placeholder="$t('common.filterByType')"
        />
      </div>
      <div class="col-12 col-sm-4">
        <q-input
          v-model="searchQuery"
          outlined
          dense
          :placeholder="$t('common.searchPermission')"
        >
          <template v-slot:prepend>
            <q-icon name="search" />
          </template>
          <template v-slot:append v-if="searchQuery">
            <q-icon name="close" class="cursor-pointer" @click="searchQuery = ''" />
          </template>
        </q-input>
      </div>
      <div class="col-12 col-sm-2">
        <q-btn-toggle
          v-model="viewMode"
          toggle-color="primary"
          :options="[
            { label: '矩阵', value: 'matrix' },
            { label: '列表', value: 'list' },
          ]"
          unelevated
        />
      </div>
    </div>

    <!-- 矩阵视图 -->
    <q-card v-if="viewMode === 'matrix'" flat bordered>
      <q-card-section class="q-pa-none">
        <div class="matrix-container">
          <table class="matrix-table">
            <thead>
              <tr>
                <th class="matrix-header matrix-header--permission">权限</th>
                <th
                  v-for="role in roles"
                  :key="role"
                  class="matrix-header matrix-header--role"
                >
                  <div class="role-header">
                    <q-checkbox
                      :model-value="allPermissionsGranted[role]"
                      :indeterminate="somePermissionsGranted[role]"
                      dense
                      @update:model-value="(val) => handleRoleToggle(role, val)"
                    />
                    <span>{{ getRoleLabel(role) }}</span>
                  </div>
                </th>
              </tr>
            </thead>
            <tbody>
              <template v-for="(group, moduleName) in filteredPermissionGroups" :key="moduleName">
                <tr class="matrix-group-header">
                  <td :colspan="roles.length + 1" class="matrix-group-cell">
                    <q-icon name="folder" class="q-mr-sm" />
                    {{ getModuleLabel(String(moduleName)) }}
                    <span class="text-grey q-ml-sm">({{ group.length }})</span>
                  </td>
                </tr>
                <tr v-for="perm in group" :key="perm.key" class="matrix-row">
                  <td class="matrix-cell matrix-cell--permission">
                    <div class="permission-info">
                      <span class="permission-name">{{ perm.name }}</span>
                      <span class="permission-key text-grey">{{ perm.key }}</span>
                    </div>
                  </td>
                  <td
                    v-for="role in roles"
                    :key="role"
                    class="matrix-cell matrix-cell--checkbox"
                  >
                    <q-checkbox
                      :model-value="getCellValue(role, perm.key)"
                      dense
                      @update:model-value="(val) => handleCellChange(role, perm.key, val)"
                    />
                  </td>
                </tr>
              </template>
            </tbody>
          </table>
        </div>
      </q-card-section>
    </q-card>

    <!-- 列表视图 -->
    <q-card v-else flat bordered>
      <q-card-section class="q-pa-none">
        <q-table
          :rows="filteredPermissions"
          :columns="listColumns"
          row-key="key"
          flat
          :pagination="{ rowsPerPage: 15 }"
        >
          <!-- 权限信息 -->
          <template v-slot:body-cell-permission="props">
            <q-td :props="props">
              <div>
                <div class="text-weight-medium">{{ props.row.name }}</div>
                <div class="text-caption text-grey">{{ props.row.key }}</div>
              </div>
            </q-td>
          </template>

          <!-- 模块标签 -->
          <template v-slot:body-cell-module="props">
            <q-td :props="props">
              <q-badge color="secondary" :label="getModuleLabel(props.row.module)" />
            </q-td>
          </template>

          <!-- 角色权限 -->
          <template v-slot:body-cell-roles="props">
            <q-td :props="props">
              <div class="row q-gutter-xs justify-center">
                <q-badge
                  v-for="role in roles"
                  :key="role"
                  :color="getCellValue(role, props.row.key) ? 'positive' : 'grey-5'"
                  text-color="white"
                  :label="getCellValue(role, props.row.key) ? '✓' : '✗'"
                  class="role-badge"
                />
              </div>
            </q-td>
          </template>

          <!-- 操作 -->
          <template v-slot:body-cell-actions="props">
            <q-td :props="props">
              <q-btn flat dense color="primary" icon="edit" @click="handleEditPermission(props.row)" />
            </q-td>
          </template>
        </q-table>
      </q-card-section>
    </q-card>

    <!-- 权限详情对话框 -->
    <q-dialog v-model="showDetailDialog">
      <q-card style="min-width: 500px">
        <q-card-section>
          <div class="text-h6">权限详情</div>
        </q-card-section>

        <q-separator />

        <q-card-section v-if="selectedPermission">
          <q-list>
            <q-item>
              <q-item-section>权限标识</q-item-section>
              <q-item-section side class="text-weight-medium">
                {{ selectedPermission.key }}
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>权限名称</q-item-section>
              <q-item-section side>{{ selectedPermission.name }}</q-item-section>
            </q-item>
            <q-item>
              <q-item-section>所属模块</q-item-section>
              <q-item-section side>
                <q-badge color="secondary" :label="getModuleLabel(selectedPermission.module ?? '')" />
              </q-item-section>
            </q-item>
            <q-item>
              <q-item-section>权限类型</q-item-section>
              <q-item-section side>
                <q-badge color="info" :label="selectedPermission.category" />
              </q-item-section>
            </q-item>
            <q-item v-if="selectedPermission.description">
              <q-item-section>{{ $t('common.description') }}</q-item-section>
              <q-item-section side>{{ selectedPermission.description }}</q-item-section>
            </q-item>
          </q-list>

          <q-separator class="q-my-md" />

          <div class="text-subtitle2 q-mb-sm">角色授权情况</div>
          <div class="row q-gutter-sm">
            <q-chip
              v-for="role in roles"
              :key="role"
              :color="getCellValue(role, selectedPermission.key) ? 'positive' : 'grey-4'"
              text-color="white"
              :icon="getCellValue(role, selectedPermission.key) ? 'check' : 'close'"
            >
              {{ getRoleLabel(role) }}
            </q-chip>
          </div>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.close')" v-close-popup />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { usePermissionMatrix } from '@erp-new-frontend-monorepo/composables/src/usePermissionMatrix';;

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
const {
  moduleFilter, categoryFilter, searchQuery, viewMode,
  showDetailDialog, selectedPermission, roles,
  moduleOptions, categoryOptions, filteredPermissions, filteredPermissionGroups,
  allPermissionsGranted, somePermissionsGranted, listColumns,
  getModuleLabel, getRoleLabel, getCellValue, handleCellChange,
  handleRoleToggle, handleRefresh, handleSave, handleEditPermission,
} = usePermissionMatrix();
</script>

