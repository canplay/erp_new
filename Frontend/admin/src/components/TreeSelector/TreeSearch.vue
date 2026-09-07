// @ts-nocheck
/**
 * @file TreeSelector/TreeSearch.vue
 * @description 树形选择器 - 搜索和工具栏
 * @date 2026-08-22
 */

<template>
  <div class="tree-search">
    <!-- 搜索框 -->
    <q-card-section v-if="searchable" class="q-pt-none">
      <q-input
        v-model="searchQuery"
        dense
        outlined
        :placeholder="searchPlaceholder"
        clearable
      >
        <template v-slot:prepend>
          <q-icon name="search" />
        </template>
      </q-input>
    </q-card-section>

    <!-- 工具栏 -->
    <q-card-section v-if="showToolbar" class="q-pt-none">
      <div class="row items-center q-gutter-sm">
        <q-btn
          flat
          dense
          icon="expand_more"
          :label="expandAll ? collapseAllLabel : expandAllLabel"
          @click="$emit('toggle-expand')"
        />
        <q-btn
          v-if="showRefresh"
          flat
          dense
          icon="refresh"
          :label="refreshLabel"
          @click="$emit('refresh')"
        />
        <q-space />
        <span v-if="selectedCount > 0" class="text-caption text-grey-6">
          {{ selectedLabel }}: {{ selectedCount }}
        </span>
      </div>
    </q-card-section>
  </div>
</template>

<script setup lang="ts">
interface Props {
  searchable: boolean;
  showToolbar: boolean;
  showRefresh: boolean;
  expandAll: boolean;
  selectedCount: number;
  modelValue?: string;
  searchQuery?: string;
  searchPlaceholder?: string;
  expandAllLabel?: string;
  collapseAllLabel?: string;
  refreshLabel?: string;
  selectedLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  searchable: true,
  showToolbar: true,
  showRefresh: true,
  expandAll: false,
  selectedCount: 0,
  searchQuery: '',
  searchPlaceholder: '搜索',
  expandAllLabel: '展开全部',
  collapseAllLabel: '收起全部',
  refreshLabel: '刷新',
  selectedLabel: '已选择',
});

const _emit = defineEmits<{
  'toggle-expand': [];
  'refresh': [];
}>();
</script>
