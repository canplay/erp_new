<template>
  <q-page class="flex flex-center">
    <q-card style="min-width: 250px">
      <q-card-section>
        <q-form @submit="onSubmit" @reset="onReset" class="q-gutter-md">
          <q-input
            v-model="username"
            :label="$t('common.username')"
            lazy-rules
            :rules="[(val: string) => (val && val.length > 0) || '此为必填项']"
          />

          <q-input
            v-model="password"
            type="password"
            :label="$t('common.password')"
            lazy-rules
            :rules="[(val: string) => (val && val.length > 0) || '此为必填项']"
          />

          <div class="row">
            <q-btn class="col" :label="$t('common.reset')" type="reset" color="primary" flat />
            <div class="col-auto" style="width: 5px" />
            <q-btn class="col" :label="$t('common.login')" type="submit" color="primary" />
          </div>
        </q-form>
      </q-card-section>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { useEbikeStore as useStore, type ScheduleParams } from '@/stores/ebike';
import { httpClient as api } from '@/utils/alova';

interface UserInfo {
  msg: {
    id: string;
    username: string;
    password: string;
    level: number;
    name: string;
  };
}

const router = useRouter();
const $q = useQuasar();
const store = useStore();

const username = ref('');
const password = ref('');

function onReset(): void {
  username.value = '';
  password.value = '';
}

async function onSubmit(): Promise<void> {
  $q.cookies.remove('ebike');

  try {
    const resp = await api.post(store.backend.private + '/login', {
      username: username.value,
      password: password.value,
    });

    const data = resp.data as { token?: string; user?: { id: number; username: string; nickname: string; role: string } };
    if (!data?.token) {
      $q.notify('用户名或密码错误');
      return;
    }

    // 用 JWT 登录返回的用户信息填充 store
    await login({
      msg: {
        id: String(data.user?.id ?? ''),
        username: data.user?.username ?? username.value,
        password: password.value,
        level: data.user?.role === 'admin' ? 0 : 1,
        name: data.user?.nickname ?? data.user?.username ?? '',
      },
    });
  } catch {
    $q.notify('用户名或密码错误');
  }
}

async function onReLogin(user?: UserInfo): Promise<void> {
  if (store.test) {
    void router.push('/index');
    return;
  }

  if (user) {
    await login(user);
  } else if ($q.cookies.has('ebike') && $q.cookies.get('ebike') !== undefined) {
    try {
      const resp = await api.get(
        store.backend.private + '/info/' + $q.cookies.get('ebike'),
      );
      await login(resp.data as UserInfo);
    } catch {
      // silent
    }
  }
}

async function login(user: UserInfo): Promise<void> {
  $q.loading.show();
  let loadingTimer: ReturnType<typeof setTimeout> | null = null;
  loadingTimer = setTimeout(() => {
    $q.loading.hide();
    if (loadingTimer !== null) {
      clearTimeout(loadingTimer);
      loadingTimer = null;
    }
  }, 120000);

  try {
    if (!store.test) {
      const resp = await api.post(store.backend.private + '/options', {
        method: 'query',
      });
      const data = resp.data as Array<Record<string, unknown>>;
      for (const element of data) {
        const el = element as { level?: number; options?: unknown };
        if (el.level === 0 && el.options) {
          store.setOptions(el.options as ScheduleParams);
        } else if (el.level) {
          store.setOptionsList([el]);
        }
      }

      store.initSchedule();

      $q.cookies.set('ebike', user.msg.id);

      store.login({
        username: user.msg.username,
        password: user.msg.password,
        level: user.msg.level,
        name: user.msg.name,
      });
    }

    $q.loading.hide();
    if (loadingTimer !== null) {
      clearTimeout(loadingTimer);
      loadingTimer = null;
    }
    void router.push('/index');
  } catch (e) {
    console.error('登录失败:', e);
    $q.loading.hide();
    if (loadingTimer !== null) {
      clearTimeout(loadingTimer);
      loadingTimer = null;
    }
    $q.notify('网络错误，请稍后重试');
  }
}

onMounted(() => {
  void onReLogin();
});
</script>
