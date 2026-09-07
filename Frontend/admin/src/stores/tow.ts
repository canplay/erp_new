import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import * as towApi from '@/api/tow';
import type { TowCar, TowCarQueryParams, DictItem } from '@/api/tow';

export const useTowStore = defineStore('tow', () => {
  const cars = ref<TowCar[]>([]);
  const currentCar = ref<TowCar | null>(null);
  const loading = ref(false);
  const total = ref(0);
  const page = ref(1);
  const page_size = ref(20);

  const carTypes = ref<DictItem[]>([]);
  const carColors = ref<DictItem[]>([]);
  const dcCauses = ref<DictItem[]>([]);

  const activeCars = computed(() => cars.value.filter(c => !c.delete));
  const carTypeOptions = computed(() => carTypes.value.map(d => ({ label: d.name, value: d.value })));
  const carColorOptions = computed(() => carColors.value.map(d => ({ label: d.name, value: d.value })));

  function getData(res: unknown): Record<string, unknown> {
    const r = res as Record<string, unknown>;
    return (r.data as Record<string, unknown>) ?? r;
  }

  function getDictList(res: unknown): DictItem[] {
    const data = getData(res);
    const list = (data.list as DictItem[] | undefined) ?? (data.items as DictItem[] | undefined);
    return Array.isArray(list) ? list : [];
  }

  async function fetchCars(params?: TowCarQueryParams) {
    loading.value = true;
    try {
      const res = await towApi.listTowCars(params);
      const data = getData(res);
      cars.value = (data.list as TowCar[]) ?? (data.cars as TowCar[]) ?? [];
      total.value = (data.total as number) ?? cars.value.length;
      page.value = params?.page ?? 1;
      page_size.value = params?.page_size ?? 20;
    } finally {
      loading.value = false;
    }
  }

  async function fetchCar(id: number) {
    loading.value = true;
    try {
      const res = await towApi.getTowCar(id);
      currentCar.value = getData(res) as unknown as TowCar;
    } finally {
      loading.value = false;
    }
  }

  async function fetchDicts() {
    const [types, colors, causes] = await Promise.all([
      towApi.listCarTypes(),
      towApi.listCarColors(),
      towApi.listDcCauses(),
    ]);
    carTypes.value = getDictList(types);
    carColors.value = getDictList(colors);
    dcCauses.value = getDictList(causes);
  }

  return {
    cars, currentCar, loading, total, page, page_size,
    carTypes, carColors, dcCauses,
    activeCars, carTypeOptions, carColorOptions,
    fetchCars, fetchCar, fetchDicts,
  };
});
