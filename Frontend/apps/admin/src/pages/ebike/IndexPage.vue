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
              <h4 class="text-white mb-2">车辆管理</h4>
              <q-input
                v-model="searchText"
                outlined
                dense
                placeholder="搜索识别码/运营商"
                class="mb-2"
                @update:model-value="onSearchInput"
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
                    label="状态"
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
                    @click="throttledQuery"
                  />
                </div>
                <div class="col-4">
                  <q-btn
                    flat
                    color="primary"
                    :label="$t('ebike.export')"
                    @click="throttledExport"
                    class="mb-2"
                  />
                </div>
              </div>

              <q-table
                :rows="filteredCars"
                :columns="carColumns"
                row-key="code"
                :loading="loading"
                flat
                bordered
                :pagination="(pagination as any)"
              />
            </div>
          </div>
        </div>
      </template>

      <template v-slot:after>
        <div class="col-12 col-md-5 bg-grey-2 elevation-1" style="padding: 16px;">
          <MapComponent
            ref="mapComponentRef"
            :vehicles="cars"
            :polygons="polygons"
            :alerts="alerts"
          />
        </div>
      </template>
    </q-splitter>
  </q-page>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, computed, watch } from 'vue';
import { useQuasar, exportFile } from 'quasar';
import MapComponent from './MapComponent.vue';
import { useEbikeStore as useStore } from '@/stores/ebike';
import { httpClient as api } from '@/utils/alova';
import { parseCarResponse } from '@/utils/ebike';
import { carColumns } from '@/composables/useEbikeColumns';
import type { CarItem, StorageItem, AlertItem } from '@/types/ebike';

const $q = useQuasar();
const store = useStore();
const mapComponentRef = ref<{ setCenter?: (gps: { lat?: number; lng?: number }, zoom?: number) => void } | null>(null);
const splitter = ref({
  model: 100,
  show: true,
});

// Data refs
const cars = ref<CarItem[]>([]);
const polygons = ref<StorageItem[]>([]);
const alerts = ref<AlertItem[]>([]);

// Search
const searchText = ref('');

// Filters
const filters = ref({
  status: -1,
});

// 防抖搜索：300ms 内只触发最后一次请求
function onSearchInput(): void {
  store.debouncedSearch(() => {
    void loadCarData();
  }, 300);
}

// 节流查询/导出：300ms 内只执行一次
function throttledQuery(): void {
  store.throttledAction(() => {
    void loadCarData();
  });
}

function throttledExport(): void {
  store.throttledAction(() => {
    exportCar();
  });
}

// Loading
const loading = ref(false);

// Computed
const filteredCars = computed(() => {
  let result = [...cars.value];

  if (searchText.value) {
    const lower = searchText.value.toLowerCase();
    result = result.filter((c) =>
      (c.code || '').toLowerCase().includes(lower)
    );
  }

  if (filters.value.status !== -1) {
    result = result.filter((c) => c.status === filters.value.status);
  }

  return result;
});

// Watch (数据变化由 props 驱动 MapComponent 内部渲染, 无需手动同步)
watch(filters, () => {
  store.throttledAction(() => {
    void loadCarData();
  });
}, { deep: true });

// Methods
async function loadCarData(): Promise<void> {
  loading.value = true;
  try {
    const { limit, offset } = store.getPaginationParams();
    const resp = await api.post(store.backend.private + '/car', {
      method: 'query',
      code: '',
      provide: '',
      status: filters.value.status,
      time: { start: '', end: '' },
      limit,
      offset,
    });

    const data = parseCarResponse(resp) as CarItem[];
    cars.value = data || [];
    // 更新分页总数（假设后端返回中可能包含 total 字段，或默认按当前返回量估算）
    store.updatePagination(data?.length ?? 0);
  } catch (error) {
    console.error('Failed to load data:', error);
    $q.notify({ color: 'negative', message: '数据加载失败，请检查网络连接', icon: 'warning' });
  } finally {
    loading.value = false;
  }
}

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
    polygons.value = (parseCarResponse(resp) as StorageItem[]) || [];
  } catch (error) {
    console.error('Failed to load data:', error);
    $q.notify({ color: 'negative', message: '数据加载失败，请检查网络连接', icon: 'warning' });
  } finally {
    loading.value = false;
  }
}

async function loadAlertData(): Promise<void> {
  loading.value = true;
  try {
    // 修复 (2026-08-10): 后端无独立 /alert 端点, 通过 /car 的 method='alert' 查询告警车辆
    const resp = await api.post(store.backend.private + '/car', {
      method: 'alert',
      code: '',
      provide: '',
      status: -1,
      time: '',
      alert: '',
      remark: '',
    });

    // 修复 (2026-08-12 QA: P1-3) 解析 API 返回格式
    // 统一使用 parseCarResponse 处理所有可能的返回结构
    alerts.value = (parseCarResponse(resp) as AlertItem[]) || [];
  } catch (error) {
    console.error('Failed to load data:', error);
    $q.notify({ color: 'negative', message: '数据加载失败，请检查网络连接', icon: 'warning' });
  } finally {
    loading.value = false;
  }
}

function queryCar() {
  void loadCarData();
}

function exportCar() {
  if (cars.value.length === 0) {
    $q.notify({
      type: 'warning',
      message: '暂无数据可导出',
    });
    return;
  }

  const headers = ['识别码', '状态', '速度', '位置', '时间', '警告', '备注'];
  const rows = cars.value.map((c: CarItem) => [
    c.code ?? '',
    c.status?.toString() ?? '',
    c.speed?.toString() ?? '',
    JSON.stringify(c.gps) ?? '',
    JSON.stringify(c.time) ?? '',
    c.alert ?? '',
    c.remark ?? '',
  ]);
  const content = [headers.join(','), ...rows.map((r: string[]) => r.join(','))].join('\r\n');

  const status = exportFile('车辆列表.xlsx', content, 'text/xlsx');
  if (status !== true) {
    $q.notify({ type: 'info', message: '正在准备下载，请稍后...' });
  }
}

// 地图中心设为示例城市
onMounted(async () => {
  await loadCarData();
  await loadStorageData();
  await loadAlertData();

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
