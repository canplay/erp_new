<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <q-btn flat round dense icon="arrow_back" @click="router.back()" />
      <div class="text-h5 q-ml-sm">{{ device?.name || '设备详情' }}</div>
      <q-space />
      <q-btn flat color="primary" icon="picture_as_pdf" label="导出PDF" @click="exportPdf" />
    </div>

    <div v-if="device" class="row q-col-gutter-md">
      <div class="col-12 col-md-8">
        <q-card flat bordered class="erp-card">
          <q-card-section>
            <div class="text-subtitle1">基本信息</div>
            <q-list class="q-mt-sm">
              <q-item
                ><q-item-section>编号</q-item-section
                ><q-item-section side>{{ device.code }}</q-item-section></q-item
              >
              <q-item
                ><q-item-section>类型</q-item-section
                ><q-item-section side>{{ device.type }}</q-item-section></q-item
              >
              <q-item
                ><q-item-section>位置</q-item-section
                ><q-item-section side>{{ device.location }}</q-item-section></q-item
              >
              <q-item
                ><q-item-section>状态</q-item-section
                ><q-item-section side
                  ><q-badge :color="device.status === 'Active' ? 'green' : 'grey'">{{
                    device.status
                  }}</q-badge></q-item-section
                ></q-item
              >
              <q-item
                ><q-item-section>健康度</q-item-section
                ><q-item-section side
                  ><HealthScoreBadge
                    :score="device.healthScore"
                    :level="device.healthLevel" /></q-item-section
              ></q-item>
            </q-list>
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-md-4">
        <q-card flat bordered class="erp-card">
          <q-card-section class="text-center">
            <DeviceQrcode :device-id="device.id" :payload="device.qrcode" :size="160" />
          </q-card-section>
        </q-card>
      </div>

      <div class="col-12">
        <q-tabs v-model="tab" dense class="text-grey q-mt-md q-mb-sm" active-color="primary">
          <q-tab name="inspections" label="检验记录" />
          <q-tab name="repairs" label="维修记录" />
          <q-tab name="spare-parts" label="备件" />
        </q-tabs>

        <q-tab-panels v-model="tab" animated>
          <q-tab-panel name="inspections">
            <PageTable
              :rows="inspections.items"
              :columns="inspectionColumns"
              :loading="loadingInspections"
              :total="inspections.total"
              @reload="loadInspections"
            />
          </q-tab-panel>
          <q-tab-panel name="repairs">
            <PageTable
              :rows="repairs"
              :columns="repairColumns"
              :loading="loadingRepairs"
              :total="repairs.length"
              @reload="loadRepairs"
            />
          </q-tab-panel>
          <q-tab-panel name="spare-parts">
            <PageTable
              :rows="spareParts"
              :columns="sparePartColumns"
              :loading="loadingSpareParts"
              :total="spareParts.length"
              @reload="loadSpareParts"
            />
          </q-tab-panel>
        </q-tab-panels>
      </div>
    </div>

    <EmptyState v-else-if="!loading" icon="error" title="设备不存在" />
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { deviceApi } from '@erp-new-frontend-monorepo/api';
import type { Device, Inspection, Repair, SparePart } from '@erp-new-frontend-monorepo/types';
import {
  PageTable,
  HealthScoreBadge,
  DeviceQrcode,
  EmptyState,
} from '@erp-new-frontend-monorepo/components';

const route = useRoute();
const router = useRouter();
const device = ref<Device | null>(null);
const loading = ref(true);
const tab = ref('inspections');

const inspections = ref<{ total: number; items: Inspection[] }>({ total: 0, items: [] });
const repairs = ref<Repair[]>([]);
const spareParts = ref<SparePart[]>([]);
const loadingInspections = ref(false);
const loadingRepairs = ref(false);
const loadingSpareParts = ref(false);

const inspectionColumns = [
  { name: 'code', label: '检验编号', field: 'code' },
  { name: 'type', label: '检验类型', field: 'type' },
  { name: 'result', label: '结果', field: 'result' },
  { name: 'date', label: '检验日期', field: 'inspectionDate' },
];
const repairColumns = [
  { name: 'title', label: '维修项目', field: 'title' },
  { name: 'cost', label: '费用', field: 'cost' },
  { name: 'status', label: '状态', field: 'status' },
];
const sparePartColumns = [
  { name: 'name', label: '备件名称', field: 'name' },
  { name: 'stock', label: '库存', field: 'currentStock' },
];

async function loadInspections() {
  if (!device.value) return;
  loadingInspections.value = true;
  try {
    inspections.value = await deviceApi.inspectionsPaged(device.value.id);
  } finally {
    loadingInspections.value = false;
  }
}
async function loadRepairs() {
  if (!device.value) return;
  loadingRepairs.value = true;
  try {
    repairs.value = await deviceApi.repairs(device.value.id);
  } finally {
    loadingRepairs.value = false;
  }
}
async function loadSpareParts() {
  if (!device.value) return;
  loadingSpareParts.value = true;
  try {
    spareParts.value = await deviceApi.spareParts(device.value.id);
  } finally {
    loadingSpareParts.value = false;
  }
}
function exportPdf() {
  if (device.value) window.open(`/api/v1/device/${device.value.id}/lifecycle-report.pdf`, '_blank');
}

const id = String((route.params as { id?: string }).id ?? '');

onMounted(async () => {
  try {
    device.value = await deviceApi.detail(id);
  } finally {
    loading.value = false;
  }
  if (device.value) {
    void loadInspections();
    void loadRepairs();
    void loadSpareParts();
  }
});
</script>
