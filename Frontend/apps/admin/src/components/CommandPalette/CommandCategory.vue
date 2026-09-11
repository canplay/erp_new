<template>
  <div class="command-category">
    <div class="text-subtitle2 text-grey-6 q-mb-sm">{{ category }}</div>
    <div class="command-category__list">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  category: string
}>()
</script>

<style scoped>
.command-category__list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
</style>
