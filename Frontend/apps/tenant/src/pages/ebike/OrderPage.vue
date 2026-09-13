<template>
  <q-page class="q-pa-md">
    <div class="col-12 bg-grey-2 elevation-1" style="padding: 16px;">
      <div class="row">
        <div class="col-12">
          <h4 class="text-white mb-2">订单管理</h4>
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
                @click="queryOrder"
              />
            </div>
            <div class="col-4">
              <q-btn
                flat
                color="primary"
                :label="$t('ebike.export')"
                @click="exportOrder"
                class="mb-2"
              />
            </div>
          </div>

          <q-table
            :rows="filteredOrders"
            :columns="orderColumns"
            row-key="code"
            :loading="loading"
            flat
            bordered
            :pagination="{ rowsPerPage: 20 }"
          />
        </div>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, computed } from 'vue';
import { useQuasar, exportFile } from 'quasar';
import { useEbikeStore as useStore } from '@/stores/ebike';
import { httpClient as api } from '@/utils/alova';
import { parseCarResponse } from '@/utils/ebike';
import { orderColumns } from '@erp-new-frontend-monorepo/composables/src/orderColumns';;
import type { OrderItem } from '@/types/ebike';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
const $q = useQuasar();
const store = useStore();

// Data refs
const orders = ref<OrderItem[]>([]);

// Search
const searchText = ref('');

// Filters
const filters = ref({
  status: -1,
});

// Loading
const loading = ref(false);

// Computed
const filteredOrders = computed(() => {
  let result = [...orders.value];

  if (searchText.value) {
    const lower = searchText.value.toLowerCase();
    result = result.filter((o) =>
      (o.code || '').toLowerCase().includes(lower)
    );
  }

  if (filters.value.status !== -1) {
    result = result.filter((o) => o.status === filters.value.status);
  }

  return result;
});

// Watch
// 数据变化由 props 驱动, 无需手动同步

// Methods
async function loadOrderData(): Promise<void> {
  loading.value = true;
  try {
    const resp = await api.post(store.backend.private + '/order', {
      method: 'query',
      code: '',
      provide: '',
      status: -1,
      time: { start: '', end: '' },
    });

    // 修复 (2026-08-12 QA: P1-3) 解析 API 返回格式
    // 统一使用 parseCarResponse 处理所有可能的返回结构
    orders.value = (parseCarResponse(resp) as OrderItem[]) || [];
  } catch (error) {
    console.error('Failed to load order data:', error);
    $q.notify({
      type: 'negative',
      message: '加载订单数据失败',
    });
  } finally {
    loading.value = false;
  }
}

function queryOrder() {
  void loadOrderData();
}

function exportOrder() {
  if (orders.value.length === 0) {
    $q.notify({
      type: 'warning',
      message: '暂无数据可导出',
    });
    return;
  }

  const headers = ['识别码', '类型', '运营商', '状态', '速度', '时间', '警告', '备注'];
  const rows = orders.value.map((o: OrderItem) => [
    o.code ?? '',
    o.type ?? '',
    o.provide ?? '',
    o.status?.toString() ?? '',
    o.speed?.toString() ?? '',
    o.time ?? '',
    o.alert ?? '',
    o.remark ?? '',
  ]);
  const content = [headers.join(','), ...rows.map((r: string[]) => r.join(','))].join('\r\n');

  const status = exportFile('订单列表.xlsx', content, 'text/xlsx');
  if (status !== true) {
    $q.notify({ type: 'info', message: '正在准备下载，请稍后...' });
  }
}

// 修复 (2026-08-12 QA: P1-4) mounted 时自动调用 onSearch
onMounted(async () => {
  await loadOrderData();
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
