<template>
  <UsersView mode="admin" :title="i18nT('raw.s0fed5c')" :dense="dense" />
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t: i18nT } = useI18n();
import { UsersView } from '@erp-new-frontend-monorepo/pages';

withDefaults(defineProps<{ dense?: boolean }>(), { dense: false });
</script>
