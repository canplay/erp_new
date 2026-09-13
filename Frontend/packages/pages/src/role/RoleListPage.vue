<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-md text-weight-bold">{{ $t('role.title') }}</div>

    <!-- 高级搜索 -->
    <AdvancedSearch
      v-model="advancedFilters"
      :show-status="true"
      :keyword-placeholder="$t('role.searchPlaceholder') || $t('common.keyword')"
      :status-label="$t('common.status')"
      :field-options="[]"
      @search="handleAdvancedSearch"
      @reset="handleSearchReset"
    />

    <!-- 工具栏 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center justify-end">
        <q-btn color="positive" :label="$t('role.createRole')" icon="add" @click="openCreateDialog" />
      </q-card-section>
    </q-card>

    <!-- 角色列表 -->
    <q-card bordered>
      <q-table
        :rows="roles"
        :columns="columns"
        row-key="name"
        :loading="loading"
        flat
        bordered
        :pagination="pagination"
        @request="onTableRequest"
      >
        <!-- 状态列 -->
        <template v-slot:body-cell-status="props">
          <q-td :props="props">
            <q-chip
              :color="props.value === 1 ? 'positive' : 'negative'"
              text-color="white"
              dense
              :icon="props.value === 1 ? 'check_circle' : 'block'"
            >
              {{ props.value === 1 ? $t('dictionary.enabled') : $t('dictionary.disabled') }}
            </q-chip>
          </q-td>
        </template>

        <!-- 角色类型 -->
        <template v-slot:body-cell-type="props">
          <q-td :props="props">
            <q-chip
              :color="props.value === 'admin' ? 'primary' : props.value === 'vip' ? 'amber' : 'grey'"
              text-color="white"
              dense
            >
              {{ getRoleTypeLabel(props.value) }}
            </q-chip>
          </q-td>
        </template>

        <!-- 操作列 -->
        <template v-slot:body-cell-actions="props">
          <q-td :props="props" class="text-center">
            <q-btn
              flat
              dense
              color="primary"
              :label="$t('role.assignPermissions')"
              @click="openPermissionDialog(props.row)"
            />
            <q-btn
              v-if="!props.row.is_predefined"
              flat
              dense
              color="negative"
              :label="$t('common.delete')"
              :disable="props.row.is_predefined"
              @click="confirmDelete(props.row)"
            />
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
            <div>{{ message || $t('table.noData') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 删除确认对话框 -->
    <q-dialog v-model="showDelete" persistent>
      <q-card>
        <q-card-section>
          <div class="text-h6">{{ $t('common.confirmDelete') }}</div>
        </q-card-section>
        <q-card-section>
          <p>{{ $t('role.deleteConfirm', { name: deleteTarget?.name }) }}</p>
          <p v-if="deleteTarget?.user_count" class="text-negative">
            {{ $t('role.deleteWarning', { count: deleteTarget.user_count }) }}
          </p>
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showDelete = false" />
          <q-btn color="negative" :label="$t('common.confirm')" @click="doDelete" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 权限分配对话框 -->
    <q-dialog v-model="showPermissionDialog" persistent>
      <q-card style="min-width: 600px">
        <q-card-section>
          <div class="text-h6">{{ $t('role.assignPermissions') }} - {{ currentRole?.name }}</div>
        </q-card-section>
        <q-card-section>
          <div class="q-mb-md">
            <q-toggle v-model="selectAll" :label="$t('role.selectAll')" />
          </div>
          <div v-for="group in permissionGroups" :key="group.key" class="q-mb-md">
            <div class="text-subtitle1 text-weight-medium q-mb-sm">
              <q-icon :name="group.icon" class="q-mr-xs" />
              {{ group.label }}
            </div>
            <div class="row q-gutter-sm">
              <q-checkbox
                v-for="permission in group.permissions"
                :key="permission.value"
                v-model="selectedPermissions"
                :val="permission.value"
                :label="getPermissionLabel(permission.action, group.label)"
              />
            </div>
          </div>
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showPermissionDialog = false" />
          <q-btn color="primary" :label="$t('common.save')" :loading="savingPermissions" @click="savePermissions" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 创建角色对话框 -->
    <q-dialog v-model="showCreateDialog" persistent>
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">{{ $t('role.createRole') }}</div>
        </q-card-section>
        <q-card-section>
          <q-input
            v-model="createForm.name"
            :label="$t('role.role_name')"
            outlined
            class="q-mb-md"
          />
          <q-select
            v-model="createForm.type"
            :options="roleTypeOptions"
            :label="$t('role.roleType')"
            outlined
            emit-value
            map-options
            class="q-mb-md"
          />
          <q-input
            v-model="createForm.description"
            :label="$t('role.description')"
            outlined
            type="textarea"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showCreateDialog = false" />
          <q-btn color="primary" :label="$t('common.create')" :loading="creating" @click="handleCreate" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import AdvancedSearch from '@erp-new-frontend-monorepo/components/src/AdvancedSearch/Main.vue';
import { useRoleList } from '@erp-new-frontend-monorepo/composables/src/useRoleList';

const {
  loading, roles, advancedFilters, columns, pagination, selectAll,
  showDelete, deleteTarget,
  showPermissionDialog, currentRole, selectedPermissions, savingPermissions,
  permissionGroups, showCreateDialog, creating, createForm, roleTypeOptions,
  loadRoleTypeOptions, getPermissionLabel, getRoleTypeLabel,
  openPermissionDialog, savePermissions,
  loadRoles, onTableRequest, handleAdvancedSearch, handleSearchReset,
  confirmDelete, doDelete, openCreateDialog, handleCreate,
} = useRoleList();

onMounted(() => {
  void loadRoles();
  void loadRoleTypeOptions();
});
</script>
