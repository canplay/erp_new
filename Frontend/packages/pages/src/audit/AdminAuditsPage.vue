<template>
  <AuditsList :dense="dense" />
</template>

<script setup lang="ts">
import AuditsList from '../audit/PermissionChangeLogPage.vue';

withDefaults(defineProps<{ dense?: boolean }>(), { dense: false });
</script>
