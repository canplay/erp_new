<template>
  <q-card flat bordered erp-card class="q-mt-md">
    <q-card-section>
      <div class="text-subtitle1">{{ i18nT('raw.s703acc') }}</div>
      <div class="row q-col-gutter-sm q-mt-xs">
        <div class="col-12 col-md-6">
          <q-input
            v-model="form.referenceNo"
            :label="i18nT('raw.s6009b0')"
            dense
            outlined
            :rules="[(v: string) => !!v?.trim() || i18nT('raw.s6a17dd')]"
            lazy-rules
          />
        </div>
        <div class="col-12 col-md-6">
          <q-input
            v-model="form.replyToReferenceNo"
            :label="i18nT('raw.s245261')"
            dense
            outlined
            :placeholder="i18nT('raw.sbe4a9f')"
          />
        </div>
        <div class="col-12">
          <q-input
            v-model="form.title"
            :label="i18nT('raw.s4f075a')"
            dense
            outlined
            :rules="[(v: string) => !!v?.trim() || i18nT('raw.s6a17dd')]"
            lazy-rules
          />
        </div>
        <div class="col-12">
          <q-input
            v-model="form.content"
            :label="i18nT('raw.sccd37a')"
            type="textarea"
            dense
            outlined
            rows="3"
            :rules="[(v: string) => !!v?.trim() || i18nT('raw.s6a17dd')]"
            lazy-rules
          />
        </div>
        <div class="col-12">
          <q-btn
            color="primary"
            icon="send"
            :label="i18nT('raw.sc6fce4')"
            :loading="submitting"
            @click="onSubmit"
            :title="'发送'"
          />
        </div>
      </div>
    </q-card-section>
    <q-card-section v-if="submitted.length" class="q-pt-none">
      <div class="text-caption text-grey-7 q-mb-xs">{{ i18nT('raw.s071d04') }}</div>
      <q-list dense bordered separator>
        <q-item v-for="f in submitted" :key="f.id">
          <q-item-section>
            <q-item-label>{{ f.title }}</q-item-label>
            <q-item-label caption>
              #{{ f.referenceNo }} · {{ formatDateTime(f.feedbackAt) }} ·
              {{ i18nT('raw.s113ret') }}: {{ f.id }}
            </q-item-label>
          </q-item-section>
        </q-item>
      </q-list>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { regulatoryApi } from '@/api';
import type { RegulatoryFeedbackRecordItem } from '@/api';
import { formatDateTime } from '@erp-new-frontend-monorepo/utils';

const $q = useQuasar();

const form = ref<{
  referenceNo: string;
  title: string;
  content: string;
  replyToReferenceNo?: string;
}>({ referenceNo: '', title: '', content: '', replyToReferenceNo: '' });

const submitting = ref(false);
const submitted = ref<RegulatoryFeedbackRecordItem[]>([]);

async function onSubmit() {
  if (!form.value.referenceNo.trim() || !form.value.title.trim() || !form.value.content.trim()) {
    $q.notify({ type: 'warning', message: i18nT('raw.sa0f86d') });
    return;
  }
  submitting.value = true;
  try {
    const res = await regulatoryApi.feedback({
      referenceNo: form.value.referenceNo.trim(),
      title: form.value.title.trim(),
      content: form.value.content.trim(),
      replyToReferenceNo: form.value.replyToReferenceNo?.trim() || null,
    });
    submitted.value.unshift({
      id: res.id,
      referenceNo: form.value.referenceNo.trim(),
      title: form.value.title.trim(),
      content: form.value.content.trim(),
      replyToReferenceNo: form.value.replyToReferenceNo?.trim() || null,
      feedbackAt: new Date().toISOString(),
    });
    $q.notify({ type: 'positive', message: `${i18nT('raw.s253fbk')}: ${res.id}` });
    form.value = { referenceNo: '', title: '', content: '', replyToReferenceNo: '' };
  } catch (e) {
    $q.notify({
      type: 'negative',
      message: `${i18nT('raw.s256sub')}: ${e instanceof Error ? e.message : String(e)}`,
    });
  } finally {
    submitting.value = false;
  }
}
</script>
