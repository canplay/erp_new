<template>
  <AuthLogin :subtitle="i18nT('raw.s60697c')" />
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { AuthLogin } from '@erp-new-frontend-monorepo/pages';
</script>
