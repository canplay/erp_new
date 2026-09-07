<template>
  <div class="command-item">
    <q-icon :name="command.icon" class="command-item__icon" />
    <div class="command-item__content">
      <div class="text-subtitle2">{{ command.label }}</div>
      <div class="text-caption text-grey-6">{{ command.category }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Command {
  id: string
  label: string
  icon: string
  category: string
  action: () => void
}

defineProps<{
  command: Command
}>()
</script>

<style scoped>
.command-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  border-radius: 4px;
}

.command-item__icon {
  color: #757575;
}

.command-item__content {
  flex: 1;
}
</style>
