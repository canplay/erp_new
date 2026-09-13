<template>
  <q-dialog v-model="dialog">
    <q-card
      style="
        min-width: 680px;
        max-width: 92vw;
        width: 720px;
        display: flex;
        flex-direction: column;
        max-height: 85vh;
      "
    >
      <q-card-section class="row items-start q-gutter-sm">
        <div class="col">
          <div class="text-h6 ellipsis">{{ current?.title || i18nT('raw.s3ed272') }}</div>
          <div class="q-mt-xs">
            <q-badge :color="ticketStatusColor(current?.status)" text-color="white" class="q-mr-sm">
              {{ statusLabel(current?.status) }}
            </q-badge>
            <q-badge :color="ticketPriorityColor(current?.priority)" outline>
              {{ priorityLabel(current?.priority) }}
            </q-badge>
            <span class="q-ml-sm text-caption text-grey-7">
              <code>{{ current?.number }}</code>
            </span>
          </div>
        </div>
        <q-btn flat round dense icon="close" v-close-popup />
      </q-card-section>

      <q-separator />

      <q-card-section class="q-pt-md" style="flex: 1 1 auto; min-height: 0; display: flex"
        ><q-scroll-area class="col">
          <div v-if="loadingDetail" class="text-grey-7 q-pa-sm">{{ i18nT('raw.s795a79') }}</div>
          <template v-else-if="current">
            <div class="row q-col-gutter-y-md q-mb-md">
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s333f06') }}</div>
                <div class="text-body2 ellipsis" :title="current.reporterUserId || ''">
                  {{ current.reporterUserId }}
                </div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s82a318') }}</div>
                <div class="text-body2 ellipsis" :title="current.assignedToUserId || ''">
                  {{ current.assignedToUserId || i18nT('raw.s4bb3fa') }}
                </div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s247bbb') }}</div>
                <div class="text-body2">{{ current.commentCount ?? 0 }}</div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.saf9956') }}</div>
                <div class="text-body2">{{ formatDate(current.createdAtUtc) }}</div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.sfda53d') }}</div>
                <div class="text-body2">{{ formatDate(current.updatedAtUtc) }}</div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s10221a') }}</div>
                <div class="text-body2">{{ formatDate(current.resolvedAtUtc) }}</div>
              </div>
              <div class="col-6 col-sm-4">
                <div class="text-caption text-grey-7">{{ i18nT('raw.s25b138') }}</div>
                <div class="text-body2">{{ formatDate(current.closedAtUtc) }}</div>
              </div>
            </div>

            <div class="q-mb-md">
              <div class="text-subtitle2 q-mb-xs">{{ i18nT('raw.s9e58f1') }}</div>
              <p v-if="current.description" class="text-body2" style="white-space: pre-wrap">
                {{ current.description }}
              </p>
              <p v-else class="text-grey-6">{{ i18nT('raw.s977b1b') }}</p>
            </div>

            <div v-if="current.resolutionNote" class="q-mb-md">
              <div class="text-subtitle2 q-mb-xs">{{ i18nT('raw.s27f7f2') }}</div>
              <p class="text-body2" style="white-space: pre-wrap">
                {{ current.resolutionNote }}
              </p>
            </div>

            <TicketCommentsSection
              ref="commentsRef"
              :ticket-id="ticketId"
              :is-closed="isClosed"
              @updated="onCommentUpdated"
            />
          </template> </q-scroll-area
      ></q-card-section>

      <q-separator />
      <q-card-actions align="right" class="q-pa-md">
        <q-btn outline icon="assignment_ind" :label="i18nT('raw.s58d308')" @click="openAssign" />
        <q-btn
          v-if="canResolve"
          outline
          icon="task_alt"
          :label="i18nT('raw.s940291')"
          @click="openResolve"
        />
        <q-btn
          v-if="!isClosed"
          outline
          icon="close"
          :label="i18nT('raw.sdedda3')"
          :loading="closing"
          @click="closeTicket"
        />
        <q-btn
          v-if="canReopen"
          outline
          icon="replay"
          :label="i18nT('raw.s8143e5')"
          :loading="reopening"
          @click="reopenTicket"
        />
        <q-btn :label="i18nT('raw.s94320b')" v-close-popup />
      </q-card-actions>
    </q-card>
  </q-dialog>

  <!-- 指派 -->
  <q-dialog v-model="assignDialog">
    <q-card style="min-width: 420px">
      <q-card-section class="text-h6">{{ i18nT('raw.s3fe502') }}</q-card-section>
      <q-card-section>
        <q-input v-model="assigneeId" :label="i18nT('raw.sb99d44')" filled dense autofocus />
        <div class="text-caption text-grey-7 q-mt-sm">
          当前指派人：{{ current?.assignedToUserId || i18nT('raw.s4bb3fa') }}
        </div>
      </q-card-section>
      <q-card-actions align="right">
        <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
        <q-btn
          :label="i18nT('raw.sed2079')"
          color="primary"
          :loading="assigning"
          :disable="!assigneeId.trim()"
          @click="doAssign"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>

  <!-- 解决 -->
  <q-dialog v-model="resolveDialog">
    <q-card style="min-width: 480px">
      <q-card-section class="text-h6">{{ i18nT('raw.sb79420') }}</q-card-section>
      <q-card-section>
        <q-input
          v-model="resolveNote"
          :label="i18nT('raw.sf332eb')"
          filled
          type="textarea"
          autofocus
          maxlength="4096"
        />
      </q-card-section>
      <q-card-actions align="right">
        <q-btn :label="i18nT('raw.s451c97')" v-close-popup />
        <q-btn
          :label="i18nT('raw.sd45ff8')"
          color="primary"
          :loading="resolving"
          @click="doResolve"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { formatDate, ticketStatusColor, ticketPriorityColor } from '@erp-new-frontend-monorepo/utils';
import { computed, ref, watch } from 'vue';
import { useQuasar } from 'quasar';
import { ticketsApi } from '@/api';
import TicketCommentsSection from './TicketCommentsSection.vue';

type TicketStatus = 'Open' | 'InProgress' | 'Resolved' | 'Closed';
type TicketPriority = 'Low' | 'Medium' | 'High' | 'Critical';

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

const props = defineProps<{
  ticketId?: string;
  modelValue: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  updated: [];
}>();

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

function statusLabel(s?: TicketStatus | null) {
  return s ? STATUS_LABELS[s] : '—';
}

function priorityLabel(p?: TicketPriority | null) {
  return p ? PRIORITY_LABELS[p] : '—';
}

function toast(type: 'positive' | 'negative', message: string) {
  $q.notify({ type, message });
}

const dialog = computed({
  get: () => props.modelValue,
  set: (v: boolean) => emit('update:modelValue', v),
});

const current = ref<TicketItem | null>(null);
const loadingDetail = ref(false);

const canResolve = computed(
  () => current.value?.status !== 'Resolved' && current.value?.status !== 'Closed',
);
const canReopen = computed(
  () => current.value?.status === 'Resolved' || current.value?.status === 'Closed',
);
const isClosed = computed(() => current.value?.status === 'Closed');

watch(
  () => props.ticketId,
  (id) => {
    if (id && props.modelValue) {
      void loadData();
    }
  },
);

watch(
  () => props.modelValue,
  (v) => {
    if (v && props.ticketId) {
      void loadData();
    }
  },
);

async function loadData() {
  current.value = null;
  await loadDetail();
  commentsRef.value?.reload();
}

async function loadDetail() {
  if (!props.ticketId) return;
  loadingDetail.value = true;
  try {
    current.value = (await ticketsApi.detail(props.ticketId)) as TicketItem;
  } catch (e) {
    toast('negative', e instanceof Error ? e.message : i18nT('raw.seffcd5'));
  } finally {
    loadingDetail.value = false;
  }
}

function onCommentUpdated() {
  void loadDetail();
  emit('updated');
}

const commentsRef = ref<InstanceType<typeof TicketCommentsSection> | null>(null);

// ── 指派 ──

const assignDialog = ref(false);
const assigneeId = ref('');
const assigning = ref(false);

function openAssign() {
  assigneeId.value = current.value?.assignedToUserId ?? '';
  assignDialog.value = true;
}

async function doAssign() {
  const userId = assigneeId.value.trim();
  if (!userId || !props.ticketId) return;
  assigning.value = true;
  try {
    await ticketsApi.assign(props.ticketId, userId);
    assignDialog.value = false;
    toast('positive', i18nT('raw.seb2396'));
    await loadDetail();
    emit('updated');
  } catch (e) {
    toast('negative', e instanceof Error ? e.message : i18nT('raw.s81e6dd'));
  } finally {
    assigning.value = false;
  }
}

// ── 解决 ──

const resolveDialog = ref(false);
const resolveNote = ref('');
const resolving = ref(false);

function openResolve() {
  resolveNote.value = '';
  resolveDialog.value = true;
}

async function doResolve() {
  if (!props.ticketId) return;
  resolving.value = true;
  try {
    await ticketsApi.resolve(props.ticketId, resolveNote.value.trim() || undefined);
    resolveDialog.value = false;
    toast('positive', i18nT('raw.s517d3f'));
    await loadDetail();
    emit('updated');
  } catch (e) {
    toast('negative', e instanceof Error ? e.message : i18nT('raw.s0997ff'));
  } finally {
    resolving.value = false;
  }
}

// ── 关闭 / 重新打开 ──

const closing = ref(false);
const reopening = ref(false);

async function closeTicket() {
  if (!props.ticketId) return;
  closing.value = true;
  try {
    await ticketsApi.close(props.ticketId);
    toast('positive', i18nT('raw.s1ba20d'));
    await loadDetail();
    emit('updated');
  } catch (e) {
    toast('negative', e instanceof Error ? e.message : i18nT('raw.s3c04dd'));
  } finally {
    closing.value = false;
  }
}

async function reopenTicket() {
  if (!props.ticketId) return;
  reopening.value = true;
  try {
    await ticketsApi.reopen(props.ticketId);
    toast('positive', i18nT('raw.s8b5247'));
    await loadDetail();
    emit('updated');
  } catch (e) {
    toast('negative', e instanceof Error ? e.message : i18nT('raw.s3084c8'));
  } finally {
    reopening.value = false;
  }
}
</script>
