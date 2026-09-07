<template>
  <q-layout view="hHh Lpr lFf">
    <!-- 顶部导航栏 -->
    <q-header elevated class="bg-primary text-white">
      <q-toolbar>
        <q-btn dense flat round icon="menu" @click="toggleLeftDrawer" />

        <q-toolbar-title>
          <q-icon name="forum" class="q-mr-sm" />
          Social
        </q-toolbar-title>

        <q-btn
          v-if="auth.isLoggedIn"
          flat
          round
          dense
          icon="account_circle"
          @click="goToSettings"
        >
          <q-tooltip>设置</q-tooltip>
        </q-btn>
      </q-toolbar>
    </q-header>

    <!-- 侧边栏 -->
    <q-drawer
      v-model="leftDrawerOpen"
      show-if-above
      bordered
      :width="280"
      class="bg-grey-1"
    >
      <!-- 用户信息区域（已登录时） -->
      <template v-if="auth.isLoggedIn">
        <q-list class="q-mt-md">
          <q-item class="q-mb-sm">
            <q-item-section avatar>
              <q-avatar size="48px" color="primary" text-color="white">
                {{ displayNameInitial }}
              </q-avatar>
            </q-item-section>
            <q-item-section>
              <q-item-label class="text-weight-bold">{{ displayName }}</q-item-label>
              <q-item-label caption class="text-grey-7">{{ auth.userId }}</q-item-label>
            </q-item-section>
            <q-item-section side>
              <q-btn flat round dense icon="logout" size="sm" @click="handleLogout">
                <q-tooltip>退出登录</q-tooltip>
              </q-btn>
            </q-item-section>
          </q-item>
        </q-list>

        <q-separator />
      </template>

      <!-- 未登录时显示 -->
      <template v-else>
        <div class="column items-center q-mt-xl q-mb-lg">
          <q-icon name="forum" size="64px" color="primary" />
          <div class="text-h6 q-mt-md text-grey-8">Social</div>
          <div class="text-caption text-grey-6">连接你我，即时沟通</div>
        </div>
        <q-separator />
      </template>

      <!-- 主导航 -->
      <q-list class="q-mt-sm">
        <q-item
          v-for="item in navItems"
          :key="item.path"
          clickable
          v-ripple
          :to="item.path"
          :active="$route.path === item.path || $route.path.startsWith(item.path + '/')"
          active-class="bg-primary text-white"
          class="q-mx-sm q-my-xs rounded-borders"
          v-show="!item.requiresAuth || auth.isLoggedIn"
        >
          <q-item-section avatar>
            <q-icon :name="item.icon" />
          </q-item-section>
          <q-item-section>
            <q-item-label>{{ item.label }}</q-item-label>
          </q-item-section>
          <q-item-section v-if="item.badge" side>
            <q-badge :label="item.badge" color="red" />
          </q-item-section>
        </q-item>
      </q-list>

      <q-separator class="q-mt-md" />

      <!-- 底部导航：未登录时显示登录按钮 -->
      <div v-if="!auth.isLoggedIn" class="absolute-bottom q-pa-md">
        <q-btn
          color="primary"
          class="full-width"
          icon="login"
          label="登录 / 注册"
          to="/login"
          :replace="true"
        />
      </div>
    </q-drawer>

    <!-- 主内容区域 -->
    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useQuasar } from 'quasar';

const $q = useQuasar();
const router = useRouter();
const auth = useAuthStore();

const leftDrawerOpen = ref(false);

const displayName = computed(() => {
  if (!auth.userId) return '用户';
  return (auth.userId.split(':')[0] ?? '').replace('@', '');
});

const displayNameInitial = computed(() => {
  const name = auth.userId ? (auth.userId.split(':')[0] ?? '').replace('@', '') : '?';
  return name.charAt(0).toUpperCase();
});

interface NavItem {
  path: string;
  icon: string;
  label: string;
  badge?: string;
  requiresAuth: boolean;
}

const navItems: NavItem[] = [
  { path: '/rooms', icon: 'chat', label: '聊天室', requiresAuth: true },
  { path: '/contacts', icon: 'contacts', label: '通讯录', requiresAuth: true },
  { path: '/moments', icon: 'photo_library', label: '朋友圈', requiresAuth: true },
  { path: '/settings', icon: 'settings', label: '设置', requiresAuth: true },
];

function toggleLeftDrawer() {
  leftDrawerOpen.value = !leftDrawerOpen.value;
}

function goToSettings() {
  void router.push('/settings');
}

function handleLogout() {
  $q.dialog({
    title: '确认退出',
    message: '确定要退出登录吗？',
    cancel: true,
    persistent: true,
  }).onOk(() => {
    auth.logout().then(() => {
      void router.push('/login');
      $q.notify({ type: 'info', message: '已退出登录' });
    }).catch((e: unknown) => {
      $q.notify({ type: 'negative', message: '退出失败: ' + (e instanceof Error ? e.message : '未知错误') });
    });
  });
}
</script>

<style scoped>
.rounded-borders {
  border-radius: 8px;
}
</style>
