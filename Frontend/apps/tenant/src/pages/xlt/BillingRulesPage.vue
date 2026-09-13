<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">{{ $t('parking.billingRulesTitle') }}</div>
    <div class="text-grey q-mb-md">{{ $t('parking.billingRulesDesc') }}</div>

    <q-card flat bordered>
      <q-card-section>
        <div class="text-h6">{{ $t('parking.defaultBillingRules') }}</div>
      </q-card-section>
      <q-separator />
      <q-card-section>
        <q-list dense>
          <q-item><q-item-section><q-item-label>{{ $t('parking.freeMinutes') }}</q-item-label><q-item-label caption>{{ rule.free_minutes }} {{ $t('common.ms') }}</q-item-label></q-item-section>
            <q-item-section side><q-btn flat dense icon="edit" size="sm" @click="editField('free_minutes')" /></q-item-section></q-item>
          <q-separator />
          <q-item><q-item-section><q-item-label>{{ $t('parking.firstHourFee') }}</q-item-label><q-item-label caption>{{ (rule.first_hour_fee / 100).toFixed(2) }} {{ $t('parking.yuan') }}</q-item-label></q-item-section>
            <q-item-section side><q-btn flat dense icon="edit" size="sm" @click="editField('first_hour_fee')" /></q-item-section></q-item>
          <q-separator />
          <q-item><q-item-section><q-item-label>{{ $t('parking.hourlyFeeAfter') }}</q-item-label><q-item-label caption>{{ (rule.hourly_fee / 100).toFixed(2) }} {{ $t('parking.yuan') }}</q-item-label></q-item-section>
            <q-item-section side><q-btn flat dense icon="edit" size="sm" @click="editField('hourly_fee')" /></q-item-section></q-item>
          <q-separator />
          <q-item><q-item-section><q-item-label>{{ $t('parking.dailyCap') }}</q-item-label><q-item-label caption>{{ (rule.daily_cap / 100).toFixed(2) }} {{ $t('parking.yuan') }}</q-item-label></q-item-section>
            <q-item-section side><q-btn flat dense icon="edit" size="sm" @click="editField('daily_cap')" /></q-item-section></q-item>
          <q-separator />
          <q-item><q-item-section><q-item-label>{{ $t('parking.nightFee') }}</q-item-label><q-item-label caption>{{ (rule.night_fee / 100).toFixed(2) }} {{ $t('parking.yuan') }}（{{ rule.night_start }}~{{ rule.night_end }}）</q-item-label></q-item-section>
            <q-item-section side><q-btn flat dense icon="edit" size="sm" @click="editField('night_fee')" /></q-item-section></q-item>
        </q-list>
      </q-card-section>
      <q-separator />
      <q-card-actions align="right">
        <q-btn color="primary" :label="$t('common.save')" @click="save" />
      </q-card-actions>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">import { useI18n } from 'vue-i18n';

import { reactive } from 'vue';
import { useQuasar } from 'quasar';
const { t } = useI18n();
const $q = useQuasar();
const rule = reactive({
  free_minutes: 15, first_hour_fee: 500, hourly_fee: 200,
  daily_cap: 3000, night_fee: 500, night_start: '22:00', night_end: '07:00',
});
function save() {
  $q.notify({ type: 'positive', message: t('parking.saveBillingRules') });
}

function editField(field: string) {
  $q.dialog({
    title: t('parking.editRule'),
    message: `${field}: `,
    prompt: {
      model: String(rule[field as keyof typeof rule]),
      type: 'text',
    },
    cancel: true,
    persistent: true,
  }).onOk((val: string) => {
    if (field === 'free_minutes') rule.free_minutes = Number(val);
    else if (field === 'first_hour_fee') rule.first_hour_fee = Number(val);
    else if (field === 'hourly_fee') rule.hourly_fee = Number(val);
    else if (field === 'daily_cap') rule.daily_cap = Number(val);
    else if (field === 'night_fee') rule.night_fee = Number(val);
    $q.notify({ type: 'positive', message: t('parking.updateSuccess') });
  });
}
</script>
