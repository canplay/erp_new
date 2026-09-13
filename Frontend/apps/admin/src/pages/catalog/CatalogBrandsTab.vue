<template>
  <div>
    <div class="row items-center q-mb-md">
      <div class="text-subtitle1">{{ i18nT('raw.sea8d20') }}</div>
      <span class="text-caption text-grey-7 q-ml-sm">{{
        i18nT('raw.s508a01', { count: brandTotal })
      }}</span>
      <q-space />
      <q-btn color="primary" icon="add" :label="i18nT('raw.s39e271')" @click="openBrandCreate" />
    </div>

    <q-banner v-if="brandError" type="negative" rounded class="q-mb-md">
      {{ brandError }}
      <template #action>
        <q-btn flat color="white" :label="i18nT('raw.sdedda3')" @click="brandError = ''" />
      </template>
    </q-banner>

    <q-input
      v-model="brandSearch"
      :label="i18nT('raw.s0a23ef')"
      dense
      outlined
      clearable
      debounce="300"
      class="q-mb-md"
      style="max-width: 360px"
      @update:model-value="onBrandSearch"
    >
      <template #prepend>
        <q-icon name="search" />
      </template>
    </q-input>

    <PageTable
      v-if="brandLoading || brandRows.length > 0"
      :dense="$q.screen.lt.md"
      :rows="brandRows"
      :columns="brandColumns"
      :loading="brandLoading"
      :total="brandTotal"
      flat
      bordered
      @request="onBrandRequest"
    >
      <template #body-cell-description="props">
        <q-td class="ellipsis" style="max-width: 320px">
          <span :title="props.row.description || ''">{{ props.row.description || '—' }}</span>
        </q-td>
      </template>
      <template #body-cell-logo="props">
        <q-td>
          <q-avatar v-if="props.row.logoUrl" size="36px" rounded>
            <img :src="props.row.logoUrl" alt="" loading="lazy" />
          </q-avatar>
          <q-avatar v-else size="36px" rounded color="primary" text-color="white" font-size="14px">
            {{ initial(props.row.name) }}
          </q-avatar>
        </q-td>
      </template>
      <template #body-cell-createdAtUtc="props">
        <q-td>{{ formatDate(props.row.createdAtUtc) }}</q-td>
      </template>
      <template #body-cell-actions="props">
        <q-td>
          <q-btn
            flat
            dense
            round
            icon="edit"
            size="sm"
            @click="openBrandEdit(props.row)"
            :title="i18nT('raw.s207008')"
          />
          <q-btn
            flat
            dense
            round
            icon="delete"
            size="sm"
            text-color="negative"
            @click="
              brandToDelete = props.row;
              brandDeleteDialog = true;
            "
            :title="i18nT('raw.sbd7449')"
          />
        </q-td>
      </template>
    </PageTable>
    <EmptyState
      v-else
      :title="i18nT('raw.sdfaaef')"
      :hint="i18nT('raw.s855529')"
      icon="local_offer"
    />

    <!-- 新建/编辑品牌 -->
    <q-dialog v-model="brandDialog">
      <q-card style="min-width: 480px; max-width: 90vw">
        <q-card-section class="text-h6">{{
          editingBrand ? i18nT('raw.sf98e8f') : i18nT('raw.s39e271')
        }}</q-card-section>
        <q-card-section>
          <q-input
            v-model="brandForm.name"
            :label="i18nT('raw.sf43de3')"
            filled
            autofocus
            maxlength="128"
            counter
            :rules="[(v) => !!v?.trim() || i18nT('raw.s6a17dd')]"
            lazy-rules
          />
          <q-input
            v-model="brandForm.description"
            :label="i18nT('raw.s9e58f1')"
            filled
            type="textarea"
            class="q-mt-md"
            maxlength="1024"
          />
          <q-input
            v-model="brandForm.logoUrl"
            label="Logo URL"
            filled
            class="q-mt-md"
            maxlength="512"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn
            :label="i18nT('raw.sfe9512')"
            color="primary"
            :loading="brandSaving"
            :disable="!brandForm.name.trim()"
            @click="saveBrand"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 删除品牌 -->
    <q-dialog v-model="brandDeleteDialog">
      <q-card style="min-width: 420px">
        <q-card-section class="row items-center">
          <q-icon name="warning" color="negative" size="28px" class="q-mr-sm" />
          <div class="text-h6">{{ i18nT('raw.s0f2f08') }}</div>
        </q-card-section>
        <q-card-section class="q-pt-none">
          {{ i18nT('raw.s2d9b93', { name: brandToDelete?.name }) }}
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn
            :label="i18nT('raw.sbd7449')"
            color="negative"
            :loading="brandDeleting"
            @click="confirmDeleteBrand"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { formatDate } from '@erp-new-frontend-monorepo/utils';
import { onMounted, ref } from 'vue';
import { useQuasar } from 'quasar';
import { catalogApi } from '@/api';
import { EmptyState, PageTable } from '@erp-new-frontend-monorepo/components';

interface BrandItem {
  id: string;
  name: string;
  slug?: string;
  description?: string | null;
  logoUrl?: string | null;
  createdAtUtc?: string;
}

const $q = useQuasar();

function toast(type: 'positive' | 'negative', message: string) {
  $q.notify({ type, message });
}

function initial(name?: string) {
  const n = (name ?? '').trim();
  return n ? n.charAt(0).toUpperCase() : '·';
}

const brandRows = ref<BrandItem[]>([]);
const brandLoading = ref(false);
const brandError = ref('');
const brandTotal = ref(0);
const brandSearch = ref('');
const brandPagination = ref({ page: 1, rowsPerPage: 20, rowsNumber: 0 });

const brandColumns = [
  { name: 'name', label: i18nT('raw.sf43de3'), field: 'name', align: 'left' as const },
  {
    name: 'description',
    label: i18nT('raw.s9e58f1'),
    field: 'description',
    align: 'left' as const,
  },
  { name: 'logo', label: 'Logo', field: 'logoUrl', align: 'center' as const },
  {
    name: 'createdAtUtc',
    label: i18nT('raw.saf9956'),
    field: 'createdAtUtc',
    align: 'left' as const,
  },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions', align: 'right' as const },
];

async function loadBrands(p?: { page: number; rowsPerPage: number }) {
  const pg = p ?? brandPagination.value;
  brandLoading.value = true;
  try {
    const params: { page: number; pageSize: number; search?: string } = {
      page: pg.page,
      pageSize: pg.rowsPerPage,
    };
    if (brandSearch.value.trim()) params.search = brandSearch.value.trim();
    const res = (await catalogApi.brands(params)) as { total: number; items: BrandItem[] };
    brandRows.value = res.items ?? [];
    brandTotal.value = res.total ?? brandRows.value.length;
    brandPagination.value = {
      page: pg.page,
      rowsPerPage: pg.rowsPerPage,
      rowsNumber: res.total ?? brandRows.value.length,
    };
  } catch (e) {
    brandError.value = e instanceof Error ? e.message : i18nT('raw.se65c4b');
  } finally {
    brandLoading.value = false;
  }
}

function onBrandRequest(payload: { page: number; rowsPerPage: number }) {
  void loadBrands(payload);
}

function onBrandSearch() {
  void loadBrands({ page: 1, rowsPerPage: brandPagination.value.rowsPerPage });
}

const brandDialog = ref(false);
const brandSaving = ref(false);
const editingBrand = ref<BrandItem | null>(null);
const brandForm = ref<{ name: string; description: string; logoUrl: string }>({
  name: '',
  description: '',
  logoUrl: '',
});

function openBrandCreate() {
  editingBrand.value = null;
  brandForm.value = { name: '', description: '', logoUrl: '' };
  brandDialog.value = true;
}

function openBrandEdit(b: BrandItem) {
  editingBrand.value = b;
  brandForm.value = {
    name: b.name ?? '',
    description: b.description ?? '',
    logoUrl: b.logoUrl ?? '',
  };
  brandDialog.value = true;
}

async function saveBrand() {
  if (!brandForm.value.name.trim()) return;
  brandSaving.value = true;
  try {
    const payload: { name: string; description?: string; logoUrl?: string } = {
      name: brandForm.value.name.trim(),
    };
    if (brandForm.value.description.trim())
      payload.description = brandForm.value.description.trim();
    if (brandForm.value.logoUrl.trim()) payload.logoUrl = brandForm.value.logoUrl.trim();
    if (editingBrand.value) {
      await catalogApi.updateBrand(editingBrand.value.id, payload);
      toast('positive', i18nT('raw.sd53cd8'));
    } else {
      await catalogApi.createBrand(payload);
      toast('positive', i18nT('raw.s8700b3'));
    }
    brandDialog.value = false;
    await loadBrands();
  } catch (e) {
    brandError.value =
      e instanceof Error
        ? e.message
        : editingBrand.value
          ? i18nT('raw.se410d8')
          : i18nT('raw.sb62cb3');
  } finally {
    brandSaving.value = false;
  }
}

const brandDeleteDialog = ref(false);
const brandDeleting = ref(false);
const brandToDelete = ref<BrandItem | null>(null);

async function confirmDeleteBrand() {
  if (!brandToDelete.value) return;
  brandDeleting.value = true;
  try {
    await catalogApi.deleteBrand(brandToDelete.value.id);
    toast('positive', i18nT('raw.s2e9a78'));
    brandDeleteDialog.value = false;
    await loadBrands();
  } catch (e) {
    brandError.value = e instanceof Error ? e.message : i18nT('raw.s2324d0');
  } finally {
    brandDeleting.value = false;
  }
}

onMounted(() => {
  void loadBrands();
});
</script>
