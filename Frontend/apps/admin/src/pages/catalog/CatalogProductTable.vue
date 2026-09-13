<template>
  <PageTable
    v-if="loading || rows.length > 0"
    :dense="$q.screen.lt.md"
    :rows="rows"
    :columns="columns"
    :loading="loading"
    :total="total"
    flat
    bordered
    @request="$emit('request', $event)"
  >
    <template #body-cell-name="props">
      <q-td>
        <div class="row items-center no-wrap" style="gap: 12px">
          <q-avatar size="36px" rounded>
            <img
              v-if="props.row.thumbnailUrl"
              :src="props.row.thumbnailUrl"
              alt=""
              loading="lazy"
            />
            <span v-else class="text-primary">{{ initial(props.row.name) }}</span>
          </q-avatar>
          <span
            class="text-weight-medium ellipsis"
            style="max-width: 240px"
            :title="props.row.name"
          >
            {{ props.row.name }}
          </span>
          <q-badge
            v-if="props.row.isActive === false"
            color="grey"
            :label="i18nT('raw.s893688')"
            class="q-ml-xs"
          />
        </div>
      </q-td>
    </template>
    <template #body-cell-sku="props">
      <q-td
        ><code class="text-grey-8">{{ props.row.sku || '—' }}</code></q-td
      >
    </template>
    <template #body-cell-brand="props">
      <q-td>{{ brandNameOf(props.row.brandId, brandOptions) }}</q-td>
    </template>
    <template #body-cell-category="props">
      <q-td>{{ categoryNameOf(props.row.categoryId, categoryOptions) }}</q-td>
    </template>
    <template #body-cell-price="props">
      <q-td class="text-right text-weight-medium">
        {{ formatMoney(props.row.price?.amount, props.row.price?.currency) }}
      </q-td>
    </template>
    <template #body-cell-stock="props">
      <q-td>
        <q-badge
          :color="stockColor(props.row.stock)"
          :label="String(props.row.stock ?? '—')"
          outline
        />
      </q-td>
    </template>
    <template #body-cell-actions="props">
      <q-td>
        <q-btn
          flat
          dense
          round
          icon="visibility"
          size="sm"
          @click="$emit('view', props.row)"
          :title="i18nT('raw.sb166e7')"
        />
        <q-btn
          flat
          dense
          round
          icon="edit"
          size="sm"
          @click="$emit('edit', props.row)"
          :title="i18nT('raw.s207008')"
        />
        <q-btn
          flat
          dense
          round
          icon="delete"
          size="sm"
          text-color="negative"
          @click="$emit('delete', props.row)"
          :title="i18nT('raw.sbd7449')"
        />
      </q-td>
    </template>
  </PageTable>
  <EmptyState
    v-else
    :title="i18nT('raw.s5035fe')"
    :hint="i18nT('raw.s0aca38')"
    icon="inventory_2"
  />
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useQuasar } from 'quasar';
import { PageTable } from '@erp-new-frontend-monorepo/components';
import { EmptyState } from '@erp-new-frontend-monorepo/components';
import type { ProductItem } from './catalogProductUtils';
import {
  formatMoney,
  initial,
  stockColor,
  brandNameOf,
  categoryNameOf,
} from './catalogProductUtils';

const { t: i18nT } = useI18n();
const $q = useQuasar();

defineProps<{
  rows: ProductItem[];
  columns: {
    name: string;
    label: string;
    field: string;
    align?: 'left' | 'center' | 'right';
    sortable?: boolean;
  }[];
  loading: boolean;
  total: number;
  brandOptions: { label: string; value: string }[];
  categoryOptions: { label: string; value: string }[];
}>();

defineEmits<{
  (e: 'request', payload: { page: number; rowsPerPage: number }): void;
  (e: 'view', row: ProductItem): void;
  (e: 'edit', row: ProductItem): void;
  (e: 'delete', row: ProductItem): void;
}>();
</script>
