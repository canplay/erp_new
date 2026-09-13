<template>
  <q-page class="erp-page">
    <q-banner class="bg-primary text-white rounded-borders q-mb-md">
      <template #avatar>
        <q-icon name="manage_accounts" />
      </template>
      平台运营人员可通过i18nT('raw.sa73f08')以租户用户身份排查问题，操作全程留痕，请谨慎使用。
    </q-banner>

    <div class="text-h5 q-mb-md">{{ i18nT('raw.sb3e686') }}</div>

    <q-banner v-if="error" dense class="bg-negative text-white q-mb-md" icon="error">
      {{ error }}
      <template #action>
        <q-btn flat color="white" :label="i18nT('raw.s77af78')" @click="doLoad" />
      </template>
    </q-banner>

    <PageTable
      v-if="loading || rows.length > 0 || error"
      :rows="rows"
      :columns="columns"
      :loading="loading"
      :total="total"
      @request="onRequest"
    >
      <template #body-cell-actor="{ row }">
        <q-td>
          {{ row.actorUserName || row.actorUserId }}
          <div class="text-caption text-grey-6">{{ row.actorTenantId }}</div>
        </q-td>
      </template>
      <template #body-cell-target="{ row }">
        <q-td>
          {{ row.impersonatedUserName || row.impersonatedUserId }}
          <div class="text-caption text-grey-6">{{ row.impersonatedTenantId }}</div>
        </q-td>
      </template>
      <template #body-cell-status="{ row }">
        <q-td>
          <q-badge :color="statusColor(row.status)">{{ statusText(row.status) }}</q-badge>
        </q-td>
      </template>
      <template #body-cell-expires="{ row }">
        <q-td>{{ formatDate(row.expiresAtUtc) }}</q-td>
      </template>
      <template #body-cell-actions="{ row }">
        <q-td>
          <q-btn
            flat
            dense
            color="primary"
            icon="login"
            size="sm"
            :label="i18nT('raw.sa73f08')"
            :loading="starting === row.id"
            @click="start(row)"
          />
          <q-btn
            flat
            dense
            color="negative"
            icon="delete_sweep"
            size="sm"
            :loading="revoking === row.id"
            @click="revoke(row)"
            :title="i18nT('raw.seb0777')"
          />
        </q-td>
      </template>
    </PageTable>
    <EmptyState
      v-else
      :title="i18nT('raw.se0640b')"
      :hint="i18nT('raw.s81a5a1')"
      icon="manage_accounts"
    />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import {
  formatDate,
  impersonationStatusText as statusText,
  impersonationStatusColor as statusColor,
} from '@erp-new-frontend-monorepo/utils';
import { onMounted, ref } from 'vue';
import { useQuasar } from 'quasar';
import { impersonationApi } from '@/api';
import { EmptyState, PageTable } from '@erp-new-frontend-monorepo/components';
import { useTableState } from '@erp-new-frontend-monorepo/composables';

const $q = useQuasar();

interface ImpersonationGrant {
  id: string;
  actorUserId: string;
  actorUserName?: string;
  actorTenantId?: string;
  impersonatedUserId: string;
  impersonatedUserName?: string;
  impersonatedTenantId?: string;
  reason?: string;
  status?: string;
  expiresAtUtc?: string;
}

const columns = [
  { name: 'actor', label: i18nT('raw.s8a7b1e'), field: 'actorUserId' },
  { name: 'target', label: i18nT('raw.s87a895'), field: 'impersonatedUserId' },
  { name: 'reason', label: i18nT('raw.sf436e1'), field: 'reason' },
  { name: 'status', label: i18nT('raw.s9fb403'), field: 'status' },
  { name: 'expires', label: i18nT('raw.s466980'), field: 'expiresAtUtc' },
  { name: 'actions', label: i18nT('raw.s731dfc'), field: 'actions' },
];
const { rows, total, loading, error, load, page, size, goToPage, setSize } =
  useTableState<ImpersonationGrant>({ size: 20 });
const starting = ref<string | null>(null);
const revoking = ref<string | null>(null);

interface GrantsResponse {
  items: ImpersonationGrant[];
  total: number;
}

async function doLoad() {
  await load(() =>
    impersonationApi.grants(page.value, size.value).then((res: GrantsResponse) => {
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

async function start(grant: ImpersonationGrant) {
  starting.value = grant.id;
  try {
    await impersonationApi.start(grant.impersonatedUserId);
    $q.notify({
      type: 'positive',
      message: i18nT('raw.sdfc427'),
    });
  } finally {
    starting.value = null;
  }
}

async function revoke(grant: ImpersonationGrant) {
  revoking.value = grant.id;
  try {
    await impersonationApi.revoke(grant.id);
    await doLoad();
  } finally {
    revoking.value = null;
  }
}

onMounted(doLoad);
</script>
