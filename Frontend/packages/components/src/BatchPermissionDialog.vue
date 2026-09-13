/**
 * @file BatchPermissionDialog.vue
 * @description 批量权限分配对话框
 * @date 2026-04-04
 * @features
 * - 批量选择多个角色
 * - 选择要分配/移除/覆盖的权限
 * - 支持复制角色权限
 */

<template>
  <q-dialog ref="dialogRef" @hide="onDialogHide" persistent>
    <q-card style="min-width: 700px; max-width: 80vw">
      <q-card-section class="row items-center">
        <div class="text-h6">
          <q-icon name="admin_panel_settings" class="q-mr-sm" color="primary" />
          {{ title }}
        </div>
        <q-space />
        <q-btn icon="close" flat round dense v-close-popup />
      </q-card-section>

      <q-separator />

      <q-card-section class="q-pa-none">
        <q-tabs
          v-model="activeTab"
          dense
          align="left"
          active-color="primary"
          indicator-color="primary"
          narrow-indicator
        >
          <q-tab name="assign" :label="$t('common.batchAssign')" />
          <q-tab name="copy" :label="$t('common.copyPermissions')" />
        </q-tabs>

        <q-separator />

        <q-tab-panels v-model="activeTab" animated>
          <!-- ============ 批量分配面板 ============ -->
          <q-tab-panel name="assign" class="q-pa-md">
            <!-- 操作模式 -->
            <div class="q-mb-md">
              <div class="text-subtitle2 q-mb-sm">{{ $t('batchPermission.operationMode') }}</div>
              <q-option-group
                v-model="assignMode"
                :options="modeOptions"
                color="primary"
                inline
              />
              <div class="text-caption text-grey q-mt-xs">
                <span v-if="assignMode === 'add'">{{ $t('batchPermission.modeAddDesc') }}</span>
                <span v-else-if="assignMode === 'set'">{{ $t('batchPermission.modeSetDesc') }}</span>
                <span v-else>{{ $t('batchPermission.modeRemoveDesc') }}</span>
              </div>
            </div>

            <!-- 选择角色 -->
            <div class="q-mb-md">
              <div class="text-subtitle2 q-mb-sm">
                {{ $t('batchPermission.selectRole') }}
                <q-badge color="primary" :label="$t('batchPermission.selectedCount', { count: selectedRoles.length })" class="q-ml-sm" />
              </div>
              <q-select
                v-model="selectedRoles"
                :options="roleOptions"
                multiple
                filled
                dense
                use-chips
                emit-value
                map-options
                options-dense
              >
                <template v-slot:option="{ itemProps, opt, selected: isSelected, toggleOption }">
                  <q-item v-bind="itemProps" @click="toggleOption(opt)">
                    <q-item-section side>
                      <q-checkbox :model-value="isSelected" />
                    </q-item-section>
                    <q-item-section>
                      <q-item-label>{{ opt.label }}</q-item-label>
                      <q-item-label caption>{{ opt.description }}</q-item-label>
                    </q-item-section>
                  </q-item>
                </template>
              </q-select>
            </div>

            <!-- 选择权限 -->
            <div class="q-mb-md">
              <div class="text-subtitle2 q-mb-sm">
                {{ $t('batchPermission.selectPermission') }}
                <q-badge color="primary" :label="$t('batchPermission.selectedCount', { count: selectedPermissions.length })" class="q-ml-sm" />
                <q-btn
                  flat
                  dense
                  size="sm"
                  color="primary"
                  :label="selectAllMode"
                  class="q-ml-md"
                  @click="toggleSelectAllPermissions"
                />
              </div>
              <q-scroll-area style="height: 300px">
                <q-card flat bordered>
                  <q-card-section>
                    <div v-for="group in permissionGroups" :key="group.module" class="q-mb-md">
                      <div class="text-caption text-grey-7 q-mb-xs q-pa-xs bg-grey-2">
                        <q-icon :name="group.icon" class="q-mr-xs" />
                        {{ group.label }}
                      </div>
                      <q-option-group
                        v-model="selectedPermissions"
                        :options="group.permissionOptions"
                        type="checkbox"
                        color="primary"
                      />
                    </div>
                  </q-card-section>
                </q-card>
              </q-scroll-area>
            </div>
          </q-tab-panel>

          <!-- ============ 复制权限面板 ============ -->
          <q-tab-panel name="copy" class="q-pa-md">
            <!-- 源角色 -->
            <div class="q-mb-md">
              <div class="text-subtitle2 q-mb-sm">{{ $t('batchPermission.sourceRole') }}</div>
              <q-select
                v-model="sourceRole"
                :options="roleOptions"
                filled
                dense
                emit-value
                map-options
                options-dense
              >
                <template v-slot:option="{ itemProps, opt }">
                  <q-item v-bind="itemProps">
                    <q-item-section avatar>
                      <q-icon name="source" color="primary" />
                    </q-item-section>
                    <q-item-section>
                      <q-item-label>{{ opt.label }}</q-item-label>
                      <q-item-label caption>{{ opt.description }}</q-item-label>
                    </q-item-section>
                  </q-item>
                </template>
              </q-select>
            </div>

            <!-- 目标角色 -->
            <div class="q-mb-md">
              <div class="text-subtitle2 q-mb-sm">
                {{ $t('batchPermission.targetRole') }}
                <q-badge color="primary" :label="$t('batchPermission.selectedCount', { count: targetRoles.length })" class="q-ml-sm" />
              </div>
              <q-select
                v-model="targetRoles"
                :options="targetRoleOptions"
                multiple
                filled
                dense
                use-chips
                emit-value
                map-options
                options-dense
              >
                <template v-slot:option="{ itemProps, opt, selected: isSelected, toggleOption }">
                  <q-item v-bind="itemProps" @click="toggleOption(opt)">
                    <q-item-section side>
                      <q-checkbox :model-value="isSelected" />
                    </q-item-section>
                    <q-item-section>
                      <q-item-label>{{ opt.label }}</q-item-label>
                      <q-item-label caption>{{ opt.description }}</q-item-label>
                    </q-item-section>
                  </q-item>
                </template>
              </q-select>
            </div>

            <!-- 复制选项 -->
            <div>
              <div class="text-subtitle2 q-mb-sm">{{ $t('batchPermission.copyContent') }}</div>
              <q-checkbox v-model="includeDataPermission" :label="$t('batchPermission.includeDataPermission')" color="primary" />
              <q-checkbox v-model="includeFieldPermission" :label="$t('batchPermission.includeFieldPermission')" color="primary" />
            </div>
          </q-tab-panel>
        </q-tab-panels>
      </q-card-section>

      <q-separator />

      <!-- 预览 -->
      <q-card-section v-if="showPreview" class="bg-blue-1">
        <div class="text-subtitle2 q-mb-sm">
          <q-icon name="preview" class="q-mr-sm" />
          {{ $t('batchPermission.operationPreview') }}
        </div>
        <div class="text-body2">
          <template v-if="activeTab === 'assign'">
            将 <strong>{{ selectedRoles.length }}</strong> 个角色的权限
            <span v-if="assignMode === 'add'">{{ $t('common.add') }}</span>
            <span v-else-if="assignMode === 'set'">{{ $t('common.setTo') }}</span>
            <span v-else>{{ $t('common.remove') }}</span>
            <strong>{{ selectedPermissions.length }}</strong> 项
          </template>
          <template v-else>
            将角色 <strong>{{ sourceRole }}</strong> 的权限复制到
            <strong>{{ targetRoles.length }}</strong> 个目标角色
          </template>
        </div>
      </q-card-section>

      <q-card-actions align="right">
        <q-btn flat color="grey" :label="$t('common.cancel')" v-close-popup />
        <q-btn
          color="primary"
          :label="confirmText"
          :loading="loading"
          :disable="!canConfirm"
          @click="handleConfirm"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">import { useI18n } from 'vue-i18n';

/**
 * @file BatchPermissionDialog.vue
 * @description 批量权限分配对话框组件
 */

import { ref, computed, watch } from 'vue';
import { logger } from '@/utils/logger';
import { useDialogPluginComponent } from 'quasar';
import {
  PERMISSION_META,
  getAllModulesI18n,
  PermissionModule,
} from '@/types/permission';
import { batchAssignPermissions, copyRolePermissions } from '@/api/permission';

defineEmits([...useDialogPluginComponent.emits]);

const { t } = useI18n();
const { dialogRef, onDialogHide, onDialogOK } = useDialogPluginComponent();

// ============ Props ============

interface Props {
  title?: string;
  roles?: Array<{ label: string; value: string; description?: string }>;
}

const props = withDefaults(defineProps<Props>(), {
  title: t('batchPermission.defaultTitle'),
  roles: () => [],
});

// ============ 类型定义 ============

interface PermissionGroupOption {
  module: PermissionModule;
  label: string;
  icon: string;
  permissionOptions: Array<{
    label: string;
    value: string;
  }>;
}

// ============ 状态 ============

/** 当前标签页 */
const activeTab = ref<'assign' | 'copy'>('assign');

/** 批量分配模式 */
const assignMode = ref<'add' | 'set' | 'remove'>('add');

/** 选中的角色列表 */
const selectedRoles = ref<string[]>([]);

/** 选中的权限列表 */
const selectedPermissions = ref<string[]>([]);

/** 源角色（复制用） */
const sourceRole = ref<string>('');

/** 目标角色列表（复制用） */
const targetRoles = ref<string[]>([]);

/** 是否包含数据权限 */
const includeDataPermission = ref(true);

/** 是否包含字段权限 */
const includeFieldPermission = ref(true);

/** 加载状态 */
const loading = ref(false);

// ============ 常量 ============

/** 操作模式选项 */
const modeOptions = [
  { label: t('batchPermission.modeAdd'), value: 'add' },
  { label: t('batchPermission.modeSet'), value: 'set' },
  { label: t('batchPermission.modeRemove'), value: 'remove' },
];

// ============ 计算属性 ============

/** 角色选项 */
const roleOptions = computed(() => {
  return props.roles.length > 0
    ? props.roles
    : [
        { label: t('batchPermission.defaultRoleUser'), value: 'user', description: t('batchPermission.defaultRoleUserDesc') },
        { label: t('batchPermission.defaultRoleVip'), value: 'vip', description: t('batchPermission.defaultRoleVipDesc') },
        { label: t('batchPermission.defaultRoleAdmin'), value: 'admin', description: t('batchPermission.defaultRoleAdminDesc') },
      ];
});

/** 目标角色选项（排除源角色） */
const targetRoleOptions = computed(() => {
  return roleOptions.value.filter((r) => r.value !== sourceRole.value);
});

/** 权限分组 */
const permissionGroups = computed<PermissionGroupOption[]>(() => {
  const { i18nT } = useI18nT();
  const modules = getAllModulesI18n();
  return modules
    .filter((m) => {
      const modulePerms = PERMISSION_META.filter((p) => p.module === m.value);
      return modulePerms.length > 0;
    })
    .map((m) => {
      const modulePerms = PERMISSION_META.filter((p) => p.module === m.value);
      return {
        module: m.value,
        label: m.label,
        icon: getModuleIcon(m.value),
        permissionOptions: modulePerms.map((p) => ({
          label: `${i18nT(`common.perm.${p.key.replace(/:/g, '')}`, p.key)}${i18nT(`common.perm.${p.key.replace(/:/g, '')}Desc}`, '') ? ` - ${i18nT(`common.perm.${p.key.replace(/:/g, '')}Desc}`, '')}` : ''}`,
          value: p.key,
        })),
      };
    });
});

/** 全选/取消全选模式 */
const selectAllMode = computed(() => {
  const totalPerms = permissionGroups.value.reduce(
    (sum, g) => sum + g.permissionOptions.length,
    0
  );
  return selectedPermissions.value.length === totalPerms ? t('batchPermission.deselectAll') : t('batchPermission.selectAll');
});

/** 是否可以确认 */
const canConfirm = computed(() => {
  if (activeTab.value === 'assign') {
    return selectedRoles.value.length > 0 && selectedPermissions.value.length > 0;
  }
  return sourceRole.value && targetRoles.value.length > 0;
});

/** 确认按钮文本 */
const confirmText = computed(() => {
  if (activeTab.value === 'assign') {
    const actionKey = assignMode.value === 'add' ? 'actionAdd' : assignMode.value === 'set' ? 'actionSet' : 'actionRemove';
    return t('batchPermission.confirm', { action: t(`batchPermission.${actionKey}`) });
  }
  return t('batchPermission.confirmCopy');
});

/** 是否显示预览 */
const showPreview = computed(() => {
  if (activeTab.value === 'assign') {
    return selectedRoles.value.length > 0 && selectedPermissions.value.length > 0;
  }
  return sourceRole.value && targetRoles.value.length > 0;
});

// ============ 方法 ============

/**
 * @brief 获取模块图标
 */
function getModuleIcon(module: PermissionModule): string {
  const icons: Record<PermissionModule, string> = {
    [PermissionModule.USER]: 'people',
    [PermissionModule.ROLE]: 'admin_panel_settings',
    [PermissionModule.PERMISSION]: 'vpn_key',
    [PermissionModule.LOGIN_LOG]: 'login',
    [PermissionModule.OPERATION_LOG]: 'history',
    [PermissionModule.SYSTEM_CONFIG]: 'settings',
    [PermissionModule.DICTIONARY]: 'menu_book',
    [PermissionModule.NOTIFICATION]: 'notifications',
    [PermissionModule.ANNOUNCEMENT]: 'campaign',
    [PermissionModule.MONITOR]: 'monitor_heart',
    [PermissionModule.FILE]: 'folder',
    [PermissionModule.TASK]: 'schedule',
    [PermissionModule.IMPORT_EXPORT]: 'import_export',
    [PermissionModule.ORGANIZATION]: 'corporate_fare',
    [PermissionModule.AUDIT]: 'fact_check',
  };
  return icons[module] || 'security';
}

/**
 * @brief 切换全选/取消全选
 */
function toggleSelectAllPermissions() {
  const totalPerms = permissionGroups.value.flatMap((g) => g.permissionOptions.map((p) => p.value));
  if (selectedPermissions.value.length === totalPerms.length) {
    selectedPermissions.value = [];
  } else {
    selectedPermissions.value = totalPerms;
  }
}

/**
 * @brief 确认操作
 */
async function handleConfirm() {
  loading.value = true;
  try {
    if (activeTab.value === 'assign') {
      await batchAssignPermissions(selectedRoles.value, selectedPermissions.value, assignMode.value);
    } else {
      await copyRolePermissions(sourceRole.value, targetRoles.value, {
        includeDataPermission: includeDataPermission.value,
        includeFieldPermission: includeFieldPermission.value,
      });
    }

    onDialogOK({
      success: true,
      action: activeTab.value,
    });
  } catch (error) {
    logger.error('【批量权限操作失败】', error);
  } finally {
    loading.value = false;
  }
}

// ============ 监听器 ============

watch(sourceRole, (newVal) => {
  // 清除已选择的目标角色中与源角色相同的
  targetRoles.value = targetRoles.value.filter((r) => r !== newVal);
});
</script>

<style scoped>
.bg-grey-2 {
  background: rgba(0, 0, 0, 0.05);
}
</style>

