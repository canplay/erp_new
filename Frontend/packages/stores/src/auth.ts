import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

const TOKEN_KEY = 'auth_token';

export interface AuthUser {
  id?: number;
  username?: string;
  email?: string;
  roles?: string[];
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem(TOKEN_KEY));
  const user = ref<AuthUser | null>(null);

  const isAuthenticated = computed(() => !!token.value);

  function setToken(t: string) {
    token.value = t;
    localStorage.setItem(TOKEN_KEY, t);
  }

  function setUser(u: AuthUser) {
    user.value = u;
  }

  function logout() {
    token.value = null;
    user.value = null;
    localStorage.removeItem(TOKEN_KEY);
  }

  return { token, user, isAuthenticated, setToken, setUser, logout };
});
