<template>
  <q-header elevated class="main-header">
    <q-toolbar>
      <!-- 移动端菜单按钮 -->
      <q-btn
        v-if="isMobile"
        flat
        dense
        round
        icon="menu"
        :aria-label="$t('common.menu')"
        class="menu-btn"
        @click="$emit('toggleLeftDrawer')"
      >
        <q-tooltip>{{ $t('common.menu') }}</q-tooltip>
      </q-btn>

      <!-- 桌面端logo -->
      <q-avatar v-if="!isMobile" size="36px" class="logo-avatar q-mr-sm">
        <q-icon name="smart_toy" size="24px" />
      </q-avatar>

      <!-- 面包屑 -->
      <q-breadcrumbs v-if="!isMobile" active-color="white" style="font-size: 14px">
        <q-breadcrumbs-el icon="home" to="/" />
        <q-breadcrumbs-el v-if="currentRoute.meta?.title" :label="currentRoute.meta.title" />
      </q-breadcrumbs>

      <q-space />

      <!-- 桌面端搜索框 -->
      <q-input
        v-if="!isMobile"
        :model-value="searchText"
        dense
        outlined
        :placeholder="$t('cmd.openHint')"
        class="q-mr-md search-input desktop-only"
        dark
        readonly
        @click="$emit('openCommandPalette')"
      >
        <template v-slot:prepend>
          <q-icon name="search" />
        </template>
        <template v-slot:append>
          <kbd class="search-shortcut">⌘K</kbd>
        </template>
      </q-input>

      <!-- 全屏切换（桌面端） -->
      <q-btn
        v-if="!isMobile"
        flat
        dense
        round
        :icon="isFullscreen ? 'fullscreen_exit' : 'fullscreen'"
        @click="$emit('toggleFullscreen')"
        class="q-mr-sm desktop-only"
      >
        <q-tooltip>{{ isFullscreen ? $t('common.exitFullscreen') : $t('common.fullscreen') }}</q-tooltip>
      </q-btn>

      <!-- 主题切换 -->
      <q-btn flat dense round :icon="themeIcon" @click="$emit('toggleTheme')" class="q-mr-sm">
        <q-tooltip>{{ $t('theme.title') }}</q-tooltip>
      </q-btn>

      <!-- 通知面板 -->
      <NotificationPanel />

      <!-- 桌面端快捷键帮助 -->
      <q-btn
        v-if="!isMobile"
        flat
        dense
        round
        icon="keyboard"
        class="q-mr-sm desktop-only"
        @click="$emit('openShortcutsHelp')"
      >
        <q-tooltip>{{ $t('shortcuts.title') }}</q-tooltip>
      </q-btn>

      <!-- 用户菜单（桌面端） -->
      <q-btn v-if="!isMobile" flat dense no-caps class="user-menu" data-testid="user-menu">
        <q-avatar size="32px" color="white" text-color="primary" class="q-mr-sm">
          {{ userAvatarText }}
        </q-avatar>
        <span class="text-white">{{ $t('common.user') }}</span>
        <q-menu>
          <q-list style="min-width: 180px">
            <q-item clickable v-close-popup @click="$emit('goToProfile')">
              <q-item-section avatar>
                <q-icon name="person" />
              </q-item-section>
              <q-item-section>{{ $t('menu.personalCenter') }}</q-item-section>
            </q-item>
            <q-separator />
            <q-item clickable v-close-popup @click="$emit('openLogoutDialog')" data-testid="logout-button">
              <q-item-section avatar>
                <q-icon name="logout" color="negative" />
              </q-item-section>
              <q-item-section class="text-negative">{{ $t('common.logout') }}</q-item-section>
            </q-item>
          </q-list>
        </q-menu>
      </q-btn>
    </q-toolbar>
  </q-header>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { RouteLocationNormalizedLoaded } from 'vue-router';
import NotificationPanel from '@/components/NotificationPanel.vue';

const { t } = useI18n();

defineProps<{
  isMobile: boolean;
  isFullscreen: boolean;
  themeIcon: string;
  userAvatarText: string;
  currentRoute: RouteLocationNormalizedLoaded;
  searchText: string;
  leftDrawerOpen: boolean;
}>();

defineEmits<{
  toggleLeftDrawer: [];
  toggleFullscreen: [];
  toggleTheme: [];
  openShortcutsHelp: [];
  goToProfile: [];
  openLogoutDialog: [];
  openCommandPalette: [];
}>();
</script>
