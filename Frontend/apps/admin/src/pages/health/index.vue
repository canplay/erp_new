<template>
  <q-page class="erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.s7120fa') }}</div>

    <div class="row q-col-gutter-md">
      <!-- 存活探针 -->
      <div class="col-12 col-md-6">
        <q-card>
          <q-card-section class="row items-center">
            <div class="text-subtitle1">{{ i18nT('raw.s421fdc') }}</div>
            <q-space />
            <q-badge :color="live.statusColor">{{ live.text }}</q-badge>
          </q-card-section>
          <q-card-section class="text-caption text-grey">
            进程存活检查（不依赖外部组件）
          </q-card-section>
        </q-card>
      </div>

      <!-- 就绪探针 -->
      <div class="col-12 col-md-6">
        <q-card>
          <q-card-section class="row items-center">
            <div class="text-subtitle1">{{ i18nT('raw.s4f4233') }}</div>
            <q-space />
            <q-badge :color="ready.statusColor">{{ ready.text }}</q-badge>
          </q-card-section>
          <q-card-section>
            <div v-if="ready.error" class="text-negative text-caption q-mb-sm">
              请求失败：{{ ready.error }}
            </div>
            <div v-else-if="ready.checks.length === 0" class="text-grey text-caption">无检查项</div>
            <q-list v-else separator>
              <q-item v-for="c in ready.checks" :key="c.name">
                <q-item-section>
                  <q-item-label>{{ c.name }}</q-item-label>
                  <q-item-label caption v-if="c.description">{{ c.description }}</q-item-label>
                </q-item-section>
                <q-item-section side>
                  <div class="row items-center q-gutter-sm">
                    <q-badge :color="entryStatusColor(c.status)">{{ c.status }}</q-badge>
                    <span class="text-caption text-grey">{{ c.durationMs.toFixed(1) }}ms</span>
                  </div>
                </q-item-section>
              </q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <div class="q-mt-md">
      <q-btn
        :label="i18nT('raw.sb7914e')"
        icon="refresh"
        color="primary"
        outline
        :loading="loading"
        @click="load"
        :title="'刷新'"
      />
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { reactive, ref, onMounted } from 'vue';
import { healthApi, type HealthResult } from '@/api';

interface ProbeState {
  text: string;
  statusColor: string;
  error: string;
  checks: { name: string; status: string; description?: string | null; durationMs: number }[];
}

function entryStatusColor(status: string) {
  const s = (status || '').toLowerCase();
  if (s === 'healthy') return 'green';
  if (s === 'unhealthy') return 'red';
  if (s === 'degraded') return 'orange';
  return 'grey';
}

function mapProbe(res: HealthResult | null): Omit<ProbeState, 'error'> {
  const status = res?.status || '';
  return {
    text: status || i18nT('raw.s376ddb'),
    statusColor: entryStatusColor(status),
    checks: (res?.results || []).map((c) => ({
      name: c.name,
      status: c.status,
      description: c.description ?? null,
      durationMs: c.durationMs,
    })),
  };
}

const live = reactive<ProbeState>({
  text: i18nT('raw.scb561b'),
  statusColor: 'grey',
  error: '',
  checks: [],
});
const ready = reactive<ProbeState>({
  text: i18nT('raw.scb561b'),
  statusColor: 'grey',
  error: '',
  checks: [],
});
const loading = ref(false);

function errorText(reason: unknown): string {
  if (reason instanceof Error) return reason.message;
  if (reason && typeof reason === 'object' && 'message' in reason) {
    return typeof (reason as { message?: unknown }).message === 'string'
      ? (reason as { message: string }).message
      : '';
  }
  return '';
}

async function fetchProbe(fn: () => Promise<HealthResult>): Promise<HealthResult> {
  try {
    return await fn();
  } catch (e) {
    // /health/ready 不健康时返回 503 但带完整 payload，此处兼容
    const err = e as { data?: unknown };
    if (err?.data && typeof err.data === 'object' && 'status' in err.data) {
      return err.data as HealthResult;
    }
    throw e;
  }
}

async function load() {
  loading.value = true;
  try {
    const [readyRes, liveRes] = await Promise.allSettled([
      fetchProbe(() => healthApi.ready()),
      fetchProbe(() => healthApi.live()),
    ]);

    if (readyRes.status === 'fulfilled') {
      Object.assign(ready, mapProbe(readyRes.value), { error: '' });
    } else {
      Object.assign(
        ready,
        { text: i18nT('raw.sc686f8'), statusColor: 'red', checks: [] },
        { error: errorText(readyRes.reason) || i18nT('raw.s6b6bf2') },
      );
    }

    if (liveRes.status === 'fulfilled') {
      Object.assign(live, mapProbe(liveRes.value), { error: '' });
    } else {
      Object.assign(
        live,
        { text: i18nT('raw.sc686f8'), statusColor: 'red', checks: [] },
        { error: errorText(liveRes.reason) || i18nT('raw.s9e7d31') },
      );
    }
  } finally {
    loading.value = false;
  }
}

onMounted(load);
</script>
