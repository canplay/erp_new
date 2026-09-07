import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import * as ctpApi from '@/api/ctp';
import type { CtpDevice, DeviceQueryParams, LockControlRequest } from '@/api/ctp';

export const useCtpStore = defineStore('ctp', () => {
  const devices = ref<CtpDevice[]>([]);
  const currentDevice = ref<CtpDevice | null>(null);
  const loading = ref(false);
  const total = ref(0);
  const page = ref(1);
  const page_size = ref(20);

  const deviceCount = computed(() => devices.value.length);
  const onlineDevices = computed(() => devices.value.filter(d => d.status === 'Locked' || d.status === 'Unlocked'));
  const offlineDevices = computed(() => devices.value.filter(d => d.status === 'Offline'));

  function getData(res: unknown): Record<string, unknown> {
    const r = res as Record<string, unknown>;
    return (r.data as Record<string, unknown>) ?? r;
  }

  async function fetchDevices(params?: DeviceQueryParams) {
    loading.value = true;
    try {
      const res = await ctpApi.listDevices(params);
      const data = getData(res);
      devices.value = (data.devices as any) ?? (data.list as any) ?? [];
      total.value = (data.total as any) ?? devices.value.length;
      page.value = params?.page ?? 1;
      page_size.value = params?.page_size ?? 20;
    } finally {
      loading.value = false;
    }
  }

  async function fetchDevice(deviceNo: string) {
    loading.value = true;
    try {
      const res = await ctpApi.getDevice(deviceNo);
      currentDevice.value = getData(res) as unknown as CtpDevice;
    } finally {
      loading.value = false;
    }
  }

  async function sendControl(cmd: LockControlRequest) {
    const res = await ctpApi.controlLock(cmd);
    return getData(res);
  }

  return {
    devices, currentDevice, loading, total, page, page_size,
    deviceCount, onlineDevices, offlineDevices,
    fetchDevices, fetchDevice, sendControl,
  };
});
