<template>
  <q-btn flat round @click="handleClick" v-if="showBadge">
    <q-icon name="add_to_home_screen" size="24px">
      <q-badge v-if="updateAvailable" floating color="warning" />
    </q-icon>
    <q-tooltip>{{ $t('pwa.install') }}</q-tooltip>
  </q-btn>
</template>

<script setup lang="ts">
/**
 * @file PwaNotification.vue
 * @description PWA 安装提示组件
 * @date 2026-04-04
 */

import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { logger } from '@/utils/logger';

const { t: $t } = useI18n();

// PWA beforeinstallprompt 事件类型
interface BeforeInstallPromptEvent extends Event {
  readonly platforms: string[];
  readonly userChoice: Promise<{ outcome: 'accepted' | 'dismissed'; platform: string }>;
  prompt(): Promise<void>;
}

const showBadge = ref(false);
const updateAvailable = ref(false);
const deferredPrompt = ref<BeforeInstallPromptEvent | null>(null);

/**
 * @brief 监听 PWA 安装事件
 */
onMounted(() => {
  // 监听 beforeinstallprompt 事件
  window.addEventListener('beforeinstallprompt', (e) => {
    e.preventDefault();
    deferredPrompt.value = e as BeforeInstallPromptEvent;
    showBadge.value = true;
  });

  // 监听 app installed 事件
  window.addEventListener('appinstalled', () => {
    deferredPrompt.value = null;
    showBadge.value = false;
  });

  // 检查更新
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.addEventListener('controllerchange', () => {
      updateAvailable.value = true;
    });
  }
});

/**
 * @brief 处理点击安装
 */
async function handleClick() {
  if (!deferredPrompt.value) return;

  // 显示安装提示（fire-and-forget）
  void deferredPrompt.value.prompt();

  // 等待用户响应
  const { outcome } = await deferredPrompt.value.userChoice;
  logger.info('[PWA] 安装结果:', outcome);

  if (outcome === 'accepted') {
    showBadge.value = false;
  }

  deferredPrompt.value = null;
}
</script>

