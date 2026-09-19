<!-- @layout none -->
<template>
  <div class="not-found-page row items-center justify-center">
    <div class="text-center">
      <div class="nf-code">404</div>
      <div class="text-h5 q-mt-sm">{{ i18nT('raw.s3c816c') }}</div>
      <div class="text-subtitle2 text-grey q-mt-xs q-mb-md">{{ i18nT('raw.seb17d9') }}</div>
      <q-btn color="primary" :label="i18nT('raw.se0cf4d')" @click="goHome" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { useRouter } from 'vue-router';

const router = useRouter();

function goHome() {
  void router.replace('/');
}
</script>

<style scoped>
.not-found-page {
  min-height: 100vh;
}
.nf-code {
  font-size: 96px;
  font-weight: 700;
  color: #9e9e9e;
  line-height: 1;
}
</style>
