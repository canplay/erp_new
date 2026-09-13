<template>
  <div>
    <div class="text-subtitle2 q-mb-xs">
      {{ i18nT('raw.stk_comments', { count: comments.length }) }}
    </div>
    <div v-if="loadingComments" class="text-grey-7 q-mb-sm">
      {{ i18nT('raw.s795a79') }}
    </div>
    <div v-else-if="comments.length === 0" class="text-grey-6 q-mb-sm">
      {{ i18nT('raw.s6d7c45') }}
    </div>
    <div v-else class="q-gutter-y-sm q-mb-sm">
      <div v-for="c in comments" :key="c.id" class="bg-grey-1 rounded-borders q-pa-sm">
        <div class="row items-center">
          <span
            class="text-weight-medium text-caption ellipsis"
            style="max-width: 40%"
            :title="c.authorUserId"
          >
            {{ c.authorUserId }}
          </span>
          <q-space />
          <span class="text-caption text-grey-6">{{ formatDate(c.createdAtUtc) }}</span>
        </div>
        <div class="q-mt-xs text-body2" style="white-space: pre-wrap">
          {{ c.body }}
        </div>
      </div>
    </div>
    <div class="row items-end q-gutter-sm">
      <q-input
        v-model="newComment"
        :label="i18nT('raw.sb00f58')"
        filled
        type="textarea"
        class="col"
        :disable="isClosed"
        :hint="isClosed ? i18nT('raw.s360aec') : undefined"
      />
      <q-btn
        color="primary"
        icon="send"
        :label="i18nT('raw.s0e4ac2')"
        :loading="posting"
        :disable="!newComment.trim()"
        @click="onSubmit"
        :title="i18nT('raw.stk_send')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { ticketsApi } from '@/api';
import { formatDate } from '@erp-new-frontend-monorepo/utils';

interface TicketComment {
  id: string;
  ticketId?: string;
  authorUserId: string;
  body: string;
  createdAtUtc: string;
}

const props = defineProps<{
  ticketId: string | undefined;
  isClosed: boolean;
}>();

const emit = defineEmits<{
  (e: 'updated'): void;
}>();

const $q = useQuasar();

function toast(type: 'positive' | 'negative', message: string) {
  $q.notify({ type, message });
}

const comments = ref<TicketComment[]>([]);
const loadingComments = ref(false);
const newComment = ref('');
const posting = ref(false);

async function loadComments() {
  if (!props.ticketId) return;
  loadingComments.value = true;
  try {
    comments.value = (await ticketsApi.comments(props.ticketId)) as TicketComment[];
  } catch (e) {
    toast('negative', e instanceof Error ? e.message : i18nT('raw.sa97bcc'));
  } finally {
    loadingComments.value = false;
  }
}

async function onSubmit() {
  const body = newComment.value.trim();
  if (!body || !props.ticketId) return;
  posting.value = true;
  try {
    await ticketsApi.addComment(props.ticketId, body);
    newComment.value = '';
    toast('positive', i18nT('raw.sca639c'));
    await loadComments();
    emit('updated');
  } catch (e) {
    toast('negative', e instanceof Error ? e.message : i18nT('raw.s45becc'));
  } finally {
    posting.value = false;
  }
}

defineExpose({
  loadComments,
  reload: loadComments,
});
</script>
