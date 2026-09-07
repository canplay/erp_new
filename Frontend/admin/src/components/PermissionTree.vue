/**
 * @file PermissionTree.vue
 * @description 权限树形分组组件 - 按模块分组展示权限
 * @date 2026-04-04
 * @features 2026-04-06 增强
 * - 按模块分组展示权限
 * - 支持全选/取消全选分组
 * - 显示权限描述和操作提示
 * - 敏感权限高亮标识
 * - 搜索过滤功能
 * - 默认折叠状态（可配置）
 */

<template>
  <div class="permission-tree">
    <!-- 全选控制 -->
    <div class="row items-center q-mb-md">
      <q-checkbox
        v-model="selectAll"
        :indeterminate="indeterminate"
        :label="$t('common.selectAll')"
        color="primary"
        @update:model-value="toggleSelectAll"
      />
      <q-space />
      <span class="text-caption text-grey">
        {{ selectedCount }} / {{ totalCount }}
      </span>
    </div>

    <!-- 搜索过滤 -->
    <div class="row q-mb-md">
      <q-input
        v-model="searchQuery"
        :placeholder="$t('common.search') + '...'"
        outlined
        dense
        clearable
        class="col-12"
      >
        <template #prepend>
          <q-icon name="search" />
        </template>
      </q-input>
    </div>

    <q-separator class="q-mb-md" />

    <!-- 权限分组列表 -->
    <q-expansion-item
      v-for="group in filteredPermissionGroups"
      :key="group.module"
      v-model="group.expanded"
      :label="group.label"
      icon="security"
      header-class="permission-group-header"
      :default-opened="defaultExpanded"
      class="permission-group-item"
    >
      <template v-slot:header>
        <q-item-section avatar>
          <q-icon :name="group.icon || 'security'" color="primary" />
        </q-item-section>
        <q-item-section>
          <q-item-label class="text-weight-bold">{{ group.label }}</q-item-label>
          <q-item-label caption>
            {{ getGroupSelectedCount(group) }} / {{ group.permissions.length }}
            <span v-if="searchQuery" class="q-ml-xs">
              ({{ getGroupMatchedCount(group) }} {{ $t('common.matched') }})
            </span>
          </q-item-label>
        </q-item-section>
        <q-item-section side>
          <q-checkbox
            :model-value="isGroupAllSelected(group)"
            :indeterminate="isGroupIndeterminate(group)"
            dense
            color="primary"
            @update:model-value="(val) => toggleGroup(group, val)"
            @click.stop
          />
        </q-item-section>
      </template>

      <q-card flat bordered class="q-ma-sm">
        <q-card-section class="q-py-sm">
          <!-- 权限列表 -->
          <div class="row q-col-gutter-sm">
            <div
              v-for="permission in group.permissions"
              :key="permission.key"
              class="col-12 col-sm-6"
            >
              <q-checkbox
                v-model="selectedPermissions"
                :val="permission.key"
                :label="permission.name"
                :disable="disabled"
                dense
                class="permission-checkbox"
              >
                <q-tooltip v-if="permission.description" :-delay="300">
                  <div class="text-body2">
                    <div class="text-weight-bold q-mb-xs">{{ permission.name }}</div>
                    <div>{{ permission.description }}</div>
                    <div v-if="permission.sensitive" class="text-negative q-mt-xs">
                      <q-icon name="warning" size="xs" />
                      {{ $t('permission.sensitive') }}
                    </div>
                  </div>
                </q-tooltip>
              </q-checkbox>
              <!-- 敏感权限标识 -->
              <q-badge
                v-if="permission.sensitive"
                color="warning"
                text-color="black"
                class="q-ml-sm"
                :label="$t('permission.sensitive')"
              />
            </div>
          </div>

          <!-- 无匹配结果 -->
          <div
            v-if="group.permissions.length === 0"
            class="text-center text-grey q-pa-md"
          >
            {{ $t('table.noData') }}
          </div>
        </q-card-section>
      </q-card>
    </q-expansion-item>

    <!-- 无搜索结果 -->
    <div
      v-if="filteredPermissionGroups.length === 0 && searchQuery"
      class="text-center text-grey q-pa-lg"
    >
      <q-icon name="search_off" size="48px" class="q-mb-sm" />
      <div>{{ $t('common.noResults') }}</div>
    </div>

    <!-- 敏感权限提示 -->
    <q-banner
      v-if="sensitiveSelectedCount > 0"
      class="q-mt-md"
      rounded
      banner-class="bg-warning text-black"
    >
      <template #avatar>
        <q-icon name="warning" color="black" />
      </template>
      <div class="text-body2">
        {{ $t('permission.sensitiveSelectedHint', { count: sensitiveSelectedCount }) }}
      </div>
    </q-banner>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  PERMISSIONS,
  getAllModules,
  getSensitivePermissions,
  PermissionModule,
  type PermissionDefinition,
} from '@/types/permission';

const { t: $t } = useI18n();

// ============ 类型定义 ============

/**
 * @brief 权限分组接口
 */
interface PermissionGroup {
  module: PermissionModule;
  label: string;
  icon?: string;
  expanded: boolean;
  permissions: PermissionDefinition[];
}

// ============ Props 定义 ============

const props = withDefaults(defineProps<{
  /** v-model 选中的权限列表 */
  modelValue?: string[];
  /** 是否禁用 */
  disabled?: boolean;
  /** 是否默认展开所有分组（默认折叠） */
  defaultExpanded?: boolean;
  /** 排除的模块（不显示） */
  excludeModules?: PermissionModule[];
  /** 仅显示的模块 */
  includeModules?: PermissionModule[];
}>(), {
  modelValue: () => [],
  disabled: false,
  defaultExpanded: false, // 默认为折叠
  excludeModules: () => [],
});

// ============ Emit 定义 ============

const emit = defineEmits<{
  /** 更新选中的权限 */
  'update:modelValue': [value: string[]];
}>();

// ============ 状态 ============

/** 选中的权限列表 */
const selectedPermissions = ref<string[]>(props.modelValue || []);

/** 权限分组列表 */
const permissionGroups = ref<PermissionGroup[]>([]);

/** 搜索关键词 */
const searchQuery = ref('');

// ============ 监听器 ============

watch(() => props.modelValue, (newVal) => {
  selectedPermissions.value = newVal || [];
}, { immediate: true });

watch(selectedPermissions, (newVal) => {
  emit('update:modelValue', [...newVal]);
});

// ============ 计算属性 ============

/**
 * @brief 过滤后的权限分组
 */
const filteredPermissionGroups = computed(() => {
  if (!searchQuery.value.trim()) {
    return permissionGroups.value;
  }

  const query = searchQuery.value.toLowerCase().trim();

  return permissionGroups.value.map((group) => {
    const filteredPermissions = group.permissions.filter((p) => {
      return (
        p.name.toLowerCase().includes(query) ||
        p.key.toLowerCase().includes(query) ||
        (p.description && p.description.toLowerCase().includes(query))
      );
    });

    return {
      ...group,
      permissions: filteredPermissions,
      // 如果有搜索结果，自动展开该分组
      expanded: filteredPermissions.length > 0,
    };
  }).filter((group) => group.permissions.length > 0);
});

/** 全选状态 */
const selectAll = computed({
  get: () => {
    if (totalCount.value === 0) return false;
    return selectedPermissions.value.length === totalCount.value;
  },
  set: () => {},
});

/** 不确定状态（部分选中） */
const indeterminate = computed(() => {
  const count = selectedPermissions.value.length;
  return count > 0 && count < totalCount.value;
});

/** 已选权限总数 */
const selectedCount = computed(() => selectedPermissions.value.length);

/** 权限总数 */
const totalCount = computed(() => {
  return permissionGroups.value.reduce((sum, group) => sum + group.permissions.length, 0);
});

/** 已选的敏感权限数量 */
const sensitiveSelectedCount = computed(() => {
  const sensitiveKeys = getSensitivePermissions().map((p) => p.key);
  return selectedPermissions.value.filter((key) => sensitiveKeys.includes(key)).length;
});

// ============ 方法 ============

/**
 * @brief 初始化权限分组
 */
function initPermissionGroups() {
  const modules = getAllModules();
  const excludeSet = new Set(props.excludeModules || []);
  const includeSet = props.includeModules ? new Set(props.includeModules) : null;

  permissionGroups.value = modules
    .filter((m) => {
      // 排除指定模块
      if (excludeSet.has(m.value)) return false;
      // 仅包含指定模块
      if (includeSet && !includeSet.has(m.value)) return false;
      // 只显示有权限的模块（使用 String() 转换避免枚举比较类型错误）
      const moduleValue = String(m.value);
      const modulePerms = PERMISSIONS.filter((p) => String(p.module) === moduleValue);
      return modulePerms.length > 0;
    })
    .map((m) => {
      const moduleValue = String(m.value);
      const modulePerms = PERMISSIONS.filter((p) => String(p.module) === moduleValue);
      return {
        module: m.value,
        label: m.label,
        icon: getModuleIcon(m.value),
        expanded: props.defaultExpanded ?? false, // 默认为折叠
        permissions: modulePerms,
      };
    });
}

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
 * @brief 获取分组选中的数量
 */
function getGroupSelectedCount(group: PermissionGroup): number {
  return group.permissions.filter((p) => selectedPermissions.value.includes(p.key)).length;
}

/**
 * @brief 获取分组匹配的数量（用于搜索）
 */
function getGroupMatchedCount(group: PermissionGroup): number {
  if (!searchQuery.value.trim()) return group.permissions.length;
  return group.permissions.length;
}

/**
 * @brief 检查分组是否全选
 */
function isGroupAllSelected(group: PermissionGroup): boolean {
  return group.permissions.every((p) => selectedPermissions.value.includes(p.key));
}

/**
 * @brief 检查分组是否部分选中
 */
function isGroupIndeterminate(group: PermissionGroup): boolean {
  const selected = getGroupSelectedCount(group);
  return selected > 0 && selected < group.permissions.length;
}

/**
 * @brief 切换分组全选状态
 */
function toggleGroup(group: PermissionGroup, selected: boolean) {
  if (selected) {
    // 添加分组所有权限
    group.permissions.forEach((p) => {
      if (!selectedPermissions.value.includes(p.key)) {
        selectedPermissions.value.push(p.key);
      }
    });
  } else {
    // 移除分组所有权限
    selectedPermissions.value = selectedPermissions.value.filter(
      (key) => !group.permissions.some((p) => p.key === key)
    );
  }
}

/**
 * @brief 切换全选状态
 */
function toggleSelectAll(selected: boolean) {
  if (selected) {
    // 全选所有权限
    selectedPermissions.value = permissionGroups.value.flatMap((g) =>
      g.permissions.map((p) => p.key)
    );
  } else {
    // 取消全选
    selectedPermissions.value = [];
  }
}

// ============ 初始化 ============

initPermissionGroups();

// ============ 暴露方法 ============

defineExpose({
  /** 选中所有 */
  selectAll: () => {
    selectedPermissions.value = permissionGroups.value.flatMap((g) =>
      g.permissions.map((p) => p.key)
    );
  },
  /** 取消所有选中 */
  clearAll: () => {
    selectedPermissions.value = [];
  },
  /** 获取选中的权限 */
  getSelected: () => [...selectedPermissions.value],
  /** 展开所有分组 */
  expandAll: () => {
    permissionGroups.value.forEach((g) => (g.expanded = true));
  },
  /** 折叠所有分组 */
  collapseAll: () => {
    permissionGroups.value.forEach((g) => (g.expanded = false));
  },
  /** 搜索 */
  search: (query: string) => {
    searchQuery.value = query;
  },
  /** 清除搜索 */
  clearSearch: () => {
    searchQuery.value = '';
  },
});
</script>

<style scoped>
.permission-tree {
  width: 100%;
}

.permission-group-item {
  margin-bottom: 8px;
  border-radius: 8px;
  overflow: hidden;
}

.permission-group-item :deep(.q-expansion-item__container) {
  border-radius: 8px;
}

.permission-group-header {
  background: rgba(0, 0, 0, 0.02);
  border-radius: 8px;
}

.body--dark .permission-group-header {
  background: rgba(255, 255, 255, 0.05);
}

.permission-checkbox {
  width: 100%;
}

/* 暗色主题适配 */
.body--dark .q-card {
  background: #252525;
}

.body--dark .q-expansion-item {
  background: #1e1e1e;
}

.body--dark .q-expansion-item__content {
  background: #1e1e1e;
}
</style>
