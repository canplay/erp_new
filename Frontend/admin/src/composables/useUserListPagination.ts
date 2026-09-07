/**
 * @file composables/useUserListPagination.ts
 * @description UserListPage - 分页和筛选状态管理
 * @date 2026-08-22
 */

import { ref, computed } from 'vue';
import type { User } from '@/api/user';

export function useUserListPagination() {
  const filters = ref({
    keyword: '',
    start_date: '',
    end_date: '',
    status: null as number | null,
    role: null as string | null,
  });

  const pagination = ref({
    page: 1,
    rowsPerPage: 10,
    rowsNumber: 0,
    sortBy: 'id',
    descending: true,
  });

  const hasFilters = computed(() => {
    return !!(
      filters.value.keyword ||
      filters.value.start_date ||
      filters.value.end_date ||
      filters.value.status !== null ||
      filters.value.role
    );
  });

  function resetFilters() {
    filters.value = {
      keyword: '',
      start_date: '',
      end_date: '',
      status: null,
      role: null,
    };
    pagination.value.page = 1;
  }

  function resetPagination() {
    pagination.value.page = 1;
    pagination.value.rowsPerPage = 10;
    pagination.value.sortBy = 'id';
    pagination.value.descending = true;
  }

  function buildQueryParams(): Record<string, unknown> {
    const params: Record<string, unknown> = {
      page: pagination.value.page,
      page_size: pagination.value.rowsPerPage,
    };

    if (filters.value.keyword) params.keyword = filters.value.keyword;
    if (filters.value.status !== null && filters.value.status !== undefined) {
      params.status = filters.value.status;
    }
    if (filters.value.role) params.role = filters.value.role;

    return params;
  }

  return {
    filters,
    pagination,
    hasFilters,
    resetFilters,
    resetPagination,
    buildQueryParams,
  };
}
