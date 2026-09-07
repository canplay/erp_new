import { defineStore } from 'pinia';
import { ref } from 'vue';
import { accountApi, type SocialAccount } from '@/api/accounts';

export const useAccountStore = defineStore('account', () => {
  const accounts = ref<SocialAccount[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchAccounts() {
    loading.value = true;
    error.value = null;
    try {
      accounts.value = await accountApi.list();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载账号列表失败';
    } finally {
      loading.value = false;
    }
  }

  async function createAccount(data: Partial<SocialAccount>) {
    error.value = null;
    try {
      const result = await accountApi.create(data);
      accounts.value.push(result);
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '创建账号失败';
      throw e;
    }
  }

  async function updateAccount(id: string, data: Partial<SocialAccount>) {
    error.value = null;
    try {
      const result = await accountApi.update(id, data);
      const idx = accounts.value.findIndex((a) => a.id === id);
      if (idx >= 0) accounts.value[idx] = result;
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '更新账号失败';
      throw e;
    }
  }

  async function deleteAccount(id: string) {
    error.value = null;
    try {
      await accountApi.delete(id);
      accounts.value = accounts.value.filter((a) => a.id !== id);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除账号失败';
      throw e;
    }
  }

  return { accounts, loading, error, fetchAccounts, createAccount, updateAccount, deleteAccount };
});
