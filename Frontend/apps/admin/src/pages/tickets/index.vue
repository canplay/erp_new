<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <div class="text-h5">{{ i18nT('raw.sfb0bb1') }}</div>
      <q-space />
      <q-btn color="primary" icon="add" :label="i18nT('raw.sabea76')" @click="openCreate" />
    </div>

    <q-banner v-if="errorMessage" type="negative" rounded class="q-mb-md">
      {{ errorMessage }}
      <template #action>
        <q-btn flat color="white" :label="i18nT('raw.sdedda3')" @click="errorMessage = ''" />
      </template>
    </q-banner>

    <div class="row items-center q-gutter-sm q-mb-md">
      <q-select
        v-model="statusFilter"
        :options="statusOptions"
        :label="i18nT('raw.s6b78c8')"
        dense
        outlined
        emit-value
        map-options
        clearable
        style="min-width: 180px"
        @update:model-value="onStatusFilter"
      />
      <span class="text-caption text-grey-7">{{ i18nT('raw.stk_total', { count: total }) }}</span>
    </div>

    <PageTable
      v-if="loading || rows.length > 0"
      :rows="rows"
      :columns="columns"
      :loading="loading"
      :total="total"
      @request="onRequest"
      @row-click="onRowClick"
    >
      <template #body-cell-number="{ row }">
        <q-td
          ><code class="text-grey-8">{{ row.number }}</code></q-td
        >
      </template>
      <template #body-cell-title="{ row }">
        <q-td class="ellipsis" style="max-width: 320px">
          <span class="text-weight-medium">{{ row.title }}</span>
        </q-td>
      </template>
      <template #body-cell-priority="{ row }">
        <q-td>
          <q-badge :color="ticketPriorityColor(row.priority)" outline>
            {{ priorityLabel(row.priority) }}
          </q-badge>
        </q-td>
      </template>
      <template #body-cell-status="{ row }">
        <q-td>
          <q-badge :color="ticketStatusColor(row.status)" text-color="white">
            {{ statusLabel(row.status) }}
          </q-badge>
        </q-td>
      </template>
      <template #body-cell-assignedToUserId="{ row }">
        <q-td>
          <span v-if="row.assignedToUserId" class="ellipsis" :title="row.assignedToUserId">
            {{ row.assignedToUserId }}
          </span>
          <span v-else class="text-grey-6">{{ i18nT('raw.s4bb3fa') }}</span>
        </q-td>
      </template>
      <template #body-cell-createdAtUtc="{ row }">
        <q-td>{{ formatDate(row.createdAtUtc) }}</q-td>
      </template>
    </PageTable>
    <EmptyState
      v-else
      :title="i18nT('raw.s4da2e8')"
      :hint="i18nT('raw.s175d2e')"
      icon="confirmation_number"
    />

    <!-- 新建工单 -->
    <q-dialog v-model="createDialog">
      <q-card style="min-width: 480px; max-width: 90vw">
        <q-card-section class="text-h6">{{ i18nT('raw.sabea76') }}</q-card-section>
        <q-card-section>
          <q-input
            v-model="createForm.title"
            :label="i18nT('raw.sa0661a')"
            filled
            :rules="[(v) => !!v?.trim() || i18nT('raw.s6a17dd')]"
            maxlength="160"
            counter
            autofocus
            lazy-rules
          />
          <q-input
            v-model="createForm.description"
            :label="i18nT('raw.s9e58f1')"
            filled
            type="textarea"
            class="q-mt-md"
            maxlength="4096"
          />
          <q-select
            v-model="createForm.priority"
            :options="priorityOptions"
            :label="i18nT('raw.sfaa2a1')"
            filled
            class="q-mt-md"
            emit-value
            map-options
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
          <q-btn
            :label="i18nT('raw.s3089ce')"
            color="primary"
            :loading="creating"
            @click="createTicket"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- 详情 -->
    <TicketDetailDialog v-model="detailDialog" :ticket-id="detailId" @updated="onDetailUpdated" />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { formatDate, ticketStatusColor, ticketPriorityColor } from '@erp-new-frontend-monorepo/utils';
import { onMounted, ref } from 'vue';
import { useQuasar } from 'quasar';
import { ticketsApi } from '@/api';
import { EmptyState, PageTable } from '@erp-new-frontend-monorepo/components';
import TicketDetailDialog from './TicketDetailDialog.vue';

type TicketStatus = 'Open' | 'InProgress' | 'Resolved' | 'Closed';
type TicketPriority = 'Low' | 'Medium' | 'High' | 'Critical';

interface TableRequestProps {
  pagination: {
    sortBy?: string;
    descending?: boolean;
    page: number;
    rowsPerPage: number;
    rowsNumber?: number;
  };
}

interface TicketItem {
  id: string;
  number?: string;
  title?: string;
  description?: string | null;
  status?: TicketStatus;
  priority?: TicketPriority;
  reporterUserId?: string;
  assignedToUserId?: string | null;
  resolutionNote?: string | null;
  createdAtUtc?: string;
  updatedAtUtc?: string | null;
  resolvedAtUtc?: string | null;
  closedAtUtc?: string | null;
  commentCount?: number;
}

const $q = useQuasar();

const STATUS_LABELS: Record<TicketStatus, string> = {
  Open: i18nT('raw.s245e94'),
  InProgress: i18nT('raw.s1391c9'),
  Resolved: i18nT('raw.s27f20f'),
  Closed: i18nT('raw.s6d2d3d'),
};

const PRIORITY_LABELS: Record<TicketPriority, string> = {
  Low: i18nT('raw.sc1dfbc'),
  Medium: i18nT('raw.sd15a8b'),
  High: i18nT('raw.s554c69'),
  Critical: i18nT('raw.s2866cf'),
};

const rows = ref<TicketItem[]>([]);
const total = ref(0);
const loading = ref(false);
const errorMessage = ref('');
const pagination = ref({ page: 1, rowsPerPage: 20, rowsNumber: 0 });

const statusOptions = [
  { label: i18nT('raw.se25c4f'), value: '' },
  ...(['Open', 'InProgress', 'Resolved', 'Closed'] as const).map((s) => ({
    label: STATUS_LABELS[s],
    value: s,
  })),
];
const statusFilter = ref<string | null>(null);

const priorityOptions = (['Low', 'Medium', 'High', 'Critical'] as const).map((p) => ({
  label: PRIORITY_LABELS[p],
  value: p,
}));

const columns = [
  { name: 'number', label: i18nT('raw.sa86114'), field: 'number', align: 'left' as const },
  { name: 'title', label: i18nT('raw.sa0661a'), field: 'title', align: 'left' as const },
  { name: 'priority', label: i18nT('raw.sfaa2a1'), field: 'priority', align: 'center' as const },
  { name: 'status', label: i18nT('raw.s9fb403'), field: 'status', align: 'center' as const },
  {
    name: 'assignedToUserId',
    label: i18nT('raw.s82a318'),
    field: 'assignedToUserId',
    align: 'left' as const,
  },
  {
    name: 'createdAtUtc',
    label: i18nT('raw.saf9956'),
    field: 'createdAtUtc',
    align: 'left' as const,
  },
];

function statusLabel(s?: TicketStatus | null) {
  return s ? STATUS_LABELS[s] : '—';
}

function priorityLabel(p?: TicketPriority | null) {
  return p ? PRIORITY_LABELS[p] : '—';
}

function toast(type: 'positive' | 'negative', message: string) {
  $q.notify({ type, message });
}

async function load(p?: { page: number; rowsPerPage: number }) {
  const pg = p ?? pagination.value;
  loading.value = true;
  try {
    const params: { page: number; pageSize: number; status?: string } = {
      page: pg.page,
      pageSize: pg.rowsPerPage,
    };
    if (statusFilter.value) params.status = statusFilter.value;
    const res = (await ticketsApi.list(params)) as { total: number; items: TicketItem[] };
    rows.value = res.items ?? [];
    total.value = res.total ?? rows.value.length;
    pagination.value = {
      page: pg.page,
      rowsPerPage: pg.rowsPerPage,
      rowsNumber: res.total ?? rows.value.length,
    };
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : i18nT('raw.s4e3ce3');
  } finally {
    loading.value = false;
  }
}

function onRequest(props: TableRequestProps) {
  void load(props.pagination);
}

function onStatusFilter() {
  void load({ page: 1, rowsPerPage: pagination.value.rowsPerPage });
}

// ── 新建工单 ──

const createDialog = ref(false);
const creating = ref(false);
const createForm = ref<{ title: string; description: string; priority: TicketPriority }>({
  title: '',
  description: '',
  priority: 'Medium',
});

function openCreate() {
  createForm.value = { title: '', description: '', priority: 'Medium' };
  createDialog.value = true;
}

async function createTicket() {
  if (!createForm.value.title.trim()) return;
  creating.value = true;
  try {
    const payload: { title: string; description?: string; priority: string } = {
      title: createForm.value.title.trim(),
      priority: createForm.value.priority,
    };
    if (createForm.value.description.trim())
      payload.description = createForm.value.description.trim();
    await ticketsApi.create(payload);
    createDialog.value = false;
    toast('positive', i18nT('raw.s9ab0c4'));
    await load();
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : i18nT('raw.sb62cb3');
  } finally {
    creating.value = false;
  }
}

// ── 详情 ──

const detailDialog = ref(false);
const detailId = ref('');

function onRowClick(_evt: Event, row: TicketItem) {
  detailId.value = row.id;
  detailDialog.value = true;
}

function onDetailUpdated() {
  void load();
}

onMounted(() => {
  void load();
});
</script>
