import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import * as xltApi from '@/api/xlt';
import type { VehicleEvent, XltDeviceInfo } from '@/api/xlt';

export const useXltStore = defineStore('xlt', () => {
  const parkingVehicles = ref<Record<string, unknown>[]>([]);
  const records = ref<Record<string, unknown>[]>([]);
  const devices = ref<XltDeviceInfo[]>([]);
  const loading = ref(false);
  const total = ref(0);

  const currentParking = computed(() => parkingVehicles.value.length);
  const todayEntryCount = computed(() => 0);
  const todayExitCount = computed(() => 0);

  function getData(res: unknown): Record<string, unknown> {
    const r = res as Record<string, unknown>;
    return (r.data as Record<string, unknown>) ?? r;
  }

  async function fetchParkingVehicles(parkCode: string, plateNo?: string) {
    loading.value = true;
    try {
      const res = await xltApi.getParkingVehicle(parkCode, plateNo ?? '');
      parkingVehicles.value = [getData(res)];
    } finally {
      loading.value = false;
    }
  }

  async function fetchRecords(params?: Record<string, unknown>) {
    loading.value = true;
    try {
      const p = params as { park_code?: string; plate_no?: string; page?: number; page_size?: number } | undefined;
      const res = await xltApi.listParkingRecords(p);
      const data = getData(res);
      records.value = (data.list as Record<string, unknown>[]) ?? (data.records as Record<string, unknown>[]) ?? [];
      total.value = (data.total as number) ?? records.value.length;
    } finally {
      loading.value = false;
    }
  }

  async function fetchDevices() {
    loading.value = true;
    try {
      const res = await xltApi.listXltDevices();
      const data = getData(res);
      devices.value = (data.devices as XltDeviceInfo[]) ?? (data.list as XltDeviceInfo[]) ?? [];
    } finally {
      loading.value = false;
    }
  }

  async function entry(event: VehicleEvent) { return getData(await xltApi.vehicleEntry(event)); }
  async function exit(event: VehicleEvent) { return getData(await xltApi.vehicleExit(event)); }

  return {
    parkingVehicles, records, devices, loading, total,
    currentParking, todayEntryCount, todayExitCount,
    fetchParkingVehicles, fetchRecords, fetchDevices,
    entry, exit,
  };
});
