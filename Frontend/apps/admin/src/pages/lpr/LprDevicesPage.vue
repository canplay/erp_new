<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">识别设备管理</div>
    <q-table
      :rows="rows"
      :columns="columns"
      row-key="id"
      flat bordered
      :loading="loading"
      :rows-number="rows?.length ?? 0"
    >
      <template #loading>
        <q-inner-loading showing color="primary" />
      </template>
      <template #body-cell-status="{ row }">
        <q-chip :color="row.status === 'online' ? 'positive' : 'negative'" text-color="white" dense>
          {{ row.status === 'online' ? '在线' : '离线' }}
        </q-chip>
      </template>
      <template #no-data>
        <div class="q-pa-lg text-center text-grey">暂无设备数据</div>
      </template>
    </q-table>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { LprDevice } from '@/api/lpr';
import { listLprDevices } from '@/api/lpr';
import { useQuasar } from 'quasar';

const $q = useQuasar();
const columns = [
  { name: 'id', label: '编号', field: 'id', align: 'left' as const },
  { name: 'name', label: '设备名称', field: 'name', align: 'left' as const },
  { name: 'sn', label: '序列号', field: 'sn', align: 'left' as const },
  { name: 'status', label: '状态', field: 'status', align: 'center' as const },
  { name: 'park_code', label: '车场', field: 'park_code', align: 'center' as const },
];
const rows = ref<LprDevice[]>([]);
const loading = ref(false);

async function fetchDevices() {
  loading.value = true;
  try {
    const res = await listLprDevices();
    const respData = res as { list?: LprDevice[]; data?: LprDevice[] | { list?: LprDevice[] } };
    const dataObj = respData.data as { list?: LprDevice[] } | undefined;
    rows.value = respData.list || dataObj?.list || (Array.isArray(respData.data) ? (respData.data) : []) || [];
  } catch {
    $q.notify({ type: 'negative', message: '加载设备列表失败' });
    rows.value = [];
  } finally {
    loading.value = false;
  }
}

onMounted(fetchDevices);
</script>
