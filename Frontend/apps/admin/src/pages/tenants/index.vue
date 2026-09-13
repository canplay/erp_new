<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <div class="text-h5">{{ i18nT('raw.sa85900') }}</div>
      <q-space />
      <q-btn
        color="primary"
        icon="domain_add"
        :label="i18nT('raw.s44cfc2')"
        @click="showCreate = true"
      />
    </div>

    <PageTable
      :rows="rowsView"
      :columns="columns"
      :loading="loading"
      :total="total"
      :dense="$q.screen.lt.md"
      @request="onRequest"
    >
      <template #body-cell-actions="{ row }">
        <q-btn
          flat
          dense
          icon="info"
          size="sm"
          @click="openStatus(row)"
          :title="i18nT('raw.s35305d')"
        />
        <q-btn
          flat
          dense
          :icon="row.isActive ? 'pause' : 'play_arrow'"
          size="sm"
          @click="toggleTenant(row)"
          :title="row.isActive ? i18nT('raw.sd64e4d') : i18nT('raw.sc8a730')"
        />
        <q-btn
          flat
          dense
          icon="event_repeat"
          size="sm"
          @click="renewTenant(row)"
          :title="i18nT('raw.sbe9595')"
        />
      </template>
    </PageTable>

    <!-- 状态 / 有效期弹窗 -->
    <q-dialog v-model="statusDialog">
      <q-card style="min-width: 460px; max-width: 90vw">
        <q-card-section class="row items-center">
          <div class="text-h6">租户状态 — {{ currentTenant?.name }}</div>
          <q-space />
          <q-btn flat round icon="close" v-close-popup />
        </q-card-section>
        <q-card-section>
          <div v-if="statusLoading" class="text-grey-6 text-caption q-pa-md">
            {{ i18nT('raw.s2baf28') }}
          </div>
          <q-list v-else-if="status" separator dense>
            <q-item v-for="(val, key) in status" :key="key">
              <q-item-section>{{ statusKeyLabel(key) }}</q-item-section>
              <q-item-section side>{{ statusValLabel(val) }}</q-item-section>
            </q-item>
          </q-list>
          <div v-else class="text-grey-6 text-caption q-pa-md">{{ i18nT('raw.se68deb') }}</div>
        </q-card-section>
        <q-separator />
        <q-card-section>
          <div class="text-subtitle2 q-mb-sm">{{ i18nT('raw.sa397ee') }}</div>
          <div class="row items-center q-gutter-sm">
            <q-input
              v-model="validityDate"
              :label="i18nT('raw.sa920c0')"
              type="datetime-local"
              filled
              dense
              class="col"
            />
            <q-btn
              color="primary"
              :label="i18nT('raw.sab8c55')"
              :loading="adjusting"
              @click="adjustValidity"
            />
          </div>
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.sdedda3')" v-close-popup />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <q-dialog v-model="showCreate">
      <q-card style="min-width: 420px">
        <q-card-section class="text-h6">{{ i18nT('raw.s44cfc2') }}</q-card-section>
        <q-card-section>
          <q-input
            v-model="form.name"
            :label="i18nT('raw.s28dbc7')"
            filled
            :rules="[(v) => !!v || i18nT('raw.s6a17dd')]"
            lazy-rules
          />
          <q-input
            v-model="form.id"
            :label="i18nT('raw.s0d31e0')"
            filled
            :rules="[(v) => !!v || i18nT('raw.s6a17dd')]"
            :hint="i18nT('raw.s50edcd')"
            class="q-mt-md"
            lazy-rules
          />
          <q-input
            v-model="form.adminEmail"
            :label="i18nT('raw.s5ef54b')"
            type="email"
            filled
            class="q-mt-md"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn
            :label="i18nT('raw.s3089ce')"
            color="primary"
            :loading="saving"
            @click="createTenant"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { tenantsApi } from '@/api';
import { PageTable } from '@erp-new-frontend-monorepo/components';
import {
  tenantStatusKeyLabel as statusKeyLabel,
  formatDateTime,
} from '@erp-new-frontend-monorepo/utils';

interface TenantRow {
  id: string;
  name: string;
  identifier?: string;
  adminEmail?: string;
  validUpto?: string;
  isActive: boolean;
}

const columns = [
  { name: 'id', label: i18nT('raw.sc34f03'), field: 'id' },
  { name: 'name', label: i18nT('raw.sf43de3'), field: 'name' },
  { name: 'adminEmail', label: i18nT('raw.s5ef54b'), field: 'adminEmail' },
  {
    name: 'validUpto',
    label: i18nT('raw.s466980'),
    field: 'validUpto',
    format: (v: unknown) => {
      if (v === null || v === undefined) return '—';
      if (typeof v === 'string' || typeof v === 'number') {
        const d = new Date(String(v));
        if (!Number.isNaN(d.getTime())) {
          const pad = (n: number) => String(n).padStart(2, '0');
          return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
        }
        return String(v);
      }
      return '—';
    },
  },
  {
    name: 'isActive',
    label: i18nT('raw.s9fb403'),
    field: 'isActive',
    format: (v: unknown) => (v ? i18nT('raw.sc8a730') : i18nT('raw.sd64e4d')),
  },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions' },
];
const rows = ref<TenantRow[]>([]);
const loading = ref(false);
const listPage = ref({ page: 1, rowsPerPage: 20 });
const rowsView = computed(() =>
  rows.value.slice(
    (listPage.value.page - 1) * listPage.value.rowsPerPage,
    listPage.value.page * listPage.value.rowsPerPage,
  ),
);
const total = computed(() => rows.value.length);
const showCreate = ref(false);
const saving = ref(false);
const form = ref({ name: '', id: '', adminEmail: '' });

const statusDialog = ref(false);
const currentTenant = ref<TenantRow | null>(null);
const status = ref<Record<string, unknown> | null>(null);
const statusLoading = ref(false);
const validityDate = ref('');
const adjusting = ref(false);

const $q = useQuasar();

function statusValLabel(val: unknown) {
  if (typeof val === 'boolean') return val ? i18nT('raw.sc8a730') : i18nT('raw.sd64e4d');
  if (val === null || val === undefined || val === '') return '—';
  if (typeof val === 'string' && !Number.isNaN(Date.parse(val))) {
    return formatDateTime(val);
  }
  return typeof val === 'string' || typeof val === 'number' ? String(val) : '';
}

async function load() {
  loading.value = true;
  try {
    rows.value = (await tenantsApi.list()) || [];
  } finally {
    loading.value = false;
  }
}

async function openStatus(t: TenantRow) {
  currentTenant.value = t;
  status.value = null;
  statusLoading.value = true;
  statusDialog.value = true;
  validityDate.value = '';
  try {
    const s = await tenantsApi.tenantStatus(t.id);
    status.value = s && typeof s === 'object' ? s : { status: s };
    const exp = s?.expiresOnUtc ?? s?.validUpto;
    if (exp) {
      validityDate.value = new Date(exp).toISOString().slice(0, 16);
    }
  } catch (e) {
    $q.notify({
      type: 'negative',
      message: (e as { message?: string })?.message || i18nT('raw.sd6e7ff'),
    });
    statusDialog.value = false;
  } finally {
    statusLoading.value = false;
  }
}

async function adjustValidity() {
  if (!currentTenant.value || !validityDate.value) return;
  const dt = new Date(validityDate.value);
  if (Number.isNaN(dt.getTime())) {
    $q.notify({ type: 'negative', message: i18nT('raw.s114cf6') });
    return;
  }
  adjusting.value = true;
  try {
    await tenantsApi.adjustValidity(currentTenant.value.id, { expiresOnUtc: dt.toISOString() });
    $q.notify({ type: 'positive', message: i18nT('raw.sd8d05e') });
    await openStatus(currentTenant.value);
    await load();
  } catch (e) {
    $q.notify({
      type: 'negative',
      message: (e as { message?: string })?.message || i18nT('raw.s8771e4'),
    });
  } finally {
    adjusting.value = false;
  }
}

async function toggleTenant(t: TenantRow) {
  await tenantsApi.toggle(t.id, !t.isActive);
  await load();
}

async function renewTenant(t: TenantRow) {
  await tenantsApi.renew(t.id);
  await load();
}

async function createTenant() {
  saving.value = true;
  try {
    await tenantsApi.create(form.value);
    showCreate.value = false;
    form.value = { name: '', id: '', adminEmail: '' };
    await load();
  } finally {
    saving.value = false;
  }
}

function onRequest({ page, rowsPerPage }: { page: number; rowsPerPage: number }) {
  listPage.value = { page, rowsPerPage };
}

onMounted(load);
</script>
