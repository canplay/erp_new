import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import * as lprApi from '@/api/lpr';
import type { PassRecord, PassRecordQueryParams } from '@/api/lpr';

export const useLprStore = defineStore('lpr', () => {
  const records = ref<PassRecord[]>([]);
  const currentRecord = ref<PassRecord | null>(null);
  const loading = ref(false);
  const total = ref(0);
  const page = ref(1);
  const page_size = ref(20);

  const todayEntryCount = computed(() => records.value.filter(r => r.direction === 'entry').length);
  const todayExitCount = computed(() => records.value.filter(r => r.direction === 'exit').length);
  const highConfidence = computed(() => records.value.filter(r => r.confidence >= 0.9).length);

  function getData(res: unknown): Record<string, unknown> {
    const r = res as Record<string, unknown>;
    return (r.data as Record<string, unknown>) ?? r;
  }

  async function fetchRecords(params?: PassRecordQueryParams) {
    loading.value = true;
    try {
      const res = await lprApi.listPassRecords(params);
      const data = getData(res);
      records.value = (data.list as PassRecord[]) ?? (data.records as PassRecord[]) ?? [];
      total.value = (data.total as number) ?? records.value.length;
      page.value = params?.page ?? 1;
      page_size.value = params?.page_size ?? 20;
    } finally {
      loading.value = false;
    }
  }

  async function fetchRecord(id: number) {
    loading.value = true;
    try {
      const res = await lprApi.getPassRecord(id);
      currentRecord.value = getData(res) as unknown as PassRecord;
    } finally {
      loading.value = false;
    }
  }

  async function fetchStats(params?: { park_code?: string; start_date?: string; end_date?: string }) {
    const res = await lprApi.getPassStats(params);
    return getData(res);
  }

  return {
    records, currentRecord, loading, total, page, page_size,
    todayEntryCount, todayExitCount, highConfidence,
    fetchRecords, fetchRecord, fetchStats,
  };
});
