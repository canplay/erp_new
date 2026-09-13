/**
 * @file AdvancedSearch.vue
 * @description 高级搜索组件 - 组合 SearchForm + SearchResult
 * @date 2026-04-04
 */

<template>
  <q-card class="advanced-search-panel">
    <q-card-section>
      <div class="row items-center justify-between q-mb-md">
        <div class="text-h6">{{ title }}</div>
        <SearchResult
          :saved-searches="savedSearches"
          :storage-key="storageKey"
          :show-save-button="showSaveButton"
          :title="title"
          @load-search="loadSearch"
          @remove-search="removeSearch"
          @save-search="handleSaveSearch"
          @refresh="handleRefresh"
        />
      </div>

      <!-- 搜索条件表单 -->
      <SearchForm
        :field-options="fieldOptions"
        :initial-conditions="initialConditions"
        :placeholder="$t('common.enterValue')"
        :add-condition-label="$t('common.addCondition')"
        :search-label="$t('common.search')"
        :reset-label="$t('common.reset')"
        @search="handleSearch"
        @reset="handleReset"
      />
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { getStorageItem, setStorageItem } from '@/utils/storage';
import { logger } from '@/utils/logger';
import SearchForm from './SearchForm.vue';
import SearchResult from './SearchResult.vue';

/**
 * 高级搜索组件
 *
 * 提供多条件组合、搜索历史、保存搜索条件等功能
 */

interface SearchCondition {
  field: string;
  operator: string;
  value: unknown;
}

interface SavedSearch {
  name: string;
  conditions: SearchCondition[];
  created_at: number;
}

interface FieldOption {
  label: string;
  value: string;
  type: string;
  options?: Array<{ label: string; value: string }>;
}

interface Props {
  title?: string;
  fieldOptions: FieldOption[];
  showSaveButton?: boolean;
  initialConditions?: SearchCondition[];
  storageKey?: string;
}

const props = withDefaults(defineProps<Props>(), {
  title: '高级搜索',
  showSaveButton: true,
  storageKey: 'saved_searches',
  fieldOptions: () => [],
  initialConditions: () => [],
});

const emit = defineEmits<{
  search: [conditions: SearchCondition[]];
  reset: [];
  refresh: [];
}>();

export type { SearchCondition as AdvancedFilters };

const $q = useQuasar();
const savedSearches = ref<SavedSearch[]>(loadSavedSearches());

function loadSavedSearches(): SavedSearch[] {
  try {
    const saved = getStorageItem<string>(props.storageKey, '');
    return saved ? JSON.parse(saved) : [];
  } catch (error) {
    logger.warn('【加载保存的搜索失败】', error);
    return [];
  }
}

function saveSavedSearches() {
  try {
    setStorageItem(props.storageKey, JSON.stringify(savedSearches.value));
  } catch (e) {
    console.error('Failed to save searches:', e);
  }
}

function handleSearch(conditions: SearchCondition[]) {
  emit('search', conditions);
}

function handleReset() {
  emit('reset');
}

function handleSaveSearch(name: string, conditions: SearchCondition[]) {
  const search: SavedSearch = {
    name,
    conditions,
    created_at: Date.now()
  };

  savedSearches.value.push(search);
  saveSavedSearches();
  $q.notify({ type: 'positive', message: '搜索条件已保存' });
}

function loadSearch(search: SavedSearch) {
  emit('search', search.conditions);
}

function removeSearch(index: number) {
  savedSearches.value.splice(index, 1);
  saveSavedSearches();
}

function handleRefresh() {
  emit('refresh');
}
</script>

<style scoped>
.advanced-search-panel {
  max-width: 800px;
}
</style>
