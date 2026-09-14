<template>
  <q-layout view="lHh Lpr lFf">
    <!-- ============ 顶栏 ============ -->
    <MainHeader
      :is-mobile="isMobile"
      :is-fullscreen="isFullscreen"
      :theme-icon="themeIcon"
      :user-avatar-text="userAvatarText"
      :current-route="currentRoute"
      :search-text="searchText"
      :left-drawer-open="leftDrawerOpen"
      @toggle-left-drawer="toggleLeftDrawer"
      @toggle-fullscreen="toggleFullscreen"
      @toggle-theme="toggleTheme"
      @open-shortcuts-help="openShortcutsHelp"
      @go-to-profile="goToProfile"
      @open-logout-dialog="openLogoutDialog"
    />

    <!-- ============ 侧边栏 ============ -->
    <MainDrawer
      v-model:left-drawer-open="leftDrawerOpen"
      :is-mobile="isMobile"
      :user-avatar-text="userAvatarText"
      :username="authStore.userInfo?.username || ''"
      :role="String(authStore.userInfo?.role ?? '')"
      :visible-categories="visibleCategories"
    />

    <!-- ============ 主内容区 ============ -->
    <q-page-container>
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </q-page-container>

    <!-- ============ 页脚（桌面端/移动端）= 只保留一个 QFooter ============ -->
    <LayoutFooter
      :is-mobile="isMobile"
      :mobile-nav-items="mobileNavItems"
      :is-current-route="isCurrentRoute"
      @navigate-to="navigateTo"
    />

    <!-- 退出登录确认对话框 -->
    <ConfirmDialog
      ref="logoutDialogRef"
      :title="$t('common.logoutConfirm')"
      :message="$t('common.logoutMessage')"
      icon="logout"
      confirm-color="negative"
      @confirm="handleLogout"
    />

    <!-- 快捷键帮助对话框 -->
    <ShortcutsHelp ref="shortcutsHelpRef" />

    <!-- 命令面板 -->
    <CommandPalette :ref="(el: unknown) => { if (commandPaletteRef && typeof commandPaletteRef === 'object' && 'value' in commandPaletteRef) commandPaletteRef.value = el as InstanceType<typeof CommandPalette> | null }" :commands="commands" />
  </q-layout>
</template>

<script setup lang="ts">
/**
 * @file MainLayout.vue
 * @description 主布局组件 - 支持桌面端和移动端
 * @date 2026-04-03
 */

import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from '@/stores/auth';
import { useThemeStore } from '@/stores/auth';
import { MENU_CONFIG } from '@/config/menu';
import ConfirmDialog from '@erp-new-frontend-monorepo/components/src/ConfirmDialog.vue';
import ShortcutsHelp from '@erp-new-frontend-monorepo/components/src/ShortcutsHelp.vue';
import CommandPalette from '@erp-new-frontend-monorepo/components/src/CommandPalette.vue';
import MainHeader from '../MainHeader.vue';
import MainDrawer from '../MainDrawer.vue';
import LayoutFooter from '../LayoutFooter.vue';

const { t } = useI18n();
const shortcutsHelpRef = ref<InstanceType<typeof ShortcutsHelp> | null>(null);
const commandPaletteRef = ref<InstanceType<typeof CommandPalette> | null>(null);

const $q = useQuasar();
const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const themeStore = useThemeStore();

// ============ 状态 ============
const leftDrawerOpen = ref(true);
const searchText = ref('');
const isFullscreen = ref(false);
const isMobile = ref(false);
const logoutDialogRef = ref<InstanceType<typeof ConfirmDialog> | null>(null);

// ============ 计算属性 ============
const currentRoute = computed(() => route);

const userAvatarText = computed(() => {
  const username = authStore.userInfo?.username || 'A';
  return username.substring(0, 1).toUpperCase();
});

/**
 * @brief 根据主题模式获取图标
 */
const themeIcon = computed(() => {
  switch (themeStore.mode) {
    case 'dark':
      return 'dark_mode';
    case 'light':
      return 'light_mode';
    default:
      return 'brightness_auto';
  }
});

/**
 * @brief 移动端底部导航项
 */
const mobileNavItems = computed(() => [
  { path: '/', label: t('menu.dashboard') || '首页', icon: 'dashboard' },
  { path: '/ctp', label: t('menu.ctpLock') || '电子围栏', icon: 'lock', adminOnly: true },
  { path: '/xlt', label: t('menu.xltDashboard') || '停车场', icon: 'local_parking', adminOnly: true },
  { path: '/profile', label: t('menu.personalCenter') || '我的', icon: 'account_circle' },
]);

/**
 * @brief 判断是否为当前路由
 */
function isCurrentRoute(path: string): boolean {
  if (path === '/') {
    return route.path === '/';
  }
  return route.path.startsWith(path);
}

// ============ 菜单配置（使用共享 MENU_CONFIG） ============
const menuList = computed(() => MENU_CONFIG);

// ============ 菜单权限过滤 ============
const visibleCategories = computed(() => {
  return menuList.value
    .map((category) => {
      // 扁平结构：category.items
      let items = (category.items || []).filter((item) => {
        if (!item.permissions || item.permissions.length === 0) return true;
        return item.permissions.some((perm: string) => authStore.hasPermission(perm));
      });
      // 多级结构：category.children → 递归提取叶子节点作为直接链接
      if (items.length === 0 && category.children) {
        items = flattenChildren(category.children);
      }
      return { ...category, items };
    })
    .filter((category) => category.items.length > 0);
});

/** 递归提取 children 中的所有叶子节点（有 path 的项） */
function flattenChildren(children: Array<{ path?: string; title?: string; icon?: string; children?: unknown[]; permissions?: string[] }>): Array<{ path: string; title: string; icon: string }> {
  const result: Array<{ path: string; title: string; icon: string }> = [];
  for (const child of children) {
    if (child.path) {
      // 叶子节点 → 直接加入
      result.push({ path: child.path, title: child.title ?? '', icon: child.icon ?? '' });
    }
    if (child.children && child.children.length > 0) {
      // 有子节点 → 递归展开
      result.push(...flattenChildren(child.children as Array<{ path?: string; title?: string; icon?: string; children?: unknown[]; permissions?: string[] }>));
    }
  }
  return result;
}

// ============ 方法 ============

/**
 * @brief 切换侧边栏
 */
function toggleLeftDrawer() {
  leftDrawerOpen.value = !leftDrawerOpen.value;
}

/**
 * @brief 切换全屏
 */
function toggleFullscreen() {
  if (!$q.fullscreen.isActive) {
    void $q.fullscreen.request();
    isFullscreen.value = true;
  } else {
    void $q.fullscreen.exit();
    isFullscreen.value = false;
  }
}

/**
 * @brief 切换主题
 */
function toggleTheme() {
  themeStore.toggleTheme();
}

/**
 * @brief 跳转个人中心
 */
function goToProfile() {
  void router.push('/profile');
}

/**
 * @brief 打开退出登录确认对话框
 */
function openLogoutDialog() {
  logoutDialogRef.value?.open();
}

/**
 * @brief 打开快捷键帮助对话框
 */
function openShortcutsHelp() {
  shortcutsHelpRef.value?.open();
}

/**
 * @brief 执行退出登录
 */
function handleLogout() {
  authStore.logout();
  $q.notify({
    type: 'positive',
    message: t('common.loggedOut'),
    position: 'top',
  });
}

/**
 * @brief 跳转到指定页面
 */
const commands = computed(() => [
  {
    key: 'open-command-palette',
    label: t('common.commandPalette'),
    icon: 'search',
    action: () => openCommandPalette(),
  },
]);

function openCommandPalette() {
  commandPaletteRef.value?.open();
}

function navigateTo(path: string) {
  void router.push(path);
}

/**
 * @brief 检测是否为移动端
 */
function checkMobile() {
  const wasMobile = isMobile.value;
  isMobile.value = window.innerWidth < 768;

  // 移动端默认关闭侧边栏
  if (isMobile.value && !wasMobile) {
    leftDrawerOpen.value = false;
  }
  // 桌面端默认打开侧边栏
  if (!isMobile.value && wasMobile) {
    leftDrawerOpen.value = true;
  }
}

// ============ 生命周期 ============
onMounted(() => {
  // 初始化主题
  themeStore.init();
  // 检测移动端
  checkMobile();
  // 监听窗口大小变化
  window.addEventListener('resize', checkMobile);

  // 监听命令面板事件
  document.addEventListener('open-command-palette', () => {
    commandPaletteRef.value?.open();
  });
  document.addEventListener('close-command-palette', () => {
    commandPaletteRef.value?.close();
  });
});

onUnmounted(() => {
  window.removeEventListener('resize', checkMobile);
});
</script>
