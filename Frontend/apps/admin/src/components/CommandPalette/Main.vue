<template>
  <div class="command-palette">
    <q-dialog v-model="showDialog" persistent>
      <q-card class="command-palette__card">
        <q-card-section class="command-palette__header">
          <q-input
            v-model="searchQuery"
            dense
            outlined
            :placeholder="$t('command.search')"
            @input="handleSearch"
          >
            <template v-slot:prepend>
              <q-icon name="search" />
            </template>
          </q-input>
        </q-card-section>

        <q-card-section class="command-palette__content">
          <div v-if="filteredCommands.length === 0" class="command-palette__empty">
            <q-icon name="search_off" size="48px" color="grey-5" />
            <div class="text-subtitle2 q-mt-sm">{{ $t('command.noResults') }}</div>
          </div>

          <div v-else>
            <div
              v-for="(command, index) in filteredCommands"
              :key="command.id"
              class="command-palette__item"
              :class="{ 'command-palette__item--active': activeIndex === index }"
              @click="executeCommand(command)"
              @mouseenter="activeIndex = index"
            >
              <CommandItem :command="command" />
            </div>
          </div>
        </q-card-section>

        <q-card-actions align="right" class="command-palette__footer">
          <span class="text-caption text-grey-6">{{ $t('command.escToClose') }}</span>
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useCommandPalette } from './useCommandPalette'
import CommandItem from './CommandItem.vue'

interface Command {
  id: string
  label: string
  icon: string
  category: string
  action: () => void
}

interface Props {
  commands: Command[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const {
  showDialog,
  searchQuery,
  activeIndex,
  filteredCommands,
  handleSearch,
  executeCommand
} = useCommandPalette({
  commands: props.commands,
  onClose: emit
})

watch(showDialog, (val) => {
  if (!val) {
    emit('close')
  }
})
</script>

<style scoped>
.command-palette__card {
  width: 600px;
  max-width: 90vw;
  max-height: 80vh;
}

.command-palette__header {
  padding: 16px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.12);
}

.command-palette__content {
  max-height: 400px;
  overflow-y: auto;
  padding: 8px;
}

.command-palette__empty {
  text-align: center;
  padding: 32px 0;
}

.command-palette__item {
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.command-palette__item--active {
  background: #e3f2fd;
}

.command-palette__footer {
  padding: 8px 16px;
  border-top: 1px solid rgba(0, 0, 0, 0.12);
}

.body--dark .command-palette__item--active {
  background: #1a237e;
}
</style>
