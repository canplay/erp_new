<template>
  <q-page class="q-pa-md">
    <div class="text-h5 q-mb-lg">字典配置管理</div>

    <div class="row q-gutter-md">
      <q-card flat bordered class="col-4">
        <q-card-section><div class="text-h6">车辆类型</div></q-card-section>
        <q-separator />
        <q-card-section>
          <q-chip v-for="t in store.carTypeOptions" :key="t.value" color="primary" text-color="white" class="q-ma-xs">{{ t.label }}</q-chip>
          <EmptyState v-if="!store.carTypeOptions.length" icon="settings" :title="$t('common.noResults')" :description="$t('common.noResults')" />
        </q-card-section>
      </q-card>

      <q-card flat bordered class="col-4">
        <q-card-section><div class="text-h6">车辆颜色</div></q-card-section>
        <q-separator />
        <q-card-section>
          <q-chip v-for="c in store.carColorOptions" :key="c.value" color="secondary" text-color="white" class="q-ma-xs">{{ c.label }}</q-chip>
          <EmptyState v-if="!store.carColorOptions.length" icon="settings" :title="$t('common.noResults')" :description="$t('common.noResults')" />
        </q-card-section>
      </q-card>

      <q-card flat bordered class="col-4">
        <q-card-section><div class="text-h6">拖车原因</div></q-card-section>
        <q-separator />
        <q-card-section>
          <q-chip v-for="d in store.dcCauses" :key="d.id" color="warning" text-color="white" class="q-ma-xs">{{ d.name }}</q-chip>
          <EmptyState v-if="!store.dcCauses.length" icon="settings" :title="$t('common.noResults')" :description="$t('common.noResults')" />
        </q-card-section>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useTowStore } from '@/stores/tow';
const store = useTowStore();
onMounted(() => store.fetchDicts());
</script>
