<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <div class="text-h5">{{ i18nT('raw.s74d534') }}</div>
      <q-space />
      <q-btn
        color="negative"
        outline
        icon="delete_sweep"
        :label="i18nT('raw.scb6c43')"
        @click="revokeAll"
      />
    </div>

    <PageTable
      :rows="rows"
      :columns="columns"
      :loading="loading"
      :total="total"
      @request="onRequest"
    >
      <template #body-cell-user="{ row }">
        <q-td>{{ row.userEmail || row.userName }}</q-td>
      </template>
      <template #body-cell-isActive="{ row }">
        <q-td>
          <q-badge :color="row.isActive ? 'green' : 'grey'">{{
            row.isActive ? i18nT('raw.s2f558e') : i18nT('raw.s18e264')
          }}</q-badge>
          <q-badge v-if="row.isCurrentSession" color="primary" class="q-ml-xs">{{
            i18nT('raw.s8867a9')
          }}</q-badge>
        </q-td>
      </template>
      <template #body-cell-actions="{ row }">
        <q-td>
          <q-btn
            flat
            dense
            icon="delete"
            size="sm"
            :disable="row.isCurrentSession"
            @click="revoke(row.id)"
            :title="i18nT('raw.s339696')"
          />
        </q-td>
      </template>
    </PageTable>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { onMounted } from 'vue';
import { sessionsApi } from '@/api';
import { PageTable } from '@erp-new-frontend-monorepo/components';
import { useTableState } from '@erp-new-frontend-monorepo/composables';

interface SessionRow {
  id: string;
  userEmail?: string;
  userName?: string;
  deviceType?: string;
  browser?: string;
  browserVersion?: string;
  ipAddress?: string;
  createdAt?: string;
  lastActivityAt?: string;
  isActive?: boolean;
  isCurrentSession?: boolean;
}

const columns = [
  { name: 'user', label: i18nT('raw.seaa2dd'), field: 'userEmail' },
  { name: 'deviceType', label: i18nT('raw.sf15d37'), field: 'deviceType' },
  {
    name: 'browser',
    label: i18nT('raw.s37e268'),
    field: (r: SessionRow) => `${r.browser || ''} ${r.browserVersion || ''}`,
  },
  { name: 'ipAddress', label: 'IP', field: 'ipAddress' },
  { name: 'createdAt', label: i18nT('raw.saf9956'), field: 'createdAt' },
  { name: 'lastActivityAt', label: i18nT('raw.s1c3d8f'), field: 'lastActivityAt' },
  { name: 'isActive', label: i18nT('raw.s9fb403'), field: 'isActive' },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions' },
];

const { rows, total, loading, load, page, size, goToPage, setSize } = useTableState<SessionRow>({
  size: 20,
});

async function doLoad() {
  await load(() =>
    sessionsApi.all(page.value, size.value).then((res: { items: SessionRow[]; total: number }) => {
      rows.value = res.items;
      total.value = res.total;
    }),
  );
}

function onRequest({ page: p, rowsPerPage }: { page: number; rowsPerPage: number }) {
  if (rowsPerPage !== size.value) {
    void setSize(rowsPerPage);
  } else {
    void goToPage(p);
  }
}

async function revoke(id: string) {
  await sessionsApi.revoke(id);
  await doLoad();
}

async function revokeAll() {
  await sessionsApi.revokeAll();
  await doLoad();
}

onMounted(doLoad);
</script>
