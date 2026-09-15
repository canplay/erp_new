<template>
  <q-page class="q-pa-md">
    <q-splitter
      v-model="splitter.model"
      :limits="[50, 99]"
      :disable="!splitter.show"
    >
      <template v-slot:before>
        <div class="col-12 col-md-7 bg-grey-2 elevation-1" style="padding: 16px;">
          <div class="row">
            <div class="col-12">
              <h4 class="text-white mb-2">停放区管理</h4>
              <q-input
                v-model="searchText"
                outlined
                dense
                placeholder="搜索识别码/运营商"
                class="mb-2"
              >
                <template #prepend>
                  <q-icon name="search" />
                </template>
              </q-input>

              <div class="row no-gutters mb-2">
                <div class="col-4">
                  <q-select
                    v-model="filters.status"
                    :options="[0, 1, 2, 3]"
                    :label="$t('common.status')"
                    outlined
                    dense
                    clearable
                  />
                </div>
                <div class="col-4">
                  <q-btn
                    flat
                    color="primary"
                    :label="$t('ebike.query')"
                    @click="queryStorage"
                  />
                </div>
                <div class="col-4">
                  <q-btn
                    flat
                    color="primary"
                    :label="$t('ebike.export')"
                    @click="exportStorage"
                    class="mb-2"
                  />
                </div>
              </div>

              <q-table
                :rows="filteredStorages"
                :columns="storageColumns"
                row-key="code"
                :loading="loading"
                flat
                bordered
                :pagination="{ rowsPerPage: 20 }"
              />
            </div>
          </div>
        </div>
      </template>

      <template v-slot:after>
        <div class="col-12 col-md-5 bg-grey-2 elevation-1" style="padding: 16px;">
          <MapComponent
            ref="mapComponentRef"
            :vehicles="[]"
            :polygons="storages"
            :alerts="[]"
          />
        </div>
      </template>
    </q-splitter>
  </q-page>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, computed } from 'vue';
import { useQuasar, exportFile } from 'quasar';
import MapComponent from './MapComponent.vue';
import { useEbikeStore as useStore } from '@/stores/ebike';
import { httpClient as api } from '@/utils/alova';
import { parseCarResponse } from '@/utils/ebike';
import { storageColumns } from '@erp-new-frontend-monorepo/composables/src/useEbikeColumns';
import type { StorageItem } from '@/types/ebike';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
const $q = useQuasar();
const store = useStore();
const mapComponentRef = ref<{ setCenter?: (gps: { lat?: number; lng?: number }, zoom?: number) => void } | null>(null);
const splitter = ref({
  model: 100,
  show: true,
});

// Data refs
const storages = ref<StorageItem[]>([]);

// Search
const searchText = ref('');

// Filters
const filters = ref({
  status: -1,
});

// Loading
const loading = ref(false);

// Computed
const filteredStorages = computed(() => {
  let result = [...storages.value];

  if (searchText.value) {
    const lower = searchText.value.toLowerCase();
    result = result.filter((s) =>
      (s.code || '').toLowerCase().includes(lower)
    );
  }

  if (filters.value.status !== -1) {
    result = result.filter((s) => s.status === filters.value.status);
  }

  return result;
});

// Watch
// 数据变化由 props 驱动 MapComponent 内部渲染, 无需手动同步

// Methods
async function loadStorageData(): Promise<void> {
  loading.value = true;
  try {
    const resp = await api.post(store.backend.private + '/storage', {
      method: 'query',
      code: '',
      provide: '',
      status: -1,
      time: { start: '', end: '' },
    });

    // 修复 (2026-08-12 QA: P1-3) 解析 API 返回格式
    // 统一使用 parseCarResponse 处理所有可能的返回结构
    storages.value = (parseCarResponse(resp) as StorageItem[]) || [];
  } catch (error) {
    console.error('Failed to load storage data:', error);
    $q.notify({
      type: 'negative',
      message: '加载停放区数据失败',
    });
  } finally {
    loading.value = false;
  }
}

function queryStorage() {
  void loadStorageData();
}

function exportStorage() {
  if (storages.value.length === 0) {
    $q.notify({
      type: 'warning',
      message: '暂无数据可导出',
    });
    return;
  }

  const headers = ['识别码', '运营商', '状态', '车辆总数', '当前数量', '容量', '状态', '位置'];
  const rows = storages.value.map((s: StorageItem) => [
    s.code ?? '',
    s.provide ?? '',
    s.status?.toString() ?? '',
    s.sum?.toString() ?? '',
    s.cur?.toString() ?? '',
    s.cur?.toString() ?? '',
    s.status?.toString() ?? '',
    s.gps ? `${s.gps.lng ?? ''},${s.gps.lat ?? ''}` : '',
  ]);
  const content = [headers.join(','), ...rows.map((r: string[]) => r.join(','))].join('\r\n');

  const status = exportFile('停放区列表.xlsx', content, 'text/xlsx');
  if (status !== true) {
    $q.notify({ type: 'info', message: '正在准备下载，请稍后...' });
  }
}

// 修复 (2026-08-12 QA: P1-4) mounted 时自动调用 onSearch
onMounted(async () => {
  await loadStorageData();

  // 设置地图中心到示例位置
  if (mapComponentRef.value?.setCenter) {
    mapComponentRef.value.setCenter({ lat: 23.0000, lng: 104.0000 }, 13);
  }
});

onBeforeUnmount(() => {
  // Cleanup if needed
});
</script>

@import url('/src/styles/ebike-theme.css');

<style scoped>
.bg-grey-9 {
  background-color: var(--ebike-bg-primary);
}

.q-table th {
  font-weight: 600;
  color: var(--ebike-text-secondary);
}

.q-table td {
  color: var(--ebike-text-primary);
}

.q-table tbody tr:hover {
  background-color: var(--ebike-bg-hover) !important;
}
</style>
