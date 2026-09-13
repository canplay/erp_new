<template>
  <q-page class="erp-page">
    <div class="row items-center q-mb-md">
      <div class="text-h5">{{ i18nT('raw.scab922') }}</div>
      <q-space />
    </div>

    <q-tabs v-model="tab" align="left" inline-label class="text-primary">
      <q-tab name="brands" icon="local_offer" :label="i18nT('raw.sea8d20')" />
      <q-tab name="categories" icon="account_tree" :label="i18nT('raw.s2f0afe')" />
      <q-tab name="products" icon="inventory_2" :label="i18nT('raw.s651231')" />
    </q-tabs>
    <q-separator />

    <q-tab-panels v-model="tab" animated class="q-pt-md">
      <q-tab-panel name="brands">
        <CatalogBrandsTab />
      </q-tab-panel>
      <q-tab-panel name="categories">
        <CatalogCategoriesTab />
      </q-tab-panel>
      <q-tab-panel name="products">
        <CatalogProductsTab />
      </q-tab-panel>
    </q-tab-panels>
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { ref } from 'vue';
import CatalogBrandsTab from './CatalogBrandsTab.vue';
import CatalogCategoriesTab from './CatalogCategoriesTab.vue';
import CatalogProductsTab from './CatalogProductsTab.vue';

const { t: i18nT } = useI18n();
const tab = ref('brands');
</script>
