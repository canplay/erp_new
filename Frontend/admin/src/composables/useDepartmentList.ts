/**
 * @file useDepartmentList.ts
 * @description 部门管理列表页面业务逻辑 composable
 */

import { ref, reactive, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';
import {
  listDepartments,
  createDepartment,
  updateDepartment,
  deleteDepartment,
  listUsers,
  type Department,
  type User,
} from '@/api';

// ============ 类型定义 ============

export interface DepartmentTreeNode {
  id: number;
  name: string;
  parent_id?: number;
  description?: string;
  leader_name?: string;
  leader_id?: number;
  sort_order?: number;
  user_count?: number;
  children: DepartmentTreeNode[];
}

// ============ Composable ============

export function useDepartmentList() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  // ============ 状态 ============
  const loading = ref(false);
  const departments = ref<Department[]>([]);
  const users = ref<User[]>([]);
  const selectedDepartmentId = ref<number | null>(null);
  const showDepartmentDialog = ref(false);
  const isEdit = ref(false);
  const deleteDialogRef = ref<{ open: () => void } | null>(null);
  const pendingDeleteDepartment = ref<Department | null>(null);

  // 部门表单
  const departmentForm = reactive({
    name: '',
    parent_id: null as number | null,
    leader_id: null as number | null,
    description: '',
    sort_order: 0,
  });

  // ============ 方法 ============

  /**
   * @brief 获取部门完整路径
   */
  function getDepartmentPath(department_id: number): string {
    const parts: string[] = [];
    let current = departments.value.find((d: Department) => d.id === department_id);
    while (current) {
      parts.unshift(current.name);
      const parent_id = current.parent_id;
      current = parent_id ? departments.value.find((d: Department) => d.id === parent_id) : undefined;
    }
    return parts.join(' / ');
  }

  /**
   * @brief 获取父部门名称
   */
  function getParentName(parent_id: number): string {
    const parent = departments.value.find((d: Department) => d.id === parent_id);
    return parent?.name || '-';
  }

  /**
   * @brief 加载部门列表
   */
  async function loadDepartments() {
    loading.value = true;
    try {
      const response = await listDepartments();
      const respData = response as { list?: Department[]; data?: Department[] | { list?: Department[] } };
      // transformResponse 返回 { code, data: {list,total}, list: [...], ... }
      // data 可能是 {list,total} 对象，需兼容两种结构
      const dataObj = respData.data as { list?: Department[] } | undefined;
      departments.value = respData.list || dataObj?.list || (Array.isArray(respData.data) ? (respData.data) : []) || [];
    } catch (error) {
      logger.error('【加载部门失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    } finally {
      loading.value = false;
    }
  }

  /**
   * @brief 加载用户列表
   */
  async function loadUsers() {
    try {
      const response = await listUsers({ page_size: 100 });
      const respData = response as { list?: User[]; data?: User[] };
      users.value = respData.list || respData.data || [];
    } catch (error) {
      logger.error('【加载用户失败】', error);
    }
  }

  /**
   * @brief 选择部门
   */
  function onDepartmentSelect(id: number | null) {
    selectedDepartmentId.value = id;
  }

  /**
   * @brief 打开部门弹窗
   */
  function openDepartmentDialog(department?: Department, edit = false) {
    if (department) {
      isEdit.value = edit;
      Object.assign(departmentForm, {
        name: department.name,
        parent_id: department.parent_id,
        leader_id: department.leader_id,
        description: department.description || '',
        sort_order: department.sort_order || 0,
      });
    } else {
      isEdit.value = false;
      Object.assign(departmentForm, {
        name: '',
        parent_id: null,
        leader_id: null,
        description: '',
        sort_order: 0,
      });
    }
    showDepartmentDialog.value = true;
  }

  /**
   * @brief 保存部门
   */
  async function saveDepartment() {
    try {
      if (isEdit.value && selectedDepartmentId.value) {
        await updateDepartment(selectedDepartmentId.value, {
          name: departmentForm.name,
          ...(departmentForm.parent_id != null ? { parent_id: departmentForm.parent_id } : {}),
          ...(departmentForm.leader_id != null ? { leader_id: departmentForm.leader_id } : {}),
          description: departmentForm.description,
          sort_order: departmentForm.sort_order,
        });
      } else {
        await createDepartment({
          name: departmentForm.name,
          ...(departmentForm.parent_id != null ? { parent_id: departmentForm.parent_id } : {}),
          ...(departmentForm.leader_id != null ? { leader_id: departmentForm.leader_id } : {}),
          description: departmentForm.description,
          sort_order: departmentForm.sort_order,
        });
      }
      $q.notify({ type: 'positive', message: $t('common.success') });
      showDepartmentDialog.value = false;
      void loadDepartments();
    } catch (error) {
      logger.error('【保存部门失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  /**
   * @brief 处理删除
   */
  function handleDelete(department: Department) {
    pendingDeleteDepartment.value = department;
    deleteDialogRef.value?.open();
  }

  /**
   * @brief 执行删除
   */
  async function doDeleteDepartment() {
    if (!pendingDeleteDepartment.value) return;

    const department_id = pendingDeleteDepartment.value.id;
    try {
      await deleteDepartment(department_id);
      $q.notify({ type: 'positive', message: $t('common.success') });
      if (selectedDepartmentId.value === department_id) {
        selectedDepartmentId.value = null;
      }
      pendingDeleteDepartment.value = null;
      void loadDepartments();
    } catch (error) {
      logger.error('【删除部门失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  // ============ 计算属性 ============

  /**
   * @brief 将部门列表转换为树形结构
   */
  const departmentTree = computed((): DepartmentTreeNode[] => {
    const map = new Map<number, DepartmentTreeNode>();
    const roots: DepartmentTreeNode[] = [];

    departments.value.forEach((dept: Department) => {
      map.set(dept.id, {
        id: dept.id,
        name: dept.name,
        ...(dept.parent_id !== undefined ? { parent_id: dept.parent_id } : {}),
        ...(dept.description ? { description: dept.description } : {}),
        ...(dept.leader_name ? { leader_name: dept.leader_name } : {}),
        ...(dept.leader_id !== undefined ? { leader_id: dept.leader_id } : {}),
        ...(dept.sort_order !== undefined ? { sort_order: dept.sort_order } : {}),
        ...(dept.user_count !== undefined ? { user_count: dept.user_count } : {}),
        children: [],
      });
    });

    departments.value.forEach((dept: Department) => {
      const node = map.get(dept.id);
      if (!node) return;
      if (dept.parent_id && map.has(dept.parent_id)) {
        const parent = map.get(dept.parent_id);
        if (parent) {
          parent.children.push(node);
        }
      } else {
        roots.push(node);
      }
    });

    const sortNodes = (nodes: DepartmentTreeNode[]): void => {
      nodes.sort((a, b) => (a.sort_order || 0) - (b.sort_order || 0));
      nodes.forEach((node) => sortNodes(node.children));
    };
    sortNodes(roots);

    return roots;
  });

  /**
   * @brief 当前选中的部门
   */
  const currentDepartment = computed(() => {
    if (!selectedDepartmentId.value) return null;
    return departments.value.find((d: Department) => d.id === selectedDepartmentId.value) || null;
  });

  /**
   * @brief 父部门选项
   */
  const parentDepartmentOptions = computed(() => {
    return departments.value
      .filter((d: Department) => d.id !== selectedDepartmentId.value)
      .map((d: Department) => ({
        label: getDepartmentPath(d.id),
        value: d.id,
      }));
  });

  /**
   * @brief 用户选项
   */
  const userOptions = computed(() => {
    return users.value.map((u: User) => ({
      label: `${u.username}${u.nickname ? ` (${u.nickname})` : ''}`,
      value: u.id,
    }));
  });

  // ============ 生命周期 ============
  onMounted(() => {
    void Promise.all([loadDepartments(), loadUsers()]);
  });

  return {
    loading,
    departments,
    users,
    selectedDepartmentId,
    showDepartmentDialog,
    isEdit,
    deleteDialogRef,
    pendingDeleteDepartment,
    departmentForm,
    departmentTree,
    currentDepartment,
    parentDepartmentOptions,
    userOptions,
    getParentName,
    onDepartmentSelect,
    openDepartmentDialog,
    saveDepartment,
    handleDelete,
    doDeleteDepartment,
  };
}
