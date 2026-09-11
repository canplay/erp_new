<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-md">{{ t('report.editReport') }}</div>
    <q-card>
      <q-card-section>
        <q-form>
          <q-input v-model="formData.name" :label="t('common.reportTitle')" outlined :rules="[v => !!v || t('common.required')]" />
          <q-input v-model="formData.description" :label="t('common.description')" outlined type="textarea" rows="3" class="q-mt-md" />
          <q-select
            v-model="formData.reportType"
            :options="typeOptions"
            emit-value
            map-options
            :label="t('common.type')"
            outlined
            class="q-mt-md"
          />
        </q-form>
      </q-card-section>
      <q-card-actions align="right">
        <q-btn flat :label="t('common.cancel')" v-close-popup />
        <q-btn color="primary" :label="t('common.save')" @click="handleSave" />
      </q-card-actions>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { getReport, updateReport } from '@/api/report';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const $q = useQuasar();

const formData = ref({
  name: '',
  description: '',
  reportType: 'table',
});

const typeOptions = [
  { label: t('report.table'), value: 'table' },
  { label: t('report.chart'), value: 'chart' },
  { label: t('report.dashboard'), value: 'dashboard' },
];

async function handleSave() {
  try {
    await updateReport(route.params.id as string, formData.value);
    $q.notify({ type: 'positive', message: t('report.reportUpdated') });
    router.back();
  } catch {
    $q.notify({ type: 'negative', message: t('report.reportDeleteFailed') });
  }
}

onMounted(async () => {
  try {
    const response = await getReport(route.params.id as string);
    const respData = response as { data?: { data?: { name: string; description?: string; reportType: string } } };
    const report = respData.data?.data;
    if (report) {
      formData.value = {
        name: report.name,
        description: report.description || '',
        reportType: report.reportType,
      };
    }
  } catch {
    $q.notify({ type: 'negative', message: t('error.loadingFailed') });
  }
});
</script>