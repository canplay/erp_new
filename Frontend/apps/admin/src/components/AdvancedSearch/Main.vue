<template>
  <div class="advanced-search">
    <q-card class="advanced-search__panel">
      <q-card-section>
        <div class="row items-center justify-between q-mb-md">
          <div class="text-h6">{{ title }}</div>
          <q-btn v-if="showSaveButton" flat dense icon="bookmark" :label="$t('common.saveSearch')" color="primary" @click="openSaveDialog" />
        </div>

        <ConditionRow
          v-for="(condition, index) in conditions"
          :key="index"
          :condition="condition"
          :index="index"
          :field-options="fieldOptions"
          @update:condition="(val) => updateCondition(index, val)"
          @remove="removeCondition(index)"
        />

        <q-btn flat dense icon="add" :label="$t('common.addCondition')" color="primary" class="add-condition-btn q-mt-sm" @click="addCondition" />

        <div class="search-actions q-mt-md">
          <q-btn color="primary" icon="search" :label="$t('common.search')" @click="handleSearch" />
          <q-btn flat color="grey" :label="$t('common.reset')" @click="handleReset" />
        </div>
      </q-card-section>

      <SaveDialog v-model="showSaveDialog" :saved-searches="savedSearches" @save="handleSaveSearch" @load="loadSearch" @remove="removeSearch" />

      <q-card-section v-if="savedSearches.length > 0" class="saved-searches">
        <div class="text-subtitle2 q-mb-sm">已保存的搜索</div>
        <div class="row q-gutter-sm">
          <q-chip v-for="(search, index) in savedSearches" :key="index" clickable removable color="primary" text-color="white" @click="loadSearch(search)" @remove="removeSearch(index)">
            {{ search.name }}
          </q-chip>
        </div>
      </q-card-section>
    </q-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import ConditionRow from './ConditionRow.vue'
import SaveDialog from './SaveDialog.vue'
import { useAdvancedSearch } from './useAdvancedSearch'

const { t: $t } = useI18n()

interface SearchCondition {
  field: string
  operator: string
  value: unknown
}

interface SavedSearch {
  name: string
  conditions: SearchCondition[]
  created_at: number
}

interface Props {
  title?: string
  fieldOptions: Array<{ label: string; value: string; type: string; options?: Array<{ label: string; value: string }> }>
  showSaveButton?: boolean
  initialConditions?: SearchCondition[]
  storageKey?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: '高级搜索',
  showSaveButton: true,
  storageKey: 'saved_searches',
  fieldOptions: () => [],
  initialConditions: () => []
})

const emit = defineEmits<{
  (e: 'search', conditions: SearchCondition[]): void
  (e: 'reset'): void
}>()

const {
  conditions,
  savedSearches,
  showSaveDialog,
  addCondition,
  removeCondition,
  updateCondition,
  handleSearch,
  handleReset,
  handleSaveSearch,
  loadSearch,
  removeSearch,
  openSaveDialog
} = useAdvancedSearch({
  fieldOptions: props.fieldOptions,
  initialConditions: props.initialConditions,
  storageKey: props.storageKey,
  onSearch: emit,
  onReset: emit
})
</script>

<style scoped>
.advanced-search__panel {
  max-width: 800px;
}

.add-condition-btn {
  margin-top: 8px;
}

.saved-searches {
  border-top: 1px solid rgba(0, 0, 0, 0.12);
}
</style>
