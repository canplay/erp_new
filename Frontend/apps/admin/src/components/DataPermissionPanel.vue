/**
 * @file DataPermissionPanel.vue
 * @description 数据权限配置面板
 * @date 2026-04-04
 * @features
 * - 按模块配置数据权限范围
 * - 支持自定义数据范围
 * - 支持部门和租户隔离配置
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
          @click="saveDataPermissions"
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
/**
 * @file DataPermissionPanel.vue
 * @description 数据权限配置面板组件
 */

import { ref, computed, watch, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import { listDepartments } from '@/api/department';
import {
  PermissionScope,
  PermissionModule,
  type DataPermission,
  type PermissionDefinition,
  PERMISSIONS,
  getAllModules,
} from '@/types/permission';
import { updateRoleDataPermissions } from '@/api/permission';

const $q = useQuasar();
const { t: $t } = useI18n();

// ============ Props & Emits ============

const props = defineProps<{
  /** 角色名称 */
  role_name: string;
  /** 初始数据权限配置 */
  initialDataPermissions?: DataPermission[];
  /** 是否禁用 */
  disabled?: boolean;
}>();

const emit = defineEmits<{
  /** 保存成功 */
  saved: [dataPermissions: DataPermission[]];
  /** 变更 */
  changed: [dataPermissions: DataPermission[]];
}>();

// ============ 类型定义 ============

interface PermissionDataScope extends PermissionDefinition {
  scope: PermissionScope;
  custom_scope?: number[];
}

interface DepartmentNode {
  id: number;
  name: string;
  children?: DepartmentNode[];
}

// ============ 状态 ============

/** 权限数据范围列表 */
const permissionDataScopes = ref<PermissionDataScope[]>([]);

/** 隔离维度：department | creator | both */
const isolationDimension = ref<'department' | 'creator' | 'both'>('department');

/** 是否正在保存 */
const saving = ref(false);

/** 部门选择对话框 */
const showDepartmentDialog = ref(false);
const selectedDepartmentId = ref<number | null>(null);
const tickedDepartments = ref<number[]>([]);
const currentEditingItem = ref<PermissionDataScope | null>(null);

/** 部门树 */
const departmentTree = ref<DepartmentNode[]>([]);

/** 选中的部门（展示用） */
const selectedDepartments = ref<Array<{ id: number; name: string }>>([]);

// ============ 常量 ============

/** 权限范围选项 */
const scopeOptions = [
  { label: '全部数据', value: PermissionScope.ALL },
  { label: '本部门', value: PermissionScope.DEPARTMENT },
  { label: '本部门及下级', value: PermissionScope.DEPARTMENT_AND_CHILDREN },
  { label: '仅本人', value: PermissionScope.SELF },
  { label: '自定义', value: PermissionScope.CUSTOM },
];

/** 隔离维度选项 */
const isolationOptions = [
  { label: '按部门隔离', value: 'department' },
  { label: '按创建者隔离', value: 'creator' },
  { label: '两者结合', value: 'both' },
];

// ============ 计算属性 ============

/** 是否有自定义范围配置 */
const hasCustomScope = computed(() => {
  return permissionDataScopes.value.some((item) => item.scope === PermissionScope.CUSTOM);
});

// ============ 方法 ============

/**
 * @brief 获取模块图标
 */
function getModuleIcon(module: string | PermissionModule | undefined): string {
  const icons: Record<string, string> = {
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
  const moduleStr = String(module ?? '');
  return icons[moduleStr] || 'security';
}

/**
 * @brief 初始化权限数据范围
 */
function initPermissionDataScopes() {
  const modules = getAllModules();
  const initialPermMap = new Map(
    props.initialDataPermissions?.map((dp) => [dp.permission, dp]) || []
  );

  permissionDataScopes.value = modules
    .filter((m) => {
      // 只显示需要数据权限控制的模块
      const modulePerms = PERMISSIONS.filter((p) => p.module === m.value);
      return modulePerms.length > 0;
    })
    .flatMap((m) => {
      const modulePerms = PERMISSIONS.filter((p) => p.module === m.value);
      return modulePerms.map((p) => {
          const initial = initialPermMap.get(p.key);
          return {
            ...p,
            scope: initial?.scope || PermissionScope.ALL,
            ...(initial?.custom_scope ? { custom_scope: initial.custom_scope } : {}),
          };
        });
    });
}

/**
 * @brief 加载部门树（实际从API获取）
 */
async function loadDepartmentTree() {
  try {
    // 从 API 获取部门列表
    const response = await listDepartments({ page: 1, page_size: 100 });
    if (response.data?.data) {
      // 类型断言：假设 API 返回部门数组
      const departments = response.data.data as Array<{ id: number; name: string; children?: Array<{ id: number; name: string }> }>;
      departmentTree.value = departments.map((dept) => ({
        id: dept.id,
        name: dept.name,
        children: dept.children || [],
      }));
      logger.info('【部门树加载成功】', { count: departments.length });
    }
  } catch (error) {
    // API 未实现时使用默认数据（降级处理）
    logger.warn('【部门 API 未实现，使用默认数据】', error);
    departmentTree.value = [
      {
        id: 1,
        name: '总公司',
        children: [
          { id: 2, name: '技术部' },
          { id: 3, name: '运营部' },
          { id: 4, name: '市场部' },
        ],
      },
    ];
  }
}

// 组件挂载时加载部门树
onMounted(() => {
  void loadDepartmentTree();
});

/**
 * @brief 打开部门选择器
 */
function openDepartmentSelector(item: PermissionDataScope) {
  currentEditingItem.value = item;
  tickedDepartments.value = item.custom_scope || [];
  showDepartmentDialog.value = true;
}

/**
 * @brief 确认部门选择
 */
function confirmDepartmentSelection() {
  if (currentEditingItem.value) {
    currentEditingItem.value.custom_scope = [...tickedDepartments.value];
    // 更新展示用的部门列表
    selectedDepartments.value = tickedDepartments.value.map((id) => ({
      id,
      name: `部门${id}`,
    }));
  }
  showDepartmentDialog.value = false;
}

/**
 * @brief 移除部门
 */
function removeDepartment(deptId: number) {
  tickedDepartments.value = tickedDepartments.value.filter((id) => id !== deptId);
  if (currentEditingItem.value) {
    currentEditingItem.value.custom_scope = [...tickedDepartments.value];
  }
  selectedDepartments.value = selectedDepartments.value.filter((d) => d.id !== deptId);
}

/**
 * @brief 处理范围变更
 */
function handleScopeChange(item: PermissionDataScope, newScope: PermissionScope) {
  item.scope = newScope;
  if (newScope !== PermissionScope.CUSTOM) {
    delete item.custom_scope;
  }
  emit('changed', getDataPermissions());
}

/**
 * @brief 获取数据权限配置
 */
function getDataPermissions(): DataPermission[] {
  return permissionDataScopes.value
    .filter((item) => item.scope !== PermissionScope.ALL) // 只保存非全选的
    .map((item) => ({
      permission: item.key,
      scope: item.scope,
      ...(item.custom_scope ? { custom_scope: item.custom_scope } : {}),
    }));
}

/**
 * @brief 重置为默认
 */
function resetToDefault() {
  permissionDataScopes.value.forEach((item) => {
    item.scope = PermissionScope.ALL;
    delete item.custom_scope;
  });
  tickedDepartments.value = [];
  selectedDepartments.value = [];
  emit('changed', getDataPermissions());
}

/**
 * @brief 保存数据权限
 */
async function saveDataPermissions() {
  saving.value = true;
  try {
    const dataPermissions = getDataPermissions();
    await updateRoleDataPermissions(props.role_name, dataPermissions);
    $q.notify({
      type: 'positive',
      message: '数据权限保存成功',
    });
    emit('saved', dataPermissions);
  } catch (error) {
    logger.error('【保存数据权限失败】', error);
    $q.notify({
      type: 'negative',
      message: '保存失败，请重试',
    });
  } finally {
    saving.value = false;
  }
}

// ============ 监听器 ============

watch(
  () => props.initialDataPermissions,
  () => {
    initPermissionDataScopes();
  },
  { immediate: true, deep: true }
);

// ============ 生命周期 ============

void loadDepartmentTree();
</script>

<style scoped>
.data-permission-panel {
  width: 100%;
}

.cursor-pointer {
  cursor: pointer;
}
</style>

