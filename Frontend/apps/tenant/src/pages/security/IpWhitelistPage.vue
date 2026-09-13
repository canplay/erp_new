<template>
  <q-page class="q-pa-md">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg">{{ $t('ipWhitelist.title') }}</div>

    <!-- 操作栏 -->
    <div class="row q-mb-md items-center">
      <div class="col">
        <q-input
          v-model="searchKeyword"
          :placeholder="$t('common.search') + '...'"
          dense
          outlined
          clearable
          style="max-width: 300px"
          @update:model-value="loadRules"
        >
          <template #prepend>
            <q-icon name="search" />
          </template>
        </q-input>
      </div>
      <div class="row q-gutter-sm">
        <q-btn color="positive" :icon="matAdd" :label="$t('common.add')" @click="openCreateDialog" />
        <q-btn flat :icon="matRefresh" :label="$t('common.refresh')" @click="loadRules" />
      </div>
    </div>

    <!-- 规则列表 -->
    <q-card flat bordered>
      <q-table
        :rows="ruleList"
        :columns="columns"
        row-key="id"
        :loading="loading"
        flat
        :pagination="{ rowsPerPage: 15 }"
        @request="onRequest"
      >
        <!-- IP类型 -->
        <template #body-cell-ipType="{ row }">
          <q-td>
            <q-badge :color="getIpTypeColor(row.ipType)" :label="getIpTypeLabel(row.ipType)" />
          </q-td>
        </template>

        <!-- IP范围 -->
        <template #body-cell-ipRange="{ row }">
          <q-td>
            <code class="text-body2">{{ formatIpRange(row) }}</code>
          </q-td>
        </template>

        <!-- 生效对象 -->
        <template #body-cell-targetType="{ row }">
          <q-td>
            <q-badge :label="getTargetTypeLabel(row.targetType)" />
          </q-td>
        </template>

        <!-- 生效时间 -->
        <template #body-cell-effectTime="{ row }">
          <q-td>
            <div v-if="row.effectStartTime || row.effectEndTime" class="text-body2">
              {{ row.effectStartTime || '-' }} ~ {{ row.effectEndTime || '-' }}
            </div>
            <span v-else class="text-grey-5">{{ $t('ipWhitelist.permanent') }}</span>
          </q-td>
        </template>

        <!-- 状态 -->
        <template #body-cell-status="{ row }">
          <q-td>
            <q-toggle
              :model-value="row.status === 1"
              color="positive"
              @update:model-value="toggleStatus(row)"
            />
          </q-td>
        </template>

        <!-- 优先级 -->
        <template #body-cell-priority="{ row }">
          <q-td>
            <q-chip dense size="sm" color="primary" text-color="white">
              {{ row.priority }}
            </q-chip>
          </q-td>
        </template>

        <!-- 操作 -->
        <template #body-cell-actions="{ row }">
          <q-btn flat dense round :icon="matEdit" @click="openEditDialog(row)">
            <q-tooltip>{{ $t('common.edit') }}</q-tooltip>
          </q-btn>
          <q-btn flat dense round :icon="matDelete" color="negative" @click="handleDelete(row)">
            <q-tooltip>{{ $t('common.delete') }}</q-tooltip>
          </q-btn>
        </template>

        <!-- 加载状态 -->
        <template #loading>
          <q-inner-loading showing color="primary" />
        </template>

        <!-- 空状态 -->
        <template #no-data>
          <div class="full-width row flex-center text-grey-6 q-pa-lg">
            <q-icon name="vpn_lock" size="48px" class="q-mb-sm" />
            <div>{{ $t('ipWhitelist.noRules') }}</div>
          </div>
        </template>
      </q-table>
    </q-card>

    <!-- 创建/编辑对话框 -->
    <q-dialog v-model="dialogVisible" persistent>
      <q-card style="min-width: 500px; max-width: 600px">
        <q-card-section>
          <div class="text-h6">
            {{ isEdit ? $t('ipWhitelist.editRule') : $t('ipWhitelist.createRule') }}
          </div>
        </q-card-section>

        <q-separator />

        <q-card-section>
          <q-form @submit="handleSubmit" class="q-gutter-md">
            <!-- 规则名称 -->
            <q-input
              v-model="formData.name"
              :label="$t('ipWhitelist.ruleName')"
              outlined
              dense
              :rules="[(val) => !!val || $t('common.required')]"
            />

            <!-- IP类型 -->
            <q-select
              v-model="formData.ipType"
              :options="ipTypeOptions"
              :label="$t('ipWhitelist.ipType')"
              outlined
              dense
              emit-value
              map-options
              @update:model-value="onIpTypeChange"
            />

            <!-- IP地址/CIDR -->
            <q-input
              v-model="formData.ipStart"
              :label="ipStartLabel"
              outlined
              dense
              :placeholder="ipStartPlaceholder"
              :rules="[(val) => !!val || $t('common.required')]"
            />

            <!-- 结束IP（IP段类型） -->
            <q-input
              v-if="formData.ipType === 'range'"
              v-model="formData.ipEnd"
              :label="$t('ipWhitelist.ipEnd')"
              outlined
              dense
              placeholder="例：192.0.2.255"
            />

            <!-- 生效对象 -->
            <q-select
              v-model="formData.targetType"
              :options="targetTypeOptions"
              :label="$t('ipWhitelist.targetType')"
              outlined
              dense
              emit-value
              map-options
            />

            <!-- 生效时间 -->
            <div class="row q-gutter-md">
              <q-input
                v-model="formData.effectStartTime"
                :label="$t('ipWhitelist.effectStartTime')"
                outlined
                dense
                type="datetime-local"
                class="col"
              />
              <q-input
                v-model="formData.effectEndTime"
                :label="$t('ipWhitelist.effectEndTime')"
                outlined
                dense
                type="datetime-local"
                class="col"
              />
            </div>

            <!-- 优先级 -->
            <q-input
              v-model.number="formData.priority"
              :label="$t('ipWhitelist.priority')"
              outlined
              dense
              type="number"
              :hint="$t('common.lowerNumberHigherPriority')"
            />

            <!-- 状态 -->
            <q-toggle
              v-model="formData.status"
              :label="$t('ipWhitelist.status')"
              :true-value="1"
              :false-value="0"
            />

            <!-- 描述 -->
            <q-input
              v-model="formData.description"
              :label="$t('ipWhitelist.description')"
              outlined
              dense
              type="textarea"
              rows="3"
            />
          </q-form>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn
            color="primary"
            :label="$t('common.confirm')"
            :loading="submitting"
            type="submit"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 删除确认对话框 -->
    <ConfirmDialog
      ref="deleteDialogRef"
      :title="$t('common.confirmDelete')"
      :message="$t('ipWhitelist.deleteConfirmMessage')"
      icon="delete"
      confirm-color="negative"
      @confirm="confirmDelete"
    />
  </q-page>
</template>

<script setup lang="ts">
/**
 * @file IpWhitelistPage.vue
 * @description IP白名单管理页面
 * @date 2026-04-06
 */

import { onMounted } from 'vue';
import { useIpWhitelist } from '@erp-new-frontend-monorepo/composables/src/useIpWhitelist';;
import ConfirmDialog from '@erp-new-frontend-monorepo/components/src/ConfirmDialog.vue';
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';

// Material Icons
const matAdd = 'add';
const matEdit = 'edit';
const matDelete = 'delete';
const matRefresh = 'refresh';

const { t: $t } = useI18n();

const {
  loading, submitting, ruleList, searchKeyword,
  dialogVisible, isEdit, formData,
  ipTypeOptions, targetTypeOptions, ipStartLabel, ipStartPlaceholder, columns,
  getIpTypeColor, getIpTypeLabel, getTargetTypeLabel, formatIpRange,
  onIpTypeChange, loadRules, onRequest,
  openCreateDialog, openEditDialog, handleSubmit, deleteRule, toggleStatus,
} = useIpWhitelist();

const deleteDialogRef = ref<InstanceType<typeof ConfirmDialog> | null>(null);
const deleteTargetId = ref<number | null>(null);

function handleDelete(row: { id: number }) {
  deleteTargetId.value = row.id;
  deleteDialogRef.value?.open();
}

async function confirmDelete(): Promise<void> {
  if (deleteTargetId.value === null) return;
  try {
    await deleteRule(deleteTargetId.value);
  } catch (error) {
    logger.error('【删除确认失败】', error);
  }
}

// 生命周期
onMounted(() => {
  void loadRules();
});
</script>
