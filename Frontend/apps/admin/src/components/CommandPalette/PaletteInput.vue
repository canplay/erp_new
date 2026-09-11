/**
 * @file CommandPalette/PaletteInput.vue
 * @description 命令面板 - 搜索输入框和分类标签
 * @date 2026-08-22
 */

<template>
  <div class="palette-input">
    <!-- 搜索框 -->
    <q-input
      ref="searchInputRef"
      v-model="localQuery"
      dense
      outlined
      :placeholder="$t('common.inputCommand')"
      autofocus
      class="command-input"
      @keydown="handleKeydown"
    >
      <template #prepend>
        <q-icon name="search" />
      </template>
      <template #append>
        <q-chip
          v-if="selectedCategory"
          dense
          color="primary"
          text-color="white"
          class="category-chip"
        >
          {{ selectedCategory.label }}
          <q-icon name="close" size="xs" class="q-ml-xs" @click="$emit('clearCategory')" />
        </q-chip>
      </template>
    </q-input>

    <!-- 分类标签 -->
    <div class="category-section">
      <div class="row q-gutter-sm">
        <q-btn
          v-for="cat in categories"
          :key="cat.id"
          flat
          dense
          :color="selectedCategory?.id === cat.id ? 'primary' : 'grey'"
          :label="cat.label"
          @click="$emit('selectCategory', cat)"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

interface Category {
  id: string;
  label: string;
  icon?: string;
}

interface Props {
  searchQuery: string;
  selectedCategory: Category | null;
  categories: Category[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:searchQuery': [value: string];
  'selectCategory': [cat: Category];
  'clearCategory': [];
  'keydown': [event: KeyboardEvent];
}>();

const searchInputRef = ref<HTMLInputElement | null>(null);
const localQuery = computed({
  get: () => props.searchQuery,
  set: (val) => emit('update:searchQuery', val),
});

function handleKeydown(event: KeyboardEvent) {
  emit('keydown', event);
}
</script>

<style scoped>
.palette-input {
  padding: 8px 0;
}

.category-section {
  padding-top: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
}

.command-input :deep(.q-field__control) {
  height: 48px;
}
</style>
