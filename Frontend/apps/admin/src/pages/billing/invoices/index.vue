<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <q-btn
        flat
        round
        dense
        icon="arrow_back"
        @click="router.back()"
        :title="i18nT('raw.s4b2ea3')"
      />
      <div class="text-h5 q-ml-sm">{{ i18nT('raw.s9b165d') }}</div>
    </div>

    <PageTable
      :rows="rows"
      :columns="columns"
      :loading="loading"
      :total="total"
      :dense="$q.screen.lt.md"
      :rows-per-page="size"
      @request="onRequest"
      @row-click="onRowClick"
    >
      <template #body-cell-amount="{ row }">
        <q-td>{{ formatAmount(row) }}</q-td>
      </template>
      <template #body-cell-status="{ row }">
        <q-td>
          <q-badge :color="statusColor(row.status)">{{ statusText(row.status) }}</q-badge>
        </q-td>
      </template>
      <template #body-cell-time="{ row }">
        <q-td>{{ formatDate(row.createdAtUtc || row.issuedAtUtc) }}</q-td>
      </template>
    </PageTable>
    <EmptyState
      v-if="!loading && rows.length === 0"
      :title="i18nT('raw.se4088c')"
      :hint="i18nT('raw.s8698c5')"
      icon="receipt_long"
    />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { onMounted } from 'vue';
import {
  formatDate,
  invoiceStatusText as statusText,
  invoiceStatusColor as statusColor,
} from '@erp-new-frontend-monorepo/utils';
import { useRouter } from 'vue-router';
import { billingApi } from '@/api';
import { EmptyState, PageTable } from '@erp-new-frontend-monorepo/components';
import { useTableState } from '@erp-new-frontend-monorepo/composables';

const router = useRouter();

interface InvoiceRow {
  id: string;
  invoiceNumber: string;
  currency: string;
  subtotalAmount: number;
  status: string | number;
  createdAtUtc?: string;
  issuedAtUtc?: string;
}

const columns = [
  { name: 'invoiceNumber', label: i18nT('raw.s628022'), field: 'invoiceNumber', sortable: true },
  { name: 'amount', label: i18nT('raw.s0fdb17'), field: 'subtotalAmount' },
  { name: 'status', label: i18nT('raw.s9fb403'), field: 'status' },
  { name: 'time', label: i18nT('raw.sdf331d'), field: 'createdAtUtc' },
];

const { rows, total, loading, load, page, size, goToPage, setSize } = useTableState<InvoiceRow>({
  size: 20,
});

function formatAmount(row: InvoiceRow) {
  return `${row.currency || ''} ${Number(row.subtotalAmount ?? 0).toFixed(2)}`.trim();
}

async function doLoad() {
  await load(() =>
    billingApi
      .invoices(page.value, size.value)
      .then((res: { items: InvoiceRow[]; total: number }) => {
        rows.value = (res.items || []) as InvoiceRow[];
        total.value = res.total || 0;
      }),
  );
}

function onRequest({ page: p, rowsPerPage: s }: { page: number; rowsPerPage: number }) {
  if (s !== size.value) {
    void setSize(s);
  } else {
    void goToPage(p);
  }
}

function onRowClick(_evt: unknown, row: InvoiceRow) {
  void router.push(`/billing/invoices/${row.id}`);
}

onMounted(doLoad);
</script>
