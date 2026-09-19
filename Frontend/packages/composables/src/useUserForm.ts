/**
 * @file useUserForm.ts
 * @description User form state, validation, create/edit logic
 */

import { ref, reactive } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { createUser as createUserApi, type User } from '@/api/user';
import type { UserCreateForm } from '@/types/user';

export function useUserForm() {
  const $q = useQuasar();
  const { t: $t } = useI18n();

  const showUserDialog = ref(false);
  const isEdit = ref(false);
  const currentUser = ref<User | null>(null);
  const userForm = reactive({
    username: '',
    password: '',
    confirmPassword: '',
    nickname: '',
    email: '',
    phone: '',
    role: 'user',
    status: 1,
  });

  const showImportDialog = ref(false);
  const importStep = ref(1);
  const importFile = ref<File | null>(null);
  const importPreview = ref<Array<{ rowIndex: number; username: string; password: string; nickname?: string; email?: string; phone?: string; role?: string; errors?: string[] }>>([]);
  const selectedImportRows = ref<typeof importPreview.value>([]);
  const importing = ref(false);
  const importResult = ref<{ success: number; failed: number } | null>(null);

  const showDetailDialog = ref(false);
  const detailUser = ref<User | null>(null);

  const validCount = ref(0);
  const errorCount = ref(0);

  function openUserDialog(user?: User) {
    if (user) {
      isEdit.value = true;
      currentUser.value = user;
      userForm.username = user.username || '';
      userForm.nickname = user.nickname || '';
      userForm.email = user.email || '';
      userForm.phone = user.phone || '';
      userForm.role = user.role || 'user';
      userForm.status = user.status ?? 1;
      userForm.password = '';
      userForm.confirmPassword = '';
    } else {
      isEdit.value = false;
      currentUser.value = null;
      userForm.username = '';
      userForm.password = '';
      userForm.confirmPassword = '';
      userForm.nickname = '';
      userForm.email = '';
      userForm.phone = '';
      userForm.role = 'user';
      userForm.status = 1;
    }
    showUserDialog.value = true;
  }

  function viewUser(user: User) {
    detailUser.value = user;
    showDetailDialog.value = true;
  }

  async function saveUser(onSuccess: () => void) {
    if (!userForm.username) {
      $q.notify({ type: 'warning', message: $t('user.usernameRequired') });
      return;
    }

    if (!isEdit.value && !userForm.password) {
      $q.notify({ type: 'warning', message: $t('user.passwordRequired') });
      return;
    }

    if (userForm.password && userForm.password !== userForm.confirmPassword) {
      $q.notify({ type: 'warning', message: $t('user.passwordMismatch') });
      return;
    }

    try {
      if (isEdit.value && currentUser.value) {
        const payload: Record<string, unknown> = {};
        if (userForm.nickname && userForm.nickname !== currentUser.value.nickname) payload.nickname = userForm.nickname;
        if (userForm.email && userForm.email !== currentUser.value.email) payload.email = userForm.email;
        if (userForm.phone && userForm.phone !== currentUser.value.phone) payload.phone = userForm.phone;
        if (userForm.role !== currentUser.value.role) {
          const { updateUserRole } = await import('@/api/user');
          await updateUserRole(currentUser.value.id, userForm.role);
        }
        if (userForm.status !== currentUser.value.status && userForm.status !== undefined) {
          const { updateUserStatus } = await import('@/api/user');
          await updateUserStatus(currentUser.value.id, userForm.status);
        }
        if (Object.keys(payload).length > 0) {
      await createUserApi({ ...payload, id: currentUser.value.id } as UserCreateForm);
        }
        $q.notify({ type: 'positive', message: $t('common.success') });
      } else {
        const newUser: Record<string, unknown> = {
          username: userForm.username,
          password: userForm.password,
          role: userForm.role,
        };
        if (userForm.nickname) newUser.nickname = userForm.nickname;
        if (userForm.email) newUser.email = userForm.email;
        if (userForm.phone) newUser.phone = userForm.phone;
        await createUserApi(newUser as UserCreateForm);
        $q.notify({ type: 'positive', message: $t('common.success') });
      }
      showUserDialog.value = false;
      onSuccess();
    } catch (error) {
      console.error('【保存用户失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  function parseCSVLine(line: string): string[] {
    const result: string[] = [];
    let current = '';
    let inQuotes = false;

    for (const char of line) {
      if (char === '"') {
        inQuotes = !inQuotes;
      } else if (char === ',' && !inQuotes) {
        result.push(current);
        current = '';
      } else {
        current += char;
      }
    }
    result.push(current);
    return result;
  }

  async function handleUploadFile(onSuccess: () => void) {
    if (!importFile.value) return;

    try {
      const text = await importFile.value.text();
      const lines = text.split('\n').filter((line) => line.trim());

      if (lines.length < 2) {
        $q.notify({ type: 'negative', message: $t('user.fileEmpty') });
        return;
      }

      const headerLine = lines[0];
      if (!headerLine) return;
      const headers = headerLine.split(',').map((h) => h.trim().replace(/"/g, ''));
      const previewData: typeof importPreview.value = [];

      for (let i = 1; i < lines.length; i++) {
        const line = lines[i];
        if (!line) continue;
        const values = parseCSVLine(line);
        const row: (typeof importPreview.value)[0] = {
          rowIndex: i,
          username: '',
          password: '',
          errors: [],
        };

        headers.forEach((header, index) => {
          const value = values[index]?.trim().replace(/"/g, '') || '';
          switch (header.toLowerCase()) {
            case 'username':
              row.username = value;
              if (!value) row.errors?.push($t('user.usernameRequired'));
              break;
            case 'password':
              row.password = value;
              if (!value) row.errors?.push($t('user.passwordRequired'));
              else if (value.length < 8) row.errors?.push($t('user.passwordTooShort'));
              break;
            case 'nickname':
              row.nickname = value;
              break;
            case 'email':
              row.email = value;
              if (value && !/.+@.+\..+/.test(value)) row.errors?.push($t('user.emailInvalid'));
              break;
            case 'phone':
              row.phone = value;
              if (value && !/^1[3-9]\d{9}$/.test(value)) row.errors?.push($t('user.phoneInvalid'));
              break;
            case 'role':
              row.role = value || 'user';
              if (value && !['admin', 'user', 'vip'].includes(value)) {
                row.errors?.push($t('user.roleInvalid'));
              }
              break;
          }
        });

        previewData.push(row);
      }

      importPreview.value = previewData;
      selectedImportRows.value = previewData.filter((r) => !r.errors?.length);
      validCount.value = previewData.filter((r) => !r.errors?.length).length;
      errorCount.value = previewData.filter((r) => r.errors?.length).length;
      importStep.value = 3;
      onSuccess();
    } catch (error) {
      console.error('【解析文件失败】', error);
      $q.notify({ type: 'negative', message: $t('user.parseError') });
    }
  }

  async function handleStartImport(onSuccess: () => void) {
    if (selectedImportRows.value.length === 0) {
      $q.notify({ type: 'warning', message: $t('user.noValidRows') });
      return;
    }

    importing.value = true;
    let successCount = 0;
    let failCount = 0;

    for (const row of selectedImportRows.value) {
      try {
        const newUser: Record<string, unknown> = {
          username: row.username,
          password: row.password,
          role: row.role || 'user',
        };
        if (row.nickname) newUser.nickname = row.nickname;
        if (row.email) newUser.email = row.email;
        if (row.phone) newUser.phone = row.phone;
        await createUserApi(newUser as UserCreateForm);
        successCount++;
      } catch (error) {
        failCount++;
        console.error(`【导入用户 ${row.username} 失败】`, error);
      }
    }

    importing.value = false;
    importResult.value = { success: successCount, failed: failCount };
    importStep.value = 4;
    onSuccess();
  }

  function handleImportComplete(onSuccess: () => void) {
    showImportDialog.value = false;
    importStep.value = 1;
    importFile.value = null;
    importPreview.value = [];
    selectedImportRows.value = [];
    importResult.value = null;
    onSuccess();
  }

  return {
    showUserDialog,
    isEdit,
    currentUser,
    userForm,
    showImportDialog,
    importStep,
    importFile,
    importPreview,
    selectedImportRows,
    importing,
    importResult,
    showDetailDialog,
    detailUser,
    validCount,
    errorCount,
    openUserDialog,
    viewUser,
    saveUser,
    handleUploadFile,
    handleStartImport,
    handleImportComplete,
  };
}
