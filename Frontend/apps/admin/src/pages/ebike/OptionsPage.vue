<template>
  <q-page class="q-pa-md">
    <q-card square style="padding: 20px">
      <q-card-section>
        <div class="text-h6">通用设置</div>
        <q-separator />
        <q-input
          v-model="store.options.system"
          :label="$t('common.timeout')"
          outlined
          dense
        />
      </q-card-section>
      <q-separator />
      <q-card-section>
        <div class="text-h6">报警设置</div>
        <q-separator />
        <q-input
          v-model="store.options.alert"
          :label="$t('common.timeout')"
          outlined
          dense
        />
      </q-card-section>
      <q-card-section class="flex justify-end">
        <q-btn
          color="primary"
          :label="$t('common.save')"
          @click="onSave"
        />
      </q-card-section>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { useQuasar } from 'quasar';
import { useEbikeStore as useStore } from '@/stores/ebike';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
const $q = useQuasar();
const store = useStore();

function onSave(): void {
  // 保存设置到后端
  $q.notify({
    type: 'positive',
    message: '设置已保存',
  });
}
</script>
