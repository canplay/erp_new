<template>
  <q-dialog v-model="productDetailDialog">
    <q-card
      style="
        min-width: 560px;
        max-width: 92vw;
        display: flex;
        flex-direction: column;
        max-height: 85vh;
      "
    >
      <q-card-section class="row items-center">
        <div class="text-h6 ellipsis">{{ productDetail?.name || i18nT('raw.s04c773') }}</div>
        <q-space />
        <q-btn flat round dense icon="close" v-close-popup />
      </q-card-section>
      <q-separator />
      <q-card-section style="flex: 1 1 auto; min-height: 0; display: flex">
        <q-scroll-area class="col">
          <div v-if="productDetailLoading" class="text-grey-7 q-pa-sm">
            {{ i18nT('raw.s795a79') }}
          </div>
          <div v-else-if="productDetail">
            <div class="row items-center q-mb-md">
              <q-avatar size="72px" rounded>
                <img
                  v-if="productDetail.thumbnailUrl"
                  :src="productDetail.thumbnailUrl"
                  alt=""
                  loading="lazy"
                />
                <span v-else class="text-primary text-h6">{{ initial(productDetail.name) }}</span>
              </q-avatar>
              <div class="q-ml-md">
                <div class="text-h6">{{ productDetail.name }}</div>
                <code class="text-grey-7">{{ productDetail.sku }}</code>
              </div>
            </div>
            <div class="row q-col-gutter-y-md q-mb-md">
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.sea8d20') }}</div>
                <div class="text-body2">{{ brandNameOf(productDetail.brandId) }}</div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s2f0afe') }}</div>
                <div class="text-body2">{{ categoryNameOf(productDetail.categoryId) }}</div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s1a584d') }}</div>
                <div class="text-body2">
                  {{ formatMoney(productDetail.price?.amount, productDetail.price?.currency) }}
                </div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s9dbef4') }}</div>
                <div class="text-body2">{{ productDetail.stock ?? '—' }}</div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s9fb403') }}</div>
                <div class="text-body2">
                  <q-badge :color="productDetail.isActive ? 'green' : 'grey'" outline>
                    {{ productDetail.isActive ? i18nT('raw.sebf082') : i18nT('raw.s5797e7') }}
                  </q-badge>
                </div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.saf9956') }}</div>
                <div class="text-body2">{{ formatDate(productDetail.createdAtUtc) }}</div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.sfda53d') }}</div>
                <div class="text-body2">{{ formatDate(productDetail.updatedAtUtc) }}</div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s8384dd') }}</div>
                <div class="text-body2 ellipsis" :title="productDetail.id">
                  {{ productDetail.id }}
                </div>
              </div>
            </div>
            <div>
              <div class="text-subtitle2 q-mb-xs">{{ i18nT('raw.s9e58f1') }}</div>
              <p v-if="productDetail.description" class="text-body2" style="white-space: pre-wrap">
                {{ productDetail.description }}
              </p>
              <p v-else class="text-grey-6">{{ i18nT('raw.s977b1b') }}</p>
            </div>
          </div>
        </q-scroll-area>
      </q-card-section>
      <q-separator />
      <q-card-actions align="right">
        <q-btn :label="i18nT('raw.sdedda3')" v-close-popup />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { formatDate } from '@erp-new-frontend-monorepo/utils';
import { ref } from 'vue';

import { catalogApi } from '@/api';

export interface ProductItem {
  id: string;
  sku?: string;
  name: string;
  slug?: string;
  description?: string | null;
  brandId?: string;
  categoryId?: string;
  price?: { amount: number; currency: string };
  stock?: number;
  isActive?: boolean;
  thumbnailUrl?: string | null;
  createdAtUtc?: string;
  updatedAtUtc?: string | null;
}

const props = defineProps<{
  brandOptions: { label: string; value: string }[];
  categoryOptions: { label: string; value: string }[];
}>();

const emit = defineEmits<{
  (e: 'error', message: string): void;
}>();

function formatMoney(amount?: number, currency?: string) {
  if (amount === undefined || amount === null || Number.isNaN(Number(amount))) return '—';
  try {
    return new Intl.NumberFormat('zh-CN', {
      style: 'currency',
      currency: currency || 'USD',
    }).format(Number(amount));
  } catch {
    return `${currency || ''} ${amount}`.trim();
  }
}

function initial(name?: string) {
  const n = (name ?? '').trim();
  return n ? n.charAt(0).toUpperCase() : '·';
}

function brandNameOf(id?: string) {
  if (!id) return '—';
  return props.brandOptions.find((o) => o.value === id)?.label ?? '—';
}

function categoryNameOf(id?: string) {
  if (!id) return '—';
  return props.categoryOptions.find((o) => o.value === id)?.label ?? '—';
}

const productDetailDialog = ref(false);
const productDetailLoading = ref(false);
const productDetail = ref<ProductItem | null>(null);

async function openProductDetail(p: ProductItem) {
  productDetail.value = null;
  productDetailLoading.value = true;
  productDetailDialog.value = true;
  try {
    productDetail.value = (await catalogApi.product(p.id)) as ProductItem;
  } catch (e) {
    emit('error', e instanceof Error ? e.message : i18nT('raw.s59580f'));
  } finally {
    productDetailLoading.value = false;
  }
}

defineExpose({
  openProductDetail,
});
</script>
