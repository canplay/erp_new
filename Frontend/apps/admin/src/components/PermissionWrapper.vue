<template>
  <!-- 有权限时显示插槽内容 -->
  <slot v-if="hasPermission"></slot>
  <!-- 可选：无权限时显示 fallback -->
  <slot v-else name="fallback"></slot>
</template>

<script setup lang="ts">
/**
 * @file PermissionWrapper.vue
 * @description 权限包装组件 - 根据权限显示/隐藏内容
 * @date 2026-04-04
 * @example
 *   <PermissionWrapper permission="user:delete">
 *     <q-btn label="删除" />
 *   </PermissionWrapper>
 *
 *   <PermissionWrapper :permission="['user:read', 'user:update']" mode="all">
 *     <q-btn label="编辑" />
 *   </PermissionWrapper>
 */

import { computed } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { usePermissionStore } from '@/stores/permission';

interface Props {
  /** 所需权限（单个权限或权限数组） */
  permission: string | string[];
  /**
   * 权限检查模式
   * - any: 拥有任意一个权限即可显示
   * - all: 必须拥有所有权限才显示
   */
  mode?: 'any' | 'all';
  /**
   * 是否显示无权限占位（替代完全隐藏）
   * - true: 渲染一个空的 span
   * - false: 完全不渲染
   */
  placeholder?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  mode: 'any',
  placeholder: false,
});

const authStore = useAuthStore();
const permissionStore = usePermissionStore();

/**
 * @brief 检查权限
 */
const hasPermission = computed(() => {
  // 管理员拥有所有权限
  if (authStore.isAdmin) {
    return true;
  }

  const permissions = Array.isArray(props.permission) ? props.permission : [props.permission];

  if (props.mode === 'all') {
    // 必须拥有所有权限
    return permissions.every((p) =>
      permissionStore.hasPermission(p, authStore.currentRole)
    );
  }
  // 拥有任意一个权限即可
  return permissions.some((p) =>
    permissionStore.hasPermission(p, authStore.currentRole)
  );
});
</script>
