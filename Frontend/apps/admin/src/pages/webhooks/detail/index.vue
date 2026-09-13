<template>
  <q-page class="erp-page">
    <q-breadcrumbs class="q-mb-sm">
      <q-breadcrumbs-el label="Webhook 管理" to="/webhooks" />
      <q-breadcrumbs-el label="Webhook 详情" />
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
      <div class="text-h5 q-ml-sm">{{ i18nT('raw.s57ad4b') }}</div>
      <q-space />
      <q-btn
        v-if="sub"
        flat
        color="negative"
        icon="delete"
        :label="i18nT('raw.sbd7449')"
        :loading="deleting"
        @click="remove"
      />
    </div>

    <q-card v-if="sub">
      <q-card-section class="row items-center">
        <q-avatar size="48px" color="purple" text-color="white" icon="webhook" class="q-mr-md" />
        <div>
          <div class="text-h6 ellipsis" style="max-width: 640px">{{ sub.url }}</div>
          <div class="text-caption text-grey-6">ID: {{ sub.id }}</div>
        </div>
      </q-card-section>
      <q-separator />
      <q-card-section>
        <q-list dense padding>
          <q-item>
            <q-item-section avatar><q-icon name="link" color="purple" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.sea942c') }}</q-item-label>
              <q-item-label class="ellipsis">{{ sub.url }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item v-if="sub.secret">
            <q-item-section avatar><q-icon name="key" color="purple" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.sdaa4eb') }}</q-item-label>
              <q-item-label>{{ sub.secret }}</q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar><q-icon name="category" color="purple" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s7066f7') }}</q-item-label>
              <q-item-label>
                <div v-if="eventList.length" class="row q-gap-xs">
                  <q-chip
                    v-for="e in eventList"
                    :key="e"
                    size="xs"
                    color="purple"
                    text-color="white"
                    :label="e"
                  />
                </div>
                <span v-else>—</span>
              </q-item-label>
            </q-item-section>
          </q-item>
          <q-item>
            <q-item-section avatar
              ><q-icon name="power_settings_new" color="purple"
            /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.s9fb403') }}</q-item-label>
              <q-item-label>
                <q-badge :color="isActive ? 'green' : 'grey'">{{
                  isActive ? i18nT('raw.sc8a730') : i18nT('raw.sd64e4d')
                }}</q-badge>
              </q-item-label>
            </q-item-section>
          </q-item>
          <q-item v-if="sub.createdAtUtc">
            <q-item-section avatar><q-icon name="schedule" color="purple" /></q-item-section>
            <q-item-section>
              <q-item-label caption>{{ i18nT('raw.saf9956') }}</q-item-label>
              <q-item-label>{{ formatDate(sub.createdAtUtc) }}</q-item-label>
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>
    </q-card>

    <div v-if="loading" class="q-pa-xl column items-center text-grey-6">
      <q-spinner size="40px" class="q-mb-sm" />
      <div>{{ i18nT('raw.s795a79') }}</div>
    </div>
    <EmptyState v-else-if="!sub" :title="i18nT('raw.sc5dedd')" :hint="i18nT('raw.scd4299')" />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { formatDate } from '@erp-new-frontend-monorepo/utils';
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { webhooksApi } from '@/api';
import { EmptyState } from '@erp-new-frontend-monorepo/components';

const route = useRoute();
const router = useRouter();

const id = String((route.params as { id?: string }).id ?? '');

interface WebhookSubscription {
  id: string;
  url?: string;
  secret?: string;
  active?: boolean;
  isActive?: boolean;
  eventTypes?: string | string[];
  events?: string[];
  createdAtUtc?: string;
}

const sub = ref<WebhookSubscription | null>(null);
const loading = ref(false);
const deleting = ref(false);

const isActive = computed(() => Boolean(sub.value && (sub.value.active ?? sub.value.isActive)));
const eventList = computed<string[]>(() => {
  if (!sub.value) return [];
  if (Array.isArray(sub.value.eventTypes)) return sub.value.eventTypes;
  if (typeof sub.value.eventTypes === 'string')
    return sub.value.eventTypes
      .split(',')
      .map((s: string) => s.trim())
      .filter(Boolean);
  if (Array.isArray(sub.value.events)) return sub.value.events;
  return [];
});

async function load() {
  loading.value = true;
  try {
    const res = await webhooksApi.subscriptions();
    const all = (res.items ?? []) as WebhookSubscription[];
    sub.value = all.find((s) => String(s.id) === id) ?? null;
  } finally {
    loading.value = false;
  }
}

async function remove() {
  if (!sub.value) return;
  deleting.value = true;
  try {
    await webhooksApi.remove(sub.value.id);
    router.back();
  } finally {
    deleting.value = false;
  }
}

onMounted(load);
</script>
