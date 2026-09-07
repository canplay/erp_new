/**
 * @file composables/useUserListCore.ts
 * @description UserListPage - 核心状态管理（用户数据、加载状态、选择状态）
 * @date 2026-08-22
 */

import { ref } from 'vue';
import type { User } from '@/api/user';

export function useUserListCore() {
  const loading = ref(false);
  const users = ref<User[]>([]);
  const selectedUsers = ref<User[]>([]);

  return {
    loading,
    users,
    selectedUsers,
  };
}
