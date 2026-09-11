/**
 * @file useAuthLogin.ts
 * @description LoginPage 登录/注册/SSO 逻辑 — 从 LoginPage.vue 拆分
 */

import { ref, reactive, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useQuasar } from 'quasar';
import { useAuthStore } from '@/stores/auth';
import { useRoomStore } from '@/stores/room';
import { useContactStore } from '@/stores/contact';
import { config } from '@/services/config';

export function useAuthLogin() {
  const $q = useQuasar();
  const router = useRouter();
  const route = useRoute();
  const auth = useAuthStore();
  const roomStore = useRoomStore();
  const contactStore = useContactStore();

  const tab = ref<'login' | 'register'>('login');
  const showPassword = ref(false);
  const serverUrl = ref(config.tuwunelUrl);
  const loginFlows = reactive({ password: true, sso: false });

  const form = reactive({
    username: '',
    displayName: '',
    password: '',
    confirmPassword: '',
  });

  // 页面加载时获取可用登录流
  onMounted(async () => {
    auth.clearError();
    // 检测 SSO 是否可用
    try {
      const { MatrixService } = await import('@/services');
      const ms = MatrixService.getInstance();
      const flows = await ms.getAvailableLoginFlows();
      loginFlows.password = flows.password;
      loginFlows.sso = flows.sso;
    } catch {
      // 默认只显示密码登录
    }

    // 检查是否是 SSO 回调
    if (route.query.sso_callback === '1') {
      void handleSsoCallback();
    }
  });

  async function handleLogin() {
    auth.clearError();
    try {
      const creds = await auth.login(form.username, form.password);

      localStorage.setItem('matrix_access_token', creds.accessToken);
      localStorage.setItem('matrix_user_id', creds.userId);
      localStorage.setItem('matrix_device_id', creds.deviceId);

      roomStore.loadRooms();
      contactStore.loadContacts();

      $q.notify({ type: 'positive', message: `欢迎回来，${creds.userId}` });

      const redirect = (route.query.redirect as string) || '/rooms';
      void router.push(redirect);
    } catch (e: unknown) {
      $q.notify({
        type: 'negative',
        message: e instanceof Error ? e.message : '登录失败',
        icon: 'error',
      });
    }
  }

  async function handleRegister() {
    auth.clearError();
    try {
      const creds = await auth.register(
        form.username,
        form.password,
        form.displayName || undefined,
      );

      localStorage.setItem('matrix_access_token', creds.accessToken);
      localStorage.setItem('matrix_user_id', creds.userId);
      localStorage.setItem('matrix_device_id', creds.deviceId);

      $q.notify({
        type: 'positive',
        message: `注册成功！欢迎 ${creds.userId}`,
      });

      void router.push('/rooms');
    } catch (e: unknown) {
      $q.notify({
        type: 'negative',
        message: e instanceof Error ? e.message : '注册失败',
        icon: 'error',
      });
    }
  }

  /** 发起 SSO 登录 */
  async function handleSsoLogin() {
    const { MatrixService } = await import('@/services');
    const ms = MatrixService.getInstance();
    const redirect = (route.query.redirect as string) || '/rooms';
    const url = ms.getSsoLoginUrl(redirect);
    window.location.href = url;
  }

  /** 处理 SSO 回调 */
  async function handleSsoCallback() {
    auth.clearError();
    const loginToken = route.query.loginToken as string | undefined;
    const redirect = (route.query.redirect as string) || '/rooms';

    if (loginToken) {
      try {
        const creds = await auth.loginWithToken(loginToken);
        $q.notify({ type: 'positive', message: `SSO 登录成功，欢迎 ${creds.userId}` });
        void router.push(redirect);
      } catch (e: unknown) {
        $q.notify({ type: 'negative', message: 'SSO 登录失败: ' + (e instanceof Error ? e.message : '未知错误') });
      }
    }
  }

  return {
    tab,
    showPassword,
    serverUrl,
    loginFlows,
    form,
    auth,
    handleLogin,
    handleRegister,
    handleSsoLogin,
  };
}
