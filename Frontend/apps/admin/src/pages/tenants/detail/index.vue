<template>
  <q-page class="erp-page">
    <q-breadcrumbs class="q-mb-sm">
      <q-breadcrumbs-el :label="i18nT('breadcrumb_tenants')" to="/tenants" />
      <q-breadcrumbs-el :label="i18nT('breadcrumb_tenant_detail')" />
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
      <div class="text-h5 q-ml-sm">{{ i18nT('raw.s6d80c3') }}</div>
      <q-space />
      <template v-if="tenant">
        <q-btn
          flat
          :color="tenant.isActive ? 'negative' : 'positive'"
          :icon="tenant.isActive ? 'pause' : 'play_arrow'"
          :label="tenant.isActive ? i18nT('raw.sd64e4d') : i18nT('raw.sc8a730')"
          :loading="toggling"
          @click="toggle"
        />
        <q-btn
          flat
          color="primary"
          icon="event_repeat"
          :label="i18nT('raw.sbe9595')"
          class="q-ml-sm"
          :loading="renewing"
          @click="renew"
        />
      </template>
    </div>

    <q-card v-if="tenant">
      <q-card-section class="row items-center">
        <q-avatar size="48px" color="teal" text-color="white" icon="domain" class="q-mr-md" />
        <div>
          <div class="text-h6">{{ tenant.name }}</div>
          <div class="text-caption text-grey-6">ID: {{ tenant.id }}</div>
        </div>
      </q-card-section>
      <q-separator />
      <q-card-section>
        <q-list dense padding>
          <q-item>
            <q-item-section avatar><q-icon name="tag" color="teal" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s1fa927') }}</q-item-label>
              <q-item-label>{{ tenant.id }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="domain" color="teal" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.sf43de3') }}</q-item-label>
              <q-item-label>{{ tenant.name }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="mail" color="teal" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s5ef54b') }}</q-item-label>
              <q-item-label>{{ tenant.adminEmail || '—' }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="event" color="teal" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s466980') }}</q-item-label>
              <q-item-label>{{ formatDate(tenant.validUpto) }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="link" color="teal" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s9f7cb2') }}</q-item-label>
              <q-item-label class="ellipsis">{{ tenant.connectionString || '—' }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar
              ><q-icon name="power_settings_new" color="teal"
            /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s9fb403') }}</q-item-label>
              <q-item-label>
                <q-badge :color="tenant.isActive ? 'green' : 'red'">{{
                  tenant.isActive ? i18nT('raw.sc8a730') : i18nT('raw.sd64e4d')
                }}</q-badge>
              </q-item-label>
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>
    </q-card>

    <div v-if="loading" class="q-pa-xl column items-center text-grey-6">
      <q-spinner size="40px" class="q-mb-sm" />
      <div>{{ i18nT('raw.s795a79') }}</div>
    </div>
    <EmptyState v-else-if="!tenant" :title="i18nT('raw.s8b2ae9')" :hint="i18nT('raw.s9593a7')" />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { formatDate } from '@erp-new-frontend-monorepo/utils';
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { tenantsApi } from '@/api';
import { EmptyState } from '@erp-new-frontend-monorepo/components';

const route = useRoute();
const router = useRouter();

const id = String((route.params as { id?: string }).id ?? '');

interface Tenant {
  id: string;
  name?: string;
  adminEmail?: string;
  validUpto?: string;
  isActive?: boolean;
  connectionString?: string;
}

const tenant = ref<Tenant | null>(null);
const loading = ref(false);
const toggling = ref(false);
const renewing = ref(false);

async function load() {
  loading.value = true;
  try {
    const all = (await tenantsApi.list()) as Tenant[];
    tenant.value = all.find((t) => String(t.id) === id) ?? null;
  } finally {
    loading.value = false;
  }
}

async function toggle() {
  if (!tenant.value) return;
  toggling.value = true;
  try {
    await tenantsApi.toggle(tenant.value.id, !tenant.value.isActive);
    await load();
  } finally {
    toggling.value = false;
  }
}

async function renew() {
  if (!tenant.value) return;
  renewing.value = true;
  try {
    await tenantsApi.renew(tenant.value.id);
    await load();
  } finally {
    renewing.value = false;
  }
}

onMounted(load);
</script>
