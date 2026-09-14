import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useUserStore = defineStore('user', () => {
  const users = ref<any[]>([]);
  const currentUser = ref<any>(null);
  const loading = ref(false);

  function setUsers(u: any[]) { users.value = u; }
  function setCurrentUser(u: any) { currentUser.value = u; }
  function setLoading(v: boolean) { loading.value = v; }

  return { users, currentUser, loading, setUsers, setCurrentUser, setLoading };
});

export const useTenantStore = defineStore('tenant', () => {
  const tenants = ref<any[]>([]);
  const currentTenant = ref<any>(null);

  function setTenants(t: any[]) { tenants.value = t; }
  function setCurrentTenant(t: any) { currentTenant.value = t; }

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
  const notifications = ref<any[]>([]);
  const unreadCount = ref(0);

  function addNotification(n: any) { notifications.value.unshift(n); unreadCount.value++; }
  function markAllRead() { unreadCount.value = 0; }

  return { notifications, unreadCount, addNotification, markAllRead };
});
