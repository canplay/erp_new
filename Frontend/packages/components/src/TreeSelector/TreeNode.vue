/**
 * @file TreeSelector/TreeNode.vue
 * @description 树形选择器 - 树形节点展示和选择
 * @date 2026-08-22
 */

<template>
  <div class="tree-node">
    <q-tree
      v-if="nodes.length > 0"
      :nodes="nodes"
      node-key="id"
      v-model:selected="selected"
      v-model:expanded="expandedKeys"
      v-model:ticked="ticked"
      :checkable="multiple"
      :radio="!multiple"
      :filter="filter"
      :filter-method="filterMethod"
      color="primary"
      :no-results-label="noResultsLabel"
      class="tree-node__tree"
    >
      <template v-slot:default-header="{ node }">
        <div class="tree-node__content">
          <q-icon
            v-if="node.icon"
            :name="node.icon"
            class="tree-node__icon q-mr-sm"
            :style="{ color: node.iconColor || '#606266' }"
          />
          <span class="tree-node__label">{{ node.label }}</span>
          <span v-if="node.extra" class="tree-node__extra">
            {{ node.extra }}
          </span>
        </div>
      </template>

      <template v-slot:body-loading="">
        <q-spinner-dots color="primary" size="24px" />
      </template>
    </q-tree>

    <EmptyState
      v-else
      icon="folder_open"
      :title="noDataTitle"
      :description="emptyDescription"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import EmptyState from '@erp-new-frontend-monorepo/components/src/EmptyState.vue';

interface TreeNode {
  id: string | number;
  label: string;
  icon?: string;
  iconColor?: string;
  extra?: string;
  children?: TreeNode[];
  disabled?: boolean;
  [key: string]: unknown;
}

interface Props {
  nodes: TreeNode[];
  selected: string | number | (string | number)[];
  expandedKeys: (string | number)[];
  ticked: (string | number)[];
  multiple: boolean;
  filter: string;
  noResultsLabel?: string;
  noDataTitle?: string;
  emptyDescription?: string;
}

const props = withDefaults(defineProps<Props>(), {
  noResultsLabel: '无结果',
  noDataTitle: '无数据',
  emptyDescription: '',
});

function filterMethod(node: TreeNode, filter: string): boolean {
  return node.label.toLowerCase().includes(filter.toLowerCase());
}
</script>

<style scoped>
.tree-node__content {
  display: flex;
  align-items: center;
  min-height: 28px;
}

.tree-node__icon {
  flex-shrink: 0;
}

.tree-node__label {
  flex: 1;
}

.tree-node__extra {
  margin-left: 8px;
  font-size: 12px;
  color: #909399;
}

.tree-node__tree {
  width: 100%;
}
</style>
