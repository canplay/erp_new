<template>
  <ErrorBoundary>
    <router-view />
  </ErrorBoundary>
</template>

<script setup lang="ts">
/**
 * @file App.vue
 * @description 应用根组件
 * @date 2026-04-03
 */

import { onMounted, watch } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { logger } from '@/utils/logger';
import { useWebSocket } from '@erp-new-frontend-monorepo/composables/src/useWebSocket';;
import { useGlobalErrorHandler } from '@erp-new-frontend-monorepo/composables/src/useGlobalErrorHandler';;
import ErrorBoundary from '@erp-new-frontend-monorepo/components/src/ErrorBoundary.vue';

const authStore = useAuthStore();
const { connect, disconnect } = useWebSocket();
const { initGlobalErrorHandler } = useGlobalErrorHandler();

// WebSocket 配置
// 修复 (2026-08-07): WS路径从 /ws 改为 /ws/messages —— 网关实际路由是
// /ws/status 和 /ws/messages (route_builder.rs:44-45), /ws 不存在导致
// 'HTTP Authentication failed; no valid credentials available' (握手404)
// 浏览器 WebSocket API 无法在握手时带 Authorization 头, token 通过
// URL 查询参数 ?token= 传递, 后端 ws handler 内验证
const WS_BASE = import.meta.env.VITE_WS_URL || `${window.location.protocol}//${window.location.hostname}:${window.location.port}/ws/messages`;
const WS_URL = authStore.token ? `${WS_BASE}?token=${encodeURIComponent(authStore.token)}` : WS_BASE;

/**
 * @brief 初始化 WebSocket 连接
 */
function initWebSocket() {
  if (authStore.isLoggedIn) {
    logger.info('【App】初始化 WebSocket 连接...');
    connect({ url: WS_URL });
  }
}

// 监听登录状态变化
watch(
  () => authStore.isLoggedIn,
  (isLoggedIn) => {
    if (isLoggedIn) {
      initWebSocket();
    } else {
      disconnect();
    }
  }
);

onMounted(() => {
  // 初始化全局错误处理器
  initGlobalErrorHandler();

  // 如果已登录，初始化 WebSocket
  if (authStore.isLoggedIn) {
    initWebSocket();
  }

  logger.info('【App】管理后台已初始化');
});
</script>

