<template>
  <div class="row q-col-gutter-md">
    <div class="col-12 col-md-8">
      <q-card flat bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-md">{{ $t('common.coreMetrics') }}</div>
          <div class="row q-col-gutter-md">
            <div class="col-6 col-sm-3" v-for="m in coreMetrics" :key="m.key">
              <q-card class="metric-card" flat bordered>
                <q-card-section class="text-center">
                  <div class="text-h5" :class="m.color">{{ m.value }}</div>
                  <div class="text-caption">{{ m.label }}</div>
                  <div class="text-caption text-grey">{{ m.desc }}</div>
                </q-card-section>
              </q-card>
            </div>
          </div>
        </q-card-section>
        <q-separator />
        <q-card-section>
          <div class="row items-center q-mb-sm">
            <div class="text-subtitle1">{{ $t('common.loadTime') }}</div>
          </div>
          <div class="row q-col-gutter-md">
            <div class="col-4" v-for="m in loadMetrics" :key="m.key">
              <div class="text-caption text-grey">{{ m.label }}</div>
              <div class="text-h6">{{ m.value }}</div>
              <q-linear-progress :value="m.progress" :color="m.color" class="q-mt-xs" />
            </div>
          </div>
        </q-card-section>
      </q-card>
    </div>
    <div class="col-12 col-md-4">
      <q-card flat bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-md">{{ $t('common.resourceUsage') }}</div>
          <div class="q-gutter-y-md" v-for="r in resourceItems" :key="r.key">
            <div>
              <div class="row items-center justify-between q-mb-xs">
                <div class="text-caption">{{ r.label }}</div>
                <div class="text-caption text-weight-bold">{{ r.value }}</div>
              </div>
              <q-linear-progress :value="r.progress" :color="r.color" size="10px" rounded />
            </div>
          </div>
        </q-card-section>
      </q-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
const props = defineProps<{
  metrics: { FCP: number; LCP: number; CLS: number; FID: number; DOMContentLoaded: number; pageLoadTime: number; loadComplete: number };
  apiStats: { avg_response_time: number; errorRate: number };
  resourceUsage: { memoryUsage: number; cpuUsage: number; networkUsage: number } | null;
  lcpColorClass: string;
  clsColorClass: string;
  memoryUsageColor: string;
  formatBytes: (b: number | undefined) => string;
}>();

const coreMetrics = computed(() => [
  { key: 'fcp', label: 'FCP', desc: '首次内容绘制', value: `${props.metrics.FCP}ms`, color: 'text-primary' },
  { key: 'lcp', label: 'LCP', desc: '最大内容绘制', value: `${props.metrics.LCP}ms`, color: props.lcpColorClass },
  { key: 'cls', label: 'CLS', desc: '布局偏移', value: props.metrics.CLS, color: props.clsColorClass },
  { key: 'fid', label: 'FID/INP', desc: '首次输入延迟', value: `${props.metrics.FID}ms`, color: 'text-primary' },
]);

const loadMetrics = computed(() => [
  { key: 'dom', label: 'DOM 构建', value: `${props.metrics.DOMContentLoaded}ms`, progress: props.metrics.DOMContentLoaded / 3000, color: 'info' },
  { key: 'page', label: '页面加载', value: `${props.metrics.pageLoadTime}ms`, progress: props.metrics.pageLoadTime / 5000, color: 'primary' },
  { key: 'complete', label: '完全加载', value: `${props.metrics.loadComplete}ms`, progress: props.metrics.loadComplete / 8000, color: 'secondary' },
]);

const resourceItems = computed(() => [
  { key: 'mem', label: '内存使用', value: props.formatBytes(props.resourceUsage?.memoryUsage ? props.resourceUsage.memoryUsage * 1024 * 1024 : undefined), progress: (props.resourceUsage?.memoryUsage || 0) / 100, color: props.memoryUsageColor },
  { key: 'cpu', label: 'CPU 使用率', value: `${props.resourceUsage?.cpuUsage || 0}%`, progress: (props.resourceUsage?.cpuUsage || 0) / 100, color: 'info' },
  { key: 'net', label: '网络流量', value: props.formatBytes(props.resourceUsage?.networkUsage), progress: Math.min((props.resourceUsage?.networkUsage || 0) / (1024 * 1024 * 10), 1), color: 'secondary' },
]);
</script>
