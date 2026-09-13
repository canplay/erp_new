<template>
  <AuditsList :dense="dense" />
</template>

<script setup lang="ts">
import { AuditsList } from '@erp-new-frontend-monorepo/pages';

withDefaults(defineProps<{ dense?: boolean }>(), { dense: false });
</script>
