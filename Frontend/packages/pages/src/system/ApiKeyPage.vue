<template>
  <q-page class="q-pa-md api-key-page">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg text-weight-bold">{{ $t('apiKey.title') }}</div>

    <!-- 工具栏 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section class="row items-center q-col-gutter-md">
        <div class="col-12 col-sm-4">
          <q-input v-model="filters.keyword" dense outlined :placeholder="$t('common.keyword')" clearable @keyup.enter="handleSearch">
            <template v-slot:prepend>
              <q-icon name="search" />
            </template>
          </q-input>
        </div>
        <div class="col-12 col-sm-2">
          <q-select v-model="filters.status" :options="statusOptions" :label="$t('apiKey.status')" dense outlined clearable emit-value map-options />
        </div>
        <q-space />
        <div class="col-auto">
          <q-btn color="positive" icon="add" :label="$t('apiKey.create')" @click="openCreateDialog" />
        </div>
      </q-card-section>
    </q-card>

    <!-- API 密钥列表 -->
    <q-card bordered>
      <q-table :rows="apiKeys" :columns="columns" row-key="id" flat :loading="loading" :pagination="pagination" @request="onTableRequest">
        <!-- 名称列 -->
        <template v-slot:body-cell-name="props">
          <q-td :props="props">
            <div class="text-weight-medium">{{ props.row.name }}</div>
            <div class="text-caption text-grey">{{ props.row.description }}</div>
          </q-td>
        </template>

        <!-- 密钥列 -->
        <template v-slot:body-cell-key="props">
          <q-td :props="props">
            <div class="row items-center">
              <code class="api-key-display">{{ props.row.showKey ? props.row.key : '••••••••' + props.row.key.slice(-4) }}</code>
              <q-btn flat dense :icon="props.row.showKey ? 'visibility_off' : 'visibility'" size="sm" @click="props.row.showKey = !props.row.showKey">
                <q-tooltip>{{ props.row.showKey ? '隐藏' : '显示' }}</q-tooltip>
              </q-btn>
              <q-btn flat dense icon="content_copy" size="sm" @click="handleCopyKey(props.row.key)">
                <q-tooltip>复制</q-tooltip>
              </q-btn>
            </div>
          </q-td>
        </template>

        <!-- 权限列 -->
        <template v-slot:body-cell-permissions="props">
          <q-td :props="props">
            <q-chip v-for="perm in props.row.permissions.slice(0, 2)" :key="perm" dense size="sm" color="blue-grey-2" text-color="dark">
              {{ perm }}
            </q-chip>
            <q-chip v-if="props.row.permissions.length > 2" dense size="sm" color="grey" text-color="white">
              +{{ props.row.permissions.length - 2 }}
            </q-chip>
          </q-td>
        </template>

        <!-- 状态列 -->
        <template v-slot:body-cell-status="props">
          <q-td :props="props">
            <q-badge :color="props.row.status === 'active' ? 'positive' : 'grey'" :label="props.row.status === 'active' ? '活跃' : '已禁用'" />
          </q-td>
        </template>

        <!-- 用量列 -->
        <template v-slot:body-cell-usage="props">
          <q-td :props="props">
            <div class="text-center">
              <div class="text-caption">{{ formatNumber(props.row.usageCount || 0) }}</div>
              <q-linear-progress :value="(props.row.usagePercent || 0) / 100" color="primary" size="sm" class="q-mt-xs" />
            </div>
          </q-td>
        </template>

        <!-- 最后使用时间列 -->
        <template v-slot:body-cell-lastUsedAt="props">
          <q-td :props="props">
            {{ props.row.lastUsedAt ? new Date(props.row.lastUsedAt).toLocaleString() : '从未使用' }}
          </q-td>
        </template>

        <!-- 操作列 -->
        <template v-slot:body-cell-actions="props">
          <q-td :props="props">
            <q-btn flat dense icon="edit" size="sm" color="primary" @click="openEditDialog(props.row)">
              <q-tooltip>{{ $t('common.edit') }}</q-tooltip>
            </q-btn>
            <q-btn flat dense :icon="props.row.status === 'active' ? 'pause' : 'play_arrow'" size="sm" :color="props.row.status === 'active' ? 'warning' : 'positive'" @click="handleToggleStatus(props.row)">
              <q-tooltip>{{ props.row.status === 'active' ? '禁用' : '启用' }}</q-tooltip>
            </q-btn>
            <q-btn flat dense icon="delete" size="sm" color="negative" @click="handleDelete(props.row)">
              <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
            </q-btn>
          </q-td>
        </template>
      </q-table>
    </q-card>

    <!-- 创建/编辑对话框 -->
    <q-dialog v-model="showDialog" persistent>
      <q-card style="min-width: 600px">
        <q-card-section>
          <div class="text-h6">{{ isEdit ? $t('apiKey.edit') : $t('apiKey.create') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section>
          <q-form @submit="handleSave" class="q-gutter-md">
            <div class="row q-col-gutter-md">
              <div class="col-12">
                <q-input v-model="form.name" :label="$t('apiKey.name')" outlined :rules="[(val: string) => !!val || '请输入名称']" />
              </div>
              <div class="col-12">
                <q-input v-model="form.description" :label="$t('apiKey.description')" outlined type="textarea" rows="2" />
              </div>
              <div class="col-6">
                <q-select v-model="form.permissions" :options="permissionOptions" :label="$t('apiKey.permissions')" outlined emit-value map-options multiple />
              </div>
              <div class="col-6">
                <q-input v-model.number="form.rateLimit" :label="$t('apiKey.rateLimit')" outlined type="number" min="1" max="1000" />
              </div>
              <div class="col-6">
                <q-input v-model="form.expires_at" :label="$t('apiKey.expires_at')" outlined type="datetime-local" />
              </div>
              <div class="col-6">
                <q-input v-model="form.ipWhitelist" :label="$t('apiKey.ipWhitelist')" outlined placeholder="用逗号分隔,留空表示不限制" />
              </div>
            </div>

            <div class="text-caption text-grey">{{ $t('apiKey.securityHint') }}</div>
          </q-form>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" @click="showDialog = false" />
          <q-btn color="primary" :label="$t('common.save')" type="submit" @click="handleSave" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 查看密钥对话框 -->
    <q-dialog v-model="showKeyDialog">
      <q-card style="min-width: 500px">
        <q-card-section>
          <div class="text-h6 text-negative">{{ $t('apiKey.saveKeyNotice') }}</div>
        </q-card-section>

        <q-separator />

        <q-card-section>
          <div class="text-subtitle2 q-mb-sm">{{ $t('apiKey.newKey') }}</div>
          <div class="key-display">
            <code>{{ newCreatedKey }}</code>
            <q-btn flat dense icon="content_copy" @click="handleCopyKey(newCreatedKey)" />
          </div>
          <q-banner class="q-mt-md bg-warning-1" rounded>
            <template v-slot:avatar>
              <q-icon name="warning" color="warning" />
            </template>
            {{ $t('apiKey.keyWarning') }}
          </q-banner>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn color="primary" :label="$t('common.done')" v-close-popup />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file ApiKeyPage.vue
 * @description API 密钥管理页面
 */

import { useApiKey } from '@erp-new-frontend-monorepo/composables/src/useApiKey';

const {
  loading,
  apiKeys,
  pagination,
  showDialog,
  showKeyDialog,
  isEdit,
  newCreatedKey,
  filters,
  form,
  statusOptions,
  permissionOptions,
  columns,
  formatNumber,
  onTableRequest,
  handleSearch,
  openCreateDialog,
  openEditDialog,
  handleSave,
  handleToggleStatus,
  handleDelete,
  handleCopyKey,
} = useApiKey();
</script>

<style scoped>
.api-key-page {
  min-height: 100vh;
}

.api-key-display {
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 12px;
  background: #f5f5f5;
  padding: 4px 8px;
  border-radius: 4px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.key-display {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #fff5f5;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #ffcdd2;
}

.key-display code {
  flex: 1;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 14px;
  word-break: break-all;
}
</style>
