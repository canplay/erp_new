<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <div class="text-h5">{{ i18nT('raw.s637e43') }}</div>
      <q-space />
      <q-btn color="primary" icon="add" :label="i18nT('raw.s0a3bf6')" @click="showCreate = true" />
    </div>

    <PageTable
      :rows="rows"
      :columns="columns"
      :loading="loading"
      :total="total"
      :dense="$q.screen.lt.md"
      @request="onRequest"
    >
      <template #body-cell-active="{ row }">
        <q-td
          ><q-badge :color="row.active ? 'green' : 'grey'">{{
            row.active ? i18nT('raw.sc8a730') : i18nT('raw.sd64e4d')
          }}</q-badge></q-td
        >
      </template>
      <template #body-cell-actions="{ row }">
        <q-td>
          <q-btn
            flat
            dense
            icon="history"
            size="sm"
            @click="openDeliveries(row.id)"
            :title="i18nT('raw.s1dc859')"
          />
          <q-btn
            flat
            dense
            icon="play_circle_outline"
            size="sm"
            color="primary"
            @click="testTrigger(row.id)"
            :title="i18nT('raw.s1d66b5')"
          />
          <q-btn
            flat
            dense
            icon="delete"
            size="sm"
            @click="confirmRemove(row)"
            :title="i18nT('raw.sbd7449')"
          />
        </q-td>
      </template>
    </PageTable>

    <q-dialog v-model="showDeliveries">
      <q-card style="min-width: 720px; max-width: 90vw">
        <q-card-section class="row items-center">
          <div class="text-h6">{{ i18nT('raw.s1dc859') }}</div>
          <q-space />
          <q-btn
            flat
            round
            dense
            icon="refresh"
            @click="loadDeliveries()"
            :title="i18nT('raw.s6d94e0')"
          />
        </q-card-section>
        <q-card-section class="q-pt-none">
          <PageTable
            :rows="deliveries"
            :columns="deliveryColumns"
            :loading="loadingDeliveries"
            :total="deliveries.length"
          >
            <template #body-cell-time="{ row }">
              <q-td>{{
                formatDate(row.createdAtUtc || row.deliveredAtUtc || row.timestampUtc)
              }}</q-td>
            </template>
            <template #body-cell-status="{ row }">
              <q-td>
                <q-badge
                  :color="
                    String(row.status).toLowerCase().includes('success') || row.status === 200
                      ? 'green'
                      : 'orange'
                  "
                  >{{ row.status }}</q-badge
                >
              </q-td>
            </template>
            <template #body-cell-request="{ row }">
              <q-td class="ellipsis" style="max-width: 320px">{{
                row.requestBody || row.request || row.payload || '—'
              }}</q-td>
            </template>
          </PageTable>
        </q-card-section>
        <q-card-actions align="right"
          ><q-btn :label="i18nT('raw.sdedda3')" v-close-popup
        /></q-card-actions>
      </q-card>
    </q-dialog>

    <q-dialog v-model="showCreate">
      <q-card style="min-width: 420px">
        <q-card-section class="text-h6">{{ i18nT('raw.s282db4') }}</q-card-section>
        <q-card-section>
          <q-input
            v-model="form.url"
            :label="i18nT('raw.sea942c')"
            filled
            :rules="[(v) => !!v || i18nT('raw.s6a17dd')]"
            lazy-rules
          />
          <q-input v-model="form.secret" :label="i18nT('raw.sdaa4eb')" filled class="q-mt-md" />
          <q-input
            v-model="form.eventTypes"
            :label="i18nT('raw.s0ffa8a')"
            filled
            class="q-mt-md"
            placeholder="order.created,invoice.paid"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn :label="i18nT('raw.s3089ce')" color="primary" :loading="saving" @click="create" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <ConfirmDialog
      v-model="deleteDialog"
      :title="i18nT('raw.sbd7449')"
      :message="`确认删除 Webhook「${deleteTarget?.url}」？该操作不可恢复。`"
      :loading="deleteConfirming"
      @confirm="doRemove"
    />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, onMounted } from 'vue';
import { formatDate } from '@erp-new-frontend-monorepo/utils';
import { useQuasar } from 'quasar';
import { webhooksApi } from '@/api';
import { PageTable, ConfirmDialog } from '@erp-new-frontend-monorepo/components';

const $q = useQuasar();

interface WebhookSubscription {
  id: string;
  url: string;
  eventTypes?: string[] | string;
  active?: boolean;
}

interface WebhookDelivery {
  id: string;
  createdAtUtc?: string;
  deliveredAtUtc?: string;
  timestampUtc?: string;
  status?: string | number;
  requestBody?: string;
  request?: string;
  payload?: string;
}

const columns = [
  { name: 'url', label: i18nT('raw.sea942c'), field: 'url' },
  { name: 'eventTypes', label: i18nT('raw.s7066f7'), field: 'eventTypes' },
  { name: 'active', label: i18nT('raw.s9fb403'), field: 'active' },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions' },
];
const rows = ref<WebhookSubscription[]>([]);
const total = ref(0);
const loading = ref(false);
const currentPage = ref(1);
const pageSize = ref(20);
const showCreate = ref(false);
const saving = ref(false);
const deleteTarget = ref<WebhookSubscription | null>(null);
const deleteDialog = ref(false);
const deleteConfirming = ref(false);
const form = ref({ url: '', secret: '', eventTypes: '' });

const showDeliveries = ref(false);
const deliverySubId = ref('');
const deliveries = ref<WebhookDelivery[]>([]);
const loadingDeliveries = ref(false);
const deliveryColumns = [
  { name: 'time', label: i18nT('raw.sdf331d'), field: 'createdAtUtc' },
  { name: 'status', label: i18nT('raw.s9fb403'), field: 'status' },
  { name: 'request', label: i18nT('raw.s0097cf'), field: 'requestBody' },
];

function toast(type: 'positive' | 'negative', message: string) {
  $q.notify({ type, message });
}

async function loadDeliveries() {
  if (!deliverySubId.value) return;
  loadingDeliveries.value = true;
  try {
    deliveries.value = (await webhooksApi.deliveries(deliverySubId.value)) || [];
  } finally {
    loadingDeliveries.value = false;
  }
}

function openDeliveries(id: string) {
  deliverySubId.value = id;
  showDeliveries.value = true;
  void loadDeliveries();
}

async function testTrigger(id: string) {
  try {
    await webhooksApi.test(id);
    toast('positive', i18nT('raw.s62fd58'));
  } catch {
    toast('negative', i18nT('raw.s58d434'));
  }
}

async function load() {
  loading.value = true;
  try {
    const res = await webhooksApi.subscriptions(currentPage.value, pageSize.value);
    rows.value = res.items;
    total.value = res.total;
  } finally {
    loading.value = false;
  }
}

function onRequest({ page, rowsPerPage }: { page: number; rowsPerPage: number }) {
  currentPage.value = page;
  pageSize.value = rowsPerPage;
  void load();
}

async function create() {
  saving.value = true;
  try {
    await webhooksApi.create({
      url: form.value.url,
      secret: form.value.secret,
      eventTypes: form.value.eventTypes
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean),
    });
    showCreate.value = false;
    form.value = { url: '', secret: '', eventTypes: '' };
    await load();
  } finally {
    saving.value = false;
  }
}

function confirmRemove(row: WebhookSubscription) {
  deleteTarget.value = row;
  deleteDialog.value = true;
}

async function remove(id: string) {
  await webhooksApi.remove(id);
  await load();
}

async function doRemove() {
  if (!deleteTarget.value) return;
  deleteConfirming.value = true;
  try {
    await remove(deleteTarget.value.id);
    deleteDialog.value = false;
  } finally {
    deleteConfirming.value = false;
  }
}

onMounted(load);
</script>
