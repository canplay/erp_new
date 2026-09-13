<template>
  <q-page class="erp-page">
    <q-breadcrumbs class="q-mb-sm">
      <q-breadcrumbs-el :label="i18nT('breadcrumb_billing')" to="/billing" />
      <q-breadcrumbs-el :label="i18nT('breadcrumb_invoice_detail')" />
    </q-breadcrumbs>
    <div class="row items-center q-mb-md">
      <q-btn
        flat
        round
        dense
        icon="arrow_back"
        @click="router.back()"
        :title="i18nT('raw.s4b2ea3')"
      />
      <div class="text-h5 q-ml-sm">{{ i18nT('raw.s5ecf9e') }}</div>
    </div>

    <q-card v-if="invoice">
      <q-card-section class="row items-center">
        <q-avatar
          size="48px"
          color="indigo"
          text-color="white"
          icon="receipt_long"
          class="q-mr-md"
        />
        <div>
          <div class="text-h6">{{ invoice.invoiceNumber }}</div>
          <div class="text-caption text-grey-6">租户: {{ invoice.tenantId }}</div>
        </div>
        <q-space />
        <q-badge :color="statusColor" size="md">{{ statusText }}</q-badge>
      </q-card-section>
      <q-separator />
      <q-card-section>
        <!-- 操作按钮 -->
        <div class="row q-gutter-sm q-mb-md">
          <q-btn
            v-if="invoice.status === 'Draft'"
            color="primary"
            icon="send"
            label="开具"
            :loading="operating"
            @click="issueInvoice"
          />
          <q-btn
            v-if="invoice.status === 'Issued'"
            color="positive"
            icon="payments"
            label="支付"
            :loading="operating"
            @click="payInvoice"
          />
          <q-btn
            v-if="invoice.status !== 'Void' && invoice.status !== 'Draft'"
            color="negative"
            icon="cancel"
            label="作废"
            :loading="operating"
            @click="voidInvoice"
          />
          <q-btn flat icon="picture_as_pdf" label="PDF" @click="downloadPdf" />
        </div>
        <q-list dense padding>
          <q-item>
            <q-item-section avatar><q-icon name="receipt" color="indigo" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s628022') }}</q-item-label>
              <q-item-label>{{ invoice.invoiceNumber }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="domain" color="indigo" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s97a093') }}</q-item-label>
              <q-item-label>{{ invoice.tenantId }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="payments" color="indigo" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s0fdb17') }}</q-item-label>
              <q-item-label class="text-subtitle1 text-weight-medium">{{
                formatAmount(invoice.subtotalAmount, invoice.currency)
              }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar
              ><q-icon name="power_settings_new" color="indigo"
            /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s9fb403') }}</q-item-label>
              <q-item-label
                ><q-badge :color="statusColor">{{ statusText }}</q-badge></q-item-label
              >
            </q-item-section>
          </q-item>
          <q-item v-if="invoice.periodYear">
            <q-item-section avatar><q-icon name="calendar_month" color="indigo" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.sb93790') }}</q-item-label>
              <q-item-label
                >{{ invoice.periodYear }}-{{
                  String(invoice.periodMonth ?? 0).padStart(2, '0')
                }}</q-item-label
              >
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="schedule" color="indigo" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.sc2fb58') }}</q-item-label>
              <q-item-label>{{
                formatDate(invoice.issuedAtUtc || invoice.createdAtUtc)
              }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item v-if="invoice.dueAtUtc">
            <q-item-section avatar><q-icon name="event" color="indigo" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.sa920c0') }}</q-item-label>
              <q-item-label>{{ formatDate(invoice.dueAtUtc) }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item v-if="invoice.paidAtUtc">
            <q-item-section avatar><q-icon name="check_circle" color="indigo" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s9c031e') }}</q-item-label>
              <q-item-label>{{ formatDate(invoice.paidAtUtc) }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item v-if="invoice.notes">
            <q-item-section avatar><q-icon name="notes" color="indigo" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.se8e6f5') }}</q-item-label>
              <q-item-label>{{ invoice.notes }}</q-item-label>
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>
    </q-card>

    <q-card v-if="invoice && invoice.lineItems?.length" class="q-mt-md">
      <q-card-section class="text-h6">{{ i18nT('raw.s6856b8') }}</q-card-section>
      <q-table
        :dense="$q.screen.lt.md"
        :rows="lineItems"
        :columns="lineColumns"
        row-key="id"
        :pagination="{ rowsPerPage: 0 }"
        hide-bottom
      >
        <template #body-cell-amount="props">
          <q-td>{{ formatAmount(props.row.amount, invoice.currency) }}</q-td>
        </template>
        <template #body-cell-kind="props">
          <q-td>
            <q-chip
              v-if="props.row.kind"
              size="xs"
              color="indigo"
              text-color="white"
              :label="kindText(props.row.kind)"
            />
          </q-td>
        </template>
      </q-table>
    </q-card>

    <div v-if="loading" class="q-pa-xl column items-center text-grey-6">
      <q-spinner size="40px" class="q-mb-sm" />
      <div>{{ i18nT('raw.s795a79') }}</div>
    </div>
    <EmptyState
      v-else-if="!invoice"
      :title="i18nT('raw.scda539')"
      :hint="i18nT('raw.sce948c')"
      icon="receipt_long"
    />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import {
  formatDate,
  invoiceStatusText,
  invoiceStatusColor,
  invoiceKindText,
} from '@erp-new-frontend-monorepo/utils';
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { billingApi } from '@/api';
import { EmptyState } from '@erp-new-frontend-monorepo/components';
import { useTableState } from '@erp-new-frontend-monorepo/composables';
import { useQuasar } from 'quasar';

const route = useRoute();
const router = useRouter();
const $q = useQuasar();

const id = String((route.params as { id?: string }).id ?? '');

interface InvoiceLineItem {
  id: string;
  kind?: string | number;
  description?: string;
  quantity?: number;
  unitPrice?: number;
  amount?: number;
}

interface Invoice {
  id?: string;
  tenantId?: string;
  invoiceNumber?: string;
  periodYear?: number;
  periodMonth?: number;
  currency?: string;
  subtotalAmount?: number;
  status?: string | number;
  createdAtUtc?: string;
  issuedAtUtc?: string;
  dueAtUtc?: string;
  paidAtUtc?: string;
  notes?: string;
  lineItems?: InvoiceLineItem[];
}

const { loading, load } = useTableState<Invoice>();
const invoice = ref<Invoice | null>(null);

const lineColumns = [
  { name: 'kind', label: i18nT('raw.sd68dd1'), field: 'kind' },
  { name: 'description', label: i18nT('raw.s45ebe1'), field: 'description' },
  { name: 'quantity', label: i18nT('raw.sbdc4d4'), field: 'quantity' },
  { name: 'unitPrice', label: i18nT('raw.s7c6707'), field: 'unitPrice' },
  { name: 'amount', label: i18nT('raw.s0fdb17'), field: 'amount' },
];

const lineItems = computed<InvoiceLineItem[]>(() =>
  Array.isArray(invoice.value?.lineItems) ? (invoice.value?.lineItems as InvoiceLineItem[]) : [],
);

const statusText = computed(() => invoiceStatusText(invoice.value?.status));
const statusColor = computed(() => invoiceStatusColor(invoice.value?.status));

function formatAmount(value?: number | string, currency?: string) {
  return `${currency || ''} ${Number(value ?? 0).toFixed(2)}`.trim();
}

function kindText(kind: string | number | undefined) {
  return invoiceKindText(kind);
}

const operating = ref(false);

async function issueInvoice() {
  if (!invoice.value) return;
  operating.value = true;
  try {
    await billingApi.issueInvoice(invoice.value.id!);
    $q.notify({ type: 'positive', message: '发票已开具' });
    await loadInvoice();
  } catch {
    $q.notify({ type: 'negative', message: '操作失败' });
  } finally {
    operating.value = false;
  }
}

async function payInvoice() {
  if (!invoice.value) return;
  operating.value = true;
  try {
    await billingApi.payInvoice(invoice.value.id!);
    $q.notify({ type: 'positive', message: '发票已支付' });
    await loadInvoice();
  } catch {
    $q.notify({ type: 'negative', message: '操作失败' });
  } finally {
    operating.value = false;
  }
}

async function voidInvoice() {
  if (!invoice.value) return;
  operating.value = true;
  try {
    await billingApi.voidInvoice(invoice.value.id!);
    $q.notify({ type: 'positive', message: '发票已作废' });
    await loadInvoice();
  } catch {
    $q.notify({ type: 'negative', message: '操作失败' });
  } finally {
    operating.value = false;
  }
}

function downloadPdf() {
  if (invoice.value) window.open(`/api/v1/billing/invoices/${invoice.value.id}/pdf`, '_blank');
}

async function loadInvoice() {
  await load(() =>
    billingApi.invoiceDetail(id).then((r: Invoice) => {
      invoice.value = r;
    }),
  );
}

onMounted(async () => {
  await loadInvoice();
});
</script>
