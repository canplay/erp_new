/**
 * @file PermissionTree.vue
 * @description 权限树形分组组件 - 按模块分组展示权限
 * @date 2026-04-04
 * @refactored 2026-09-12 - Split into composables
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
import { watch } from 'vue';
import { PermissionModule } from '@/types/permission';
import { usePermissionTree } from './composables/usePermissionTree';

// ============ Props & Emits ============

const props = withDefaults(defineProps<{
  modelValue?: string[];
  disabled?: boolean;
  defaultExpanded?: boolean;
  excludeModules?: PermissionModule[];
  includeModules?: PermissionModule[];
}>(), {
  modelValue: () => [],
  disabled: false,
  defaultExpanded: false,
  excludeModules: () => [],
});

const emit = defineEmits<{
  'update:modelValue': [value: string[]];
}>();

// ============ Composable ============

const {
  selectedPermissions,
  searchQuery,
  filteredPermissionGroups,
  selectAll,
  indeterminate,
  selectedCount,
  totalCount,
  sensitiveSelectedCount,
  disabled,
  getGroupSelectedCount,
  getGroupMatchedCount,
  isGroupAllSelected,
  isGroupIndeterminate,
  toggleGroup,
  toggleSelectAll,
} = usePermissionTree({
  modelValue: props.modelValue,
  disabled: props.disabled,
  defaultExpanded: props.defaultExpanded,
  excludeModules: props.excludeModules,
  includeModules: props.includeModules,
});

// ============ Watchers ============

watch(selectedPermissions, (newVal) => {
  emit('update:modelValue', [...newVal]);
});

// ============ Exposed Methods ============

defineExpose({
  selectAll: () => {
    selectedPermissions.value = filteredPermissionGroups.value.flatMap((g) =>
      g.permissions.map((p) => p.key)
    );
  },
  clearAll: () => {
    selectedPermissions.value = [];
  },
  getSelected: () => [...selectedPermissions.value],
  expandAll: () => {
    filteredPermissionGroups.value.forEach((g) => (g.expanded = true));
  },
  collapseAll: () => {
    filteredPermissionGroups.value.forEach((g) => (g.expanded = false));
  },
  search: (query: string) => {
    searchQuery.value = query;
  },
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
