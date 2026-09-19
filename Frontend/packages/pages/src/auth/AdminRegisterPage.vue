<template>
  <AuthRegister :subtitle="i18nT('raw.sf3b08b')" />
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import AuthRegister from '../LoginPage.vue';
</script>
