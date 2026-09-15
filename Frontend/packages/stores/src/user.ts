import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { User } from '@erp-new-frontend-monorepo/types';
import type { UserInfo } from '@erp-new-frontend-monorepo/types';
import type { Tenant } from '@erp-new-frontend-monorepo/types';
import type { Notification } from '@erp-new-frontend-monorepo/types';

export const useUserStore = defineStore('user', () => {
  const users = ref<User[]>([]);
  const currentUser = ref<UserInfo | null>(null);
  const loading = ref(false);

  function setUsers(u: User[]) { users.value = u; }
  function setCurrentUser(u: UserInfo) { currentUser.value = u; }
  function setLoading(v: boolean) { loading.value = v; }

  return { users, currentUser, loading, setUsers, setCurrentUser, setLoading };
});

export const useTenantStore = defineStore('tenant', () => {
  const tenants = ref<Tenant[]>([]);
  const currentTenant = ref<Tenant | null>(null);

  function setTenants(t: Tenant[]) { tenants.value = t; }
  function setCurrentTenant(t: Tenant) { currentTenant.value = t; }

  return { tenants, currentTenant, setTenants, setCurrentTenant };
});

export const useThemeStore = defineStore('theme', () => {
  const darkMode = ref(false);
  const primaryColor = ref('#1976d2');

  function toggleDark() { darkMode.value = !darkMode.value; }
  function setPrimaryColor(c: string) { primaryColor.value = c; }

  return { darkMode, primaryColor, toggleDark, setPrimaryColor };
});

export const useNotificationStore = defineStore('notification', () => {
  const notifications = ref<Notification[]>([]);
  const unreadCount = ref(0);

  function addNotification(n: Notification) { notifications.value.unshift(n); unreadCount.value++; }
  function markAllRead() { unreadCount.value = 0; }

  return { notifications, unreadCount, addNotification, markAllRead };
});
