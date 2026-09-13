<!-- @layout main -->
<template>
  <q-page class="q-pa-md erp-page">
    <div class="text-h5 q-mb-md">{{ i18nT('raw.s77ff6b') }}</div>
    <KpiGrid :items="kpis" />

    <div class="text-subtitle1 q-mt-lg q-mb-sm">{{ i18nT('raw.s960172') }}</div>
    <q-card flat bordered class="erp-card">
      <q-table
        :rows="tenants"
        :columns="tenantColumns"
        row-key="id"
        :loading="loading"
        :pagination="{ page: 1, rowsPerPage: 5 }"
        hide-bottom
      />
    </q-card>

    <div class="text-subtitle1 q-mt-lg q-mb-sm">演示租户账号一览</div>
    <q-card flat bordered class="erp-card">
      <q-card-section>
        <q-markup-table flat bordered separator="cell">
          <thead>
            <tr>
              <th class="text-left">租户</th>
              <th class="text-left">租户代号（登录填）</th>
              <th class="text-left">管理员账号</th>
              <th class="text-left">密码</th>
              <th class="text-left">说明</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="acct in demoAccounts" :key="acct.tenant">
              <td>{{ acct.tenant }}</td>
              <td>
                <code>{{ acct.key }}</code>
              </td>
              <td>{{ acct.email }}</td>
              <td>
                <code>{{ acct.password }}</code>
              </td>
              <td class="text-grey-7">{{ acct.note }}</td>
            </tr>
          </tbody>
        </q-markup-table>
        <div class="text-caption text-grey-6 q-mt-sm">
          提示：企业租户登录需先填「租户代号」再填邮箱密码；平台账号租户代号填 <code>root</code>。
        </div>
      </q-card-section>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, onMounted } from 'vue';
import { KpiGrid } from '@erp-new-frontend-monorepo/components';
import { tenantsApi, billingApi, identityApi, ticketsApi } from '@/api';
import { formatDate } from '@erp-new-frontend-monorepo/utils';
import { useTableState } from '@erp-new-frontend-monorepo/composables';

interface TenantSummary {
  id: string;
  name: string;
  identifier?: string;
  createdAtUtc?: string;
  isActive?: boolean;
}

/** 演示租户账号（seed-demo 注入，密码统一 Seed:DemoPassword） */
const demoAccounts = [
  {
    tenant: '平台（Root）',
    key: 'root',
    email: 'superadmin@root.com',
    password: 'Password123!',
    note: '平台管理员',
  },
  {
    tenant: 'Acme 制造',
    key: 'acme',
    email: 'admin@acme.com',
    password: 'Password123!',
    note: '企业租户（含 alice/bob/carol）',
  },
  {
    tenant: 'Globex 能源',
    key: 'globex',
    email: 'admin@globex.com',
    password: 'Password123!',
    note: '企业租户（含 dave/erin）',
  },
];

interface KpiItem {
  label: string;
  value: string | number;
  icon: string;
  color: 'blue' | 'green' | 'orange' | 'red' | 'teal' | 'purple';
}
const kpis = ref<KpiItem[]>([
  { label: i18nT('raw.saea7d0'), value: '—', icon: 'apartment', color: 'blue' },
  { label: i18nT('raw.s09138c'), value: '—', icon: 'group', color: 'green' },
  { label: i18nT('raw.s9de88d'), value: '—', icon: 'receipt_long', color: 'orange' },
  { label: i18nT('raw.sfdf561'), value: '—', icon: 'support_agent', color: 'red' },
]);

const tenantColumns = [
  { name: 'name', label: i18nT('raw.s28dbc7'), field: 'name' },
  { name: 'identifier', label: i18nT('raw.sc34f03'), field: 'identifier' },
  {
    name: 'createdAtUtc',
    label: i18nT('raw.saf9956'),
    field: 'createdAtUtc',
    format: (v: string) => formatDate(v),
  },
  { name: 'isActive', label: i18nT('raw.s9fb403'), field: 'isActive' },
];

const { rows: tenants, loading, load } = useTableState<TenantSummary>();

onMounted(() => {
  void load(() =>
    Promise.allSettled([
      tenantsApi.list(),
      identityApi.users(1, 1),
      billingApi.invoices(1, 1),
      ticketsApi.list({ page: 1, pageSize: 1 }),
    ]).then(([t, u, inv, tk]) => {
      if (t.status === 'fulfilled') {
        const arr = t.value as TenantSummary[];
        kpis.value[0]!.value = arr.length;
        tenants.value = arr.slice(0, 5);
      }
      if (u.status === 'fulfilled') kpis.value[1]!.value = u.value.total ?? '—';
      if (inv.status === 'fulfilled') kpis.value[2]!.value = inv.value.total ?? '—';
      if (tk.status === 'fulfilled') kpis.value[3]!.value = tk.value.total ?? '—';
    }),
  );
});
</script>
