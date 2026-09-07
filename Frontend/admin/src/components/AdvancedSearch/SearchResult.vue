/**
 * @file AdvancedSearch/SearchResult.vue
 * @description 高级搜索 - 保存/加载搜索结果
 * @date 2026-08-22
 */

<template>
  <div class="search-result">
    <!-- 保存搜索按钮 -->
    <q-btn
      v-if="showSaveButton"
      flat
      dense
      icon="bookmark"
      :label="saveLabel"
      color="primary"
      @click="showSaveDialog = true"
    />

    <!-- 已保存的搜索列表 -->
    <div v-if="savedSearches.length > 0" class="saved-searches">
      <div class="text-subtitle2 q-mb-sm">{{ title }}</div>
      <div class="row q-gutter-sm">
        <q-chip
          v-for="(search, index) in savedSearches"
          :key="index"
          clickable
          removable
          color="primary"
          text-color="white"
          @click="$emit('load-search', search)"
          @remove="$emit('remove-search', index)"
        >
          {{ search.name }}
        </q-chip>
      </div>
    </div>

    <!-- 保存搜索对话框 -->
    <q-dialog v-model="showSaveDialog">
      <q-card style="min-width: 350px">
        <q-card-section>
          <div class="text-h6">{{ title }}</div>
        </q-card-section>

        <q-card-section>
          <q-input
            v-model="savedSearchName"
            outlined
            :label="nameLabel"
            :rules="[val => !!val || '请输入搜索名称']"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="cancelLabel" v-close-popup />
          <q-btn color="primary" :label="confirmLabel" @click="handleSaveSearch" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useQuasar } from 'quasar';
import { getStorageItem, setStorageItem } from '@/utils/storage';
import { logger } from '@/utils/logger';

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

interface Props {
  savedSearches: SavedSearch[];
  storageKey: string;
  title?: string;
  showSaveButton?: boolean;
  saveLabel?: string;
  nameLabel?: string;
  cancelLabel?: string;
  confirmLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  savedSearches: () => [],
  storageKey: 'saved_searches',
  title: '已保存的搜索',
  showSaveButton: true,
  saveLabel: '保存搜索条件',
  nameLabel: '搜索名称',
  cancelLabel: '取消',
  confirmLabel: '保存',
});

const emit = defineEmits<{
  'load-search': [search: SavedSearch];
  'remove-search': [index: number];
  'save-search': [name: string, conditions: SearchCondition[]];
  'refresh': [];
}>();

const $q = useQuasar();
const showSaveDialog = ref(false);
const savedSearchName = ref('');

function handleSaveSearch() {
  if (!savedSearchName.value.trim()) {
    $q.notify({ type: 'warning', message: '请输入搜索名称' });
    return;
  }

  emit('save-search', savedSearchName.value, JSON.parse(JSON.stringify(props.savedSearches)));
  showSaveDialog.value = false;
  savedSearchName.value = '';
  $q.notify({ type: 'positive', message: '搜索条件已保存' });
}
</script>

<style scoped>
.search-result {
  width: 100%;
}

.saved-searches {
  margin-top: 8px;
}
</style>
