<template>
  <div class="header-logo">
    <div class="text-h4 text-white">
      <q-icon name="dashboard" class="q-mr-sm" />
      {{ $t('app.title') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()
</script>

<style scoped>
.header-logo {
  display: flex;
  align-items: center;
}
</style>
