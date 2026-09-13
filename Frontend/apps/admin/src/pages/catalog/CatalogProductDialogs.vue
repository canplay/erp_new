<template>
  <!-- 新建/编辑商品 -->
  <q-dialog v-model="productDialog">
    <q-card style="min-width: 560px; max-width: 92vw">
      <q-card-section class="text-h6">{{
        editingProduct ? i18nT('raw.s76119e') : i18nT('raw.sb67d60')
      }}</q-card-section>
      <q-card-section class="row q-col-gutter-md">
        <q-input
          v-model="productForm.name"
          :label="i18nT('raw.sf43de3')"
          filled
          autofocus
          maxlength="200"
          counter
          class="col-12 col-sm-6"
          :rules="[(v) => !!v?.trim() || i18nT('raw.s6a17dd')]"
          lazy-rules
        />
        <q-input
          v-model="productForm.sku"
          label="SKU"
          filled
          maxlength="64"
          class="col-12 col-sm-6"
          :disable="!!editingProduct"
          :rules="editingProduct ? [] : [(v) => !!v?.trim() || i18nT('raw.s6a17dd')]"
          lazy-rules
        />
        <q-select
          v-model="productForm.brandId"
          :options="brandOptions"
          :label="i18nT('raw.sea8d20')"
          filled
          emit-value
          map-options
          class="col-12 col-sm-6"
          :rules="[(v) => !!v || i18nT('raw.s8fbabe')]"
        />
        <q-select
          v-model="productForm.categoryId"
          :options="categoryOptions"
          :label="i18nT('raw.s2f0afe')"
          filled
          emit-value
          map-options
          class="col-12 col-sm-6"
          :rules="[(v) => !!v || i18nT('raw.s8fbabe')]"
        />
        <template v-if="!editingProduct">
          <q-input
            v-model="productForm.priceAmount"
            :label="i18nT('raw.s1a584d')"
            filled
            type="number"
            min="0"
            step="0.01"
            class="col-12 col-sm-4"
          />
          <q-input
            v-model="productForm.priceCurrency"
            :label="i18nT('raw.sa43269')"
            filled
            maxlength="3"
            class="col-12 col-sm-4"
          />
          <q-input
            v-model="productForm.stock"
            :label="i18nT('raw.s9dbef4')"
            filled
            type="number"
            min="0"
            step="1"
            class="col-12 col-sm-4"
          />
        </template>
        <q-input
          v-model="productForm.description"
          :label="i18nT('raw.s9e58f1')"
          filled
          type="textarea"
          maxlength="4000"
          class="col-12"
        />
        <div v-if="editingProduct" class="col-12 row items-center">
          <span class="text-subtitle2">{{ i18nT('raw.sfbedde') }}</span>
          <q-space />
          <q-toggle
            v-model="productForm.isActive"
            :label="productForm.isActive ? i18nT('raw.sebf082') : i18nT('raw.s5797e7')"
            color="primary"
          />
        </div>
      </q-card-section>
      <q-card-actions align="right">
        <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
        <q-btn
          :label="i18nT('raw.sfe9512')"
          color="primary"
          :loading="productSaving"
          :disable="!productFormValid"
          @click="saveProduct"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>

  <!-- 删除商品 -->
  <q-dialog v-model="productDeleteDialog">
    <q-card style="min-width: 420px">
      <q-card-section class="row items-center">
        <q-icon name="warning" color="negative" size="28px" class="q-mr-sm" />
        <div class="text-h6">{{ i18nT('raw.s80b019') }}</div>
      </q-card-section>
      <q-card-section class="q-pt-none">
        {{ i18nT('raw.sdd0e60', { name: productToDelete?.name }) }}
      </q-card-section>
      <q-card-actions align="right">
        <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
        <q-btn
          :label="i18nT('raw.sbd7449')"
          color="negative"
          :loading="productDeleting"
          @click="confirmDeleteProduct"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>

  <!-- 商品详情（抽取为 CatalogProductDetailDialog） -->
  <CatalogProductDetailDialog
    ref="detailDialogRef"
    :brand-options="brandOptions"
    :category-options="categoryOptions"
    @error="emit('error', $event)"
  />
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { computed, ref } from 'vue';
import { useQuasar } from 'quasar';
import { catalogApi } from '@/api';
import CatalogProductDetailDialog from './CatalogProductDetailDialog.vue';

const $q = useQuasar();

function toast(type: 'positive' | 'negative', message: string) {
  $q.notify({ type, message });
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const _props = defineProps<{
  brandOptions: { label: string; value: string }[];
  categoryOptions: { label: string; value: string }[];
}>();

const emit = defineEmits<{
  (e: 'refresh'): void;
  (e: 'error', message: string): void;
}>();

const productDialog = ref(false);
const productSaving = ref(false);
const editingProduct = ref<ProductItem | null>(null);
const productForm = ref<{
  sku: string;
  name: string;
  description: string;
  brandId: string;
  categoryId: string;
  priceAmount: string;
  priceCurrency: string;
  stock: string;
  isActive: boolean;
}>({
  sku: '',
  name: '',
  description: '',
  brandId: '',
  categoryId: '',
  priceAmount: '0',
  priceCurrency: 'USD',
  stock: '0',
  isActive: true,
});

const productFormValid = computed(() => {
  const f = productForm.value;
  const nameOk = f.name.trim().length > 0;
  const brandOk = f.brandId.length > 0;
  const categoryOk = f.categoryId.length > 0;
  const skuOk = !!editingProduct.value || f.sku.trim().length > 0;
  const priceNum = Number.parseFloat(f.priceAmount);
  const stockNum = Number.parseInt(f.stock, 10);
  const priceOk = !Number.isNaN(priceNum) && priceNum >= 0;
  const stockOk = !Number.isNaN(stockNum) && stockNum >= 0;
  return nameOk && brandOk && categoryOk && skuOk && priceOk && stockOk;
});

function openProductCreate() {
  editingProduct.value = null;
  productForm.value = {
    sku: '',
    name: '',
    description: '',
    brandId: '',
    categoryId: '',
    priceAmount: '0',
    priceCurrency: 'USD',
    stock: '0',
    isActive: true,
  };
  productDialog.value = true;
}

function openProductEdit(p: ProductItem) {
  editingProduct.value = p;
  productForm.value = {
    sku: p.sku ?? '',
    name: p.name ?? '',
    description: p.description ?? '',
    brandId: p.brandId ?? '',
    categoryId: p.categoryId ?? '',
    priceAmount: String(p.price?.amount ?? 0),
    priceCurrency: p.price?.currency ?? 'USD',
    stock: String(p.stock ?? 0),
    isActive: p.isActive ?? true,
  };
  productDialog.value = true;
}

async function saveProduct() {
  if (!productFormValid.value) return;
  productSaving.value = true;
  try {
    if (editingProduct.value) {
      const payload: {
        name: string;
        brandId: string;
        categoryId: string;
        isActive: boolean;
        description?: string;
      } = {
        name: productForm.value.name.trim(),
        brandId: productForm.value.brandId,
        categoryId: productForm.value.categoryId,
        isActive: productForm.value.isActive,
      };
      if (productForm.value.description.trim())
        payload.description = productForm.value.description.trim();
      await catalogApi.updateProduct(editingProduct.value.id, payload);
      toast('positive', i18nT('raw.s3744bd'));
    } else {
      const payload: Record<string, unknown> = {
        sku: productForm.value.sku.trim().toUpperCase(),
        name: productForm.value.name.trim(),
        brandId: productForm.value.brandId,
        categoryId: productForm.value.categoryId,
        priceAmount: Number.parseFloat(productForm.value.priceAmount),
        priceCurrency: productForm.value.priceCurrency.trim().toUpperCase(),
        stock: Number.parseInt(productForm.value.stock, 10),
      };
      if (productForm.value.description.trim())
        payload.description = productForm.value.description.trim();
      await catalogApi.createProduct(payload);
      toast('positive', i18nT('raw.s6578d6'));
    }
    productDialog.value = false;
    emit('refresh');
  } catch (e) {
    emit(
      'error',
      e instanceof Error
        ? e.message
        : editingProduct.value
          ? i18nT('raw.se410d8')
          : i18nT('raw.sb62cb3'),
    );
  } finally {
    productSaving.value = false;
  }
}

const detailDialogRef = ref<InstanceType<typeof CatalogProductDetailDialog> | null>(null);

function openProductDetail(p: ProductItem) {
  detailDialogRef.value?.openProductDetail(p);
}

const productDeleteDialog = ref(false);
const productDeleting = ref(false);
const productToDelete = ref<ProductItem | null>(null);

function onProductDelete(row: ProductItem) {
  productToDelete.value = row;
  productDeleteDialog.value = true;
}

async function confirmDeleteProduct() {
  if (!productToDelete.value) return;
  productDeleting.value = true;
  try {
    await catalogApi.deleteProduct(productToDelete.value.id);
    toast('positive', i18nT('raw.s035ef6'));
    productDeleteDialog.value = false;
    emit('refresh');
  } catch (e) {
    emit('error', e instanceof Error ? e.message : i18nT('raw.s2324d0'));
  } finally {
    productDeleting.value = false;
  }
}

defineExpose({
  openProductCreate,
  openProductEdit,
  onProductDelete,
  openProductDetail,
});

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
</script>
