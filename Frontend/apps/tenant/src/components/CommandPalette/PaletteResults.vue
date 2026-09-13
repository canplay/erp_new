/**
 * @file CommandPalette/PaletteResults.vue
 * @description 命令面板 - 搜索结果列表
 * @date 2026-08-22
 */

<template>
  <div class="palette-results">
    <q-list v-if="commands.length > 0">
      <q-item
        v-for="(command, index) in commands"
        :key="command.id"
        clickable
        :active="index === selectedIndex"
        active-class="command-item-active"
        @click="$emit('executeCommand', command)"
        @mouseenter="selectedIndex = index"
      >
        <q-item-section avatar>
          <q-icon :name="command.icon || 'terminal'" />
        </q-item-section>
        <q-item-section>
          <q-item-label>{{ command.name }}</q-item-label>
          <q-item-label caption>{{ command.description }}</q-item-label>
        </q-item-section>
        <q-item-section side>
          <q-chip
            v-if="command.shortcut"
            dense
            outline
            color="grey"
            size="sm"
          >
            {{ command.shortcut }}
          </q-chip>
        </q-item-section>
      </q-item>
    </q-list>

    <div v-else-if="searchQuery" class="no-results">
      <q-icon name="search_off" size="xl" color="grey" />
      <div class="text-grey q-mt-sm">{{ $t('common.noMatchingCommands') }}</div>
    </div>

    <div v-else class="empty-state">
      <q-icon name="keyboard" size="xl" color="grey" />
      <div class="text-grey q-mt-sm">{{ $t('common.inputKeywordSearchCommand') }}</div>
      <div class="text-caption text-grey-6 q-mt-sm">
        按 <kbd>Ctrl</kbd>+<kbd>K</kbd> 打开命令面板
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

import { useI18n } from 'vue-i18n'
const { t: $t } = useI18n()
interface Command {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  category: string;
  shortcut?: string;
  action: () => void | Promise<void>;
  keywords?: string[];
}

interface Props {
  commands: Command[];
  selectedIndex: number;
  searchQuery: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'executeCommand': [command: Command];
  'update:selectedIndex': [index: number];
}>();

const selectedIndex = computed({
  get: () => props.selectedIndex,
  set: (val) => emit('update:selectedIndex', val),
});
</script>

<style scoped>
.palette-results {
  max-height: 400px;
  overflow-y: auto;
  padding-top: 8px;
  padding-bottom: 8px;
}

.command-item-active {
  background: rgba(0, 0, 0, 0.04);
}

.no-results,
.empty-state {
  text-align: center;
  padding: 32px 0;
}
</style>
