/**
 * @file ErrorBoundary.vue
 * @description 全局错误边界组件 - 捕获子组件渲染错误
 * @date 2026-04-03
 */

<template>
  <slot v-if="!hasError" />
  <div v-else class="error-boundary q-pa-xl text-center">
    <q-icon name="error_outline" size="64px" color="negative" />
    <div class="text-h5 q-mt-md text-negative">{{ $t('error.boundary.title') }}</div>
    <div class="text-body1 text-grey-6 q-mt-md">
      {{ $t('error.boundary.message') }}
    </div>
    <div class="q-mt-lg">
      <q-btn color="primary" :label="$t('common.refresh')" icon="refresh" @click="handleReload" />
      <q-btn flat color="grey" :label="$t('common.backHome')" class="q-ml-sm" @click="handleGoHome" />
    </div>
    <div v-if="showDetails" class="error-details q-mt-lg text-left">
      <q-card bordered>
        <q-card-section>
          <div class="text-subtitle2 text-negative q-mb-sm">{{ $t('error.boundary.details') }}</div>
          <code class="text-caption">{{ error_message }}</code>
        </q-card-section>
      </q-card>
    </div>
    <q-btn flat dense class="q-mt-md" color="grey" :label="showDetails ? $t('errorBoundary.hideDetails') : $t('errorBoundary.showDetails')" @click="showDetails = !showDetails" />
  </div>
</template>

<script setup lang="ts">
/**
 * @file ErrorBoundary.vue
 * @description 全局错误边界组件 - 捕获子组件渲染错误
 * @date 2026-04-03
 */

import { ref, onErrorCaptured } from 'vue';
import { useRouter } from 'vue-router';
import { logger } from '@/utils/logger';

const router = useRouter();

/**
 * @brief 是否显示错误
 */
const hasError = ref(false);

/**
 * @brief 错误消息
 */
const error_message = ref('');

/**
 * @brief 是否显示详情
 */
const showDetails = ref(false);

/**
 * @brief 捕获子组件错误
 */
onErrorCaptured((err: Error) => {
  logger.error('【ErrorBoundary】捕获到错误:', err);
  // 诊断 (2026-08-10): 暴露到 window 便于 CDP 抓取
  (window as unknown as Record<string, unknown>).__lastBoundaryError = err.message || String(err);
  (window as unknown as Record<string, unknown>).__lastBoundaryStack = err.stack || '';
  hasError.value = true;
  error_message.value = err.message || String(err);
  return false;
});

/**
 * @brief 刷新页面
 */
function handleReload() {
  window.location.reload();
}

/**
 * @brief 返回首页
 */
function handleGoHome() {
  hasError.value = false;
  error_message.value = '';
  void router.push('/');
}
</script>

<style scoped>
.error-boundary {
  min-height: 60vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.error-details {
  width: 100%;
  max-width: 600px;
}

.error-details code {
  display: block;
  padding: 12px;
  background: #f5f5f5;
  border-radius: 4px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.body--dark .error-details code {
  background: #2d2d2d;
}
</style>

