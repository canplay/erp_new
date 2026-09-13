<template>
  <div>
    <div class="row items-center q-mb-md">
      <div class="text-subtitle1">{{ i18nT('raw.s651231') }}</div>
      <span class="text-caption text-grey-7 q-ml-sm">{{
        i18nT('raw.s508a03', { count: productTotal })
      }}</span>
      <q-space />
      <q-btn
        color="primary"
        icon="add"
        :label="i18nT('raw.sb67d60')"
        @click="dialogs.openProductCreate()"
      />
    </div>

    <q-banner v-if="productError" type="negative" rounded class="q-mb-md">
      {{ productError }}
      <template #action>
        <q-btn flat color="white" :label="i18nT('raw.sdedda3')" @click="productError = ''" />
      </template>
    </q-banner>

    <CatalogProductFilters
      v-model:search="productSearch"
      v-model:brandFilter="brandFilter"
      v-model:categoryFilter="categoryFilter"
      :brandOptions="brandOptions"
      :categoryOptions="categoryOptions"
      @search="onProductSearch"
    />

    <CatalogProductTable
      :rows="productRows"
      :columns="productColumns"
      :loading="productLoading"
      :total="productTotal"
      :brandOptions="brandOptions"
      :categoryOptions="categoryOptions"
      @request="onProductRequest"
      @view="dialogs.openProductDetail"
      @edit="dialogs.openProductEdit"
      @delete="dialogs.onProductDelete"
    />

    <CatalogProductPagination
      :page="productPagination.page"
      :rowsPerPage="productPagination.rowsPerPage"
      :total="productTotal"
      @request="onProductRequest"
    />

    <CatalogProductDialogs
      ref="dialogsRef"
      :brandOptions="brandOptions"
      :categoryOptions="categoryOptions"
      @refresh="loadProducts()"
      @error="productError = $event"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { onMounted, ref } from 'vue';
import { catalogApi } from '@/api';
import CatalogProductFilters from './CatalogProductFilters.vue';
import CatalogProductTable from './CatalogProductTable.vue';
import CatalogProductPagination from './CatalogProductPagination.vue';
import CatalogProductDialogs from './CatalogProductDialogs.vue';
import type { ProductItem } from './CatalogProductDialogs.vue';

const productRows = ref<ProductItem[]>([]);
const productLoading = ref(false);
const productError = ref('');
const productTotal = ref(0);
const productSearch = ref('');
const brandFilter = ref<string | null>(null);
const categoryFilter = ref<string | null>(null);
const brandOptions = ref<{ label: string; value: string }[]>([]);
const categoryOptions = ref<{ label: string; value: string }[]>([]);
const productPagination = ref({ page: 1, rowsPerPage: 20, rowsNumber: 0 });

const productColumns = [
  { name: 'name', label: i18nT('raw.sf43de3'), field: 'name', align: 'left' as const },
  { name: 'sku', label: 'SKU', field: 'sku', align: 'left' as const },
  { name: 'brand', label: i18nT('raw.sea8d20'), field: 'brand', align: 'left' as const },
  { name: 'category', label: i18nT('raw.s2f0afe'), field: 'category', align: 'left' as const },
  { name: 'price', label: i18nT('raw.s1a584d'), field: 'price', align: 'right' as const },
  { name: 'stock', label: i18nT('raw.s9dbef4'), field: 'stock', align: 'center' as const },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions', align: 'right' as const },
];

let productOptionsLoaded = false;
async function ensureProductOptions() {
  if (productOptionsLoaded) return;
  productOptionsLoaded = true;
  try {
    const [brands, cats] = await Promise.all([
      catalogApi.brands({ page: 1, pageSize: 200 }) as Promise<{
        total: number;
        items: { id: string; name: string }[];
      }>,
      catalogApi.categories({ page: 1, pageSize: 200 }) as Promise<{
        total: number;
        items: { id: string; name: string }[];
      }>,
    ]);
    brandOptions.value = (brands.items ?? []).map((b) => ({ label: b.name, value: b.id }));
    categoryOptions.value = (cats.items ?? []).map((c) => ({ label: c.name, value: c.id }));
  } catch {
    productOptionsLoaded = false;
  }
}

async function loadProducts(p?: { page: number; rowsPerPage: number }) {
  const pg = p ?? productPagination.value;
  productLoading.value = true;
  try {
    const params: {
      page: number;
      pageSize: number;
      search?: string;
      brandId?: string;
      categoryId?: string;
    } = {
      page: pg.page,
      pageSize: pg.rowsPerPage,
    };
    if (productSearch.value.trim()) params.search = productSearch.value.trim();
    if (brandFilter.value) params.brandId = brandFilter.value;
    if (categoryFilter.value) params.categoryId = categoryFilter.value;
    const res = (await catalogApi.products(params)) as { total: number; items: ProductItem[] };
    productRows.value = res.items ?? [];
    productTotal.value = res.total ?? productRows.value.length;
    productPagination.value = {
      page: pg.page,
      rowsPerPage: pg.rowsPerPage,
      rowsNumber: res.total ?? productRows.value.length,
    };
  } catch (e) {
    productError.value = e instanceof Error ? e.message : i18nT('raw.sd5e9d4');
  } finally {
    productLoading.value = false;
  }
}

function onProductRequest(payload: { page: number; rowsPerPage: number }) {
  void loadProducts(payload);
}

function onProductSearch() {
  void loadProducts({ page: 1, rowsPerPage: productPagination.value.rowsPerPage });
}

const dialogsRef = ref<InstanceType<typeof CatalogProductDialogs> | null>(null);
const dialogs = {
  openProductCreate() {
    dialogsRef.value?.openProductCreate();
  },
  openProductEdit(p: ProductItem) {
    dialogsRef.value?.openProductEdit(p);
  },
  onProductDelete(p: ProductItem) {
    dialogsRef.value?.onProductDelete(p);
  },
  openProductDetail(p: ProductItem) {
    dialogsRef.value?.openProductDetail(p);
  },
};

onMounted(() => {
  void ensureProductOptions();
  void loadProducts();
});
</script>
