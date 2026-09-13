<template>
  <q-page class="erp-page">
    <div class="row items-center justify-between q-mb-sm">
      <div class="text-h5">{{ i18nT('raw.s69eacf') }}</div>
      <div class="row items-center">
        <q-badge v-if="unreadCount > 0" color="red" rounded class="q-mr-sm">
          {{ unreadCount }} 条未读
        </q-badge>
        <q-btn
          color="primary"
          outline
          dense
          icon="done_all"
          :label="i18nT('raw.s77daf1')"
          :disable="unreadCount === 0"
          :loading="markingAll"
          @click="markAllRead"
        />
      </div>
    </div>

    <q-tabs
      v-model="filter"
      class="q-mb-md"
      inline-label
      active-color="primary"
      @update:model-value="onFilterChange"
    >
      <q-tab name="all" icon="inbox" :label="i18nT('raw.s022ee7')" />
      <q-tab name="unread" icon="mark_email_unread" :label="i18nT('raw.s222176')" />
    </q-tabs>

    <q-card flat bordered>
      <q-inner-loading :showing="loading" />
      <template v-if="!loading">
        <q-list v-if="messages.length" separator>
          <q-item
            v-for="m in messages"
            :key="m.id"
            class="notification-item"
            :class="{ unread: !m.readAtUtc }"
          >
            <q-item-section avatar>
              <q-icon
                :name="m.readAtUtc ? 'mail' : 'mark_email_unread'"
                :color="m.readAtUtc ? 'grey-5' : 'primary'"
              />
            </q-item-section>
            <q-item-section>
              <q-item-label :class="{ 'text-weight-bold': !m.readAtUtc }">{{
                m.title
              }}</q-item-label>
              <q-item-label caption lines="2">{{ m.body || '—' }}</q-item-label>
            </q-item-section>
            <q-item-section side class="col-auto items-end">
              <q-item-label caption>{{ formatTime(m.createdAtUtc) }}</q-item-label>
              <q-btn
                v-if="!m.readAtUtc"
                flat
                dense
                size="sm"
                color="primary"
                icon="done"
                :label="i18nT('raw.sdd5652')"
                :loading="markingId === m.id"
                @click="markRead(m)"
              />
            </q-item-section>
          </q-item>
        </q-list>
        <EmptyState
          v-else
          :icon="filter === 'unread' ? 'mark_email_read' : 'inbox'"
          :title="filter === 'unread' ? i18nT('raw.s6014cb') : i18nT('raw.s1ea6f2')"
          :hint="i18nT('raw.sa6a8af')"
        />
      </template>
    </q-card>

    <div v-if="!loading && messages.length" class="row items-center justify-center q-mt-md">
      <q-btn
        flat
        dense
        icon="chevron_left"
        :label="i18nT('raw.sb93119')"
        :disable="page <= 1"
        @click="changePage(page - 1)"
      />
      <span class="q-mx-md text-grey-8">第 {{ page }} 页</span>
      <q-btn
        flat
        dense
        :label="i18nT('raw.sce3aad')"
        icon-right="chevron_right"
        :disable="!hasMore"
        @click="changePage(page + 1)"
      />
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref, onMounted } from 'vue';
import { getAlova } from '@erp-new-frontend-monorepo/boot/alova';
import { EmptyState } from '@erp-new-frontend-monorepo/components';
import { notificationsApi, type AdminNotification } from '@/api';

type Filter = 'all' | 'unread';

const PAGE_SIZE = 20;

const filter = ref<Filter>('all');
const messages = ref<AdminNotification[]>([]);
const unreadCount = ref(0);
const loading = ref(false);
const markingId = ref<string | null>(null);
const markingAll = ref(false);
const page = ref(1);
const hasMore = ref(false);

async function listNotifications() {
  const a = getAlova() as unknown as { Get<T = unknown>(u: string, config?: unknown): Promise<T> };
  const params: Record<string, unknown> = { page: page.value, pageSize: PAGE_SIZE };
  if (filter.value === 'unread') params.unreadOnly = true;
  const res = await a.Get<AdminNotification[]>('/api/v1/notifications', { params });
  return Array.isArray(res) ? res : [];
}

async function refreshUnreadCount() {
  try {
    const res = await notificationsApi.unreadCount();
    unreadCount.value = res?.count ?? 0;
  } catch {
    unreadCount.value = 0;
  }
}

async function fetchList() {
  loading.value = true;
  try {
    messages.value = await listNotifications();
    hasMore.value = messages.value.length >= PAGE_SIZE;
  } finally {
    loading.value = false;
  }
}

function onFilterChange() {
  page.value = 1;
  void fetchList();
}

async function changePage(next: number) {
  if (next < 1 || loading.value) return;
  page.value = next;
  await fetchList();
}

async function markRead(m: AdminNotification) {
  markingId.value = m.id;
  try {
    await notificationsApi.markRead(m.id);
    m.readAtUtc = new Date().toISOString();
    unreadCount.value = Math.max(0, unreadCount.value - 1);
    if (filter.value === 'unread') {
      messages.value = messages.value.filter((x) => x.id !== m.id);
      hasMore.value = messages.value.length >= PAGE_SIZE;
    }
  } finally {
    markingId.value = null;
  }
}

async function markAllRead() {
  markingAll.value = true;
  try {
    await notificationsApi.markAllRead();
    unreadCount.value = 0;
    if (filter.value === 'unread') {
      messages.value = [];
    } else {
      messages.value = messages.value.map((x) => ({
        ...x,
        readAtUtc: x.readAtUtc ?? new Date().toISOString(),
      }));
    }
  } finally {
    markingAll.value = false;
  }
}

function formatTime(iso?: string) {
  if (!iso) return '';
  try {
    const d = new Date(iso);
    const now = new Date();
    const sameDay = d.toDateString() === now.toDateString();
    return sameDay
      ? d.toTimeString().slice(0, 5)
      : d.toLocaleString('zh-CN', {
          month: '2-digit',
          day: '2-digit',
          hour: '2-digit',
          minute: '2-digit',
        });
  } catch {
    return iso;
  }
}

onMounted(() => {
  void refreshUnreadCount();
  void fetchList();
});
</script>

<style scoped>
.notification-item.unread {
  background: #f5faff;
}
</style>
