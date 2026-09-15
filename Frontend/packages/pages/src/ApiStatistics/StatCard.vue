<template>
  <div class="stat-card">
    <q-card class="stat-card__card">
      <q-card-section class="stat-card__section">
        <div class="row items-center">
          <div class="col">
            <div class="text-subtitle2 text-grey-6">{{ title }}</div>
            <div class="text-h4 q-mt-sm">{{ value }}</div>
          </div>
          <q-icon :name="icon" :color="color" size="48px" />
        </div>
      </q-card-section>
    </q-card>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title: string
  value: string | number
  icon: string
  color: string
}

defineProps<Props>()
</script>

<style scoped>
.stat-card__card {
  min-width: 200px;
}

.stat-card__section {
  display: flex;
  align-items: center;
}
</style>
