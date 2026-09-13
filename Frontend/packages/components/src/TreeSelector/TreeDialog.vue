<template>
  <q-dialog v-model="model" persistent>
    <q-card class="tree-dialog" :style="{ width: dialogWidth || '500px' }">
      <q-card-section class="row items-center q-pb-none">
        <div class="text-h6">{{ title || $t('common.select') }}</div>
        <q-btn icon="close" flat round dense v-close-popup />
      </q-card-section>

      <q-card-section v-if="searchable" class="q-pt-none">
        <q-input
          v-model="searchQuery"
          dense
          outlined
          :placeholder="$t('common.search')"
          clearable
        >
          <template v-slot:prepend>
            <q-icon name="search" />
          </template>
        </q-input>
      </q-card-section>

      <q-card-section v-if="showToolbar" class="q-pt-none">
        <div class="row items-center q-gutter-sm">
          <q-btn flat dense icon="expand_more" :label="expandAll ? $t('tree.collapseAll') : $t('tree.expandAll')" @click="toggleExpandAll" />
          <q-btn v-if="showRefresh" flat dense icon="refresh" :label="$t('common.refresh')" @click="$emit('refresh')" />
          <q-space />
          <span v-if="selectedCount > 0" class="text-caption text-grey-6">
            {{ $t('tree.selected') }}: {{ selectedCount }}
          </span>
        </div>
      </q-card-section>

      <q-card-section class="tree-dialog__tree-container">
        <q-tree
          v-if="data.length > 0"
          :nodes="filteredTreeData"
          node-key="id"
          v-model:selected="localSelected"
          v-model:expanded="expandedKeys"
          v-model:ticked="internalTicked"
          :checkable="multiple"
          :radio="!multiple"
          :filter="searchQuery"
          :filter-method="(node: TreeNode, filter: string) => node.label.toLowerCase().includes(filter.toLowerCase())"
          color="primary"
          :no-results-label="$t('tree.noResults')"
          class="tree-dialog__tree"
        >
          <template v-slot:default-header="{ node }">
            <div class="tree-node-content">
              <q-icon v-if="node.icon" :name="node.icon" class="q-mr-sm" :style="{ color: node.iconColor || '#606266' }" />
              <span class="tree-node-label">{{ node.label }}</span>
              <span v-if="node.extra" class="tree-node-extra">{{ node.extra }}</span>
            </div>
          </template>
          <template v-slot:body-loading="">
            <q-spinner-dots color="primary" size="24px" />
          </template>
        </q-tree>
        <EmptyState v-else icon="folder_open" :title="$t('tree.noData')" :description="emptyDescription || ''" />
      </q-card-section>

      <q-card-actions v-if="showFooter" align="right" class="q-pa-md">
        <q-btn flat :label="$t('common.cancel')" v-close-popup />
        <q-btn v-if="clearable && selectedCount > 0" flat color="negative" :label="$t('common.clearSelection')" @click="handleClear" />
        <q-btn color="primary" :label="$t('common.confirm')" @click="$emit('confirm')" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import EmptyState from '@erp-new-frontend-monorepo/components/src/EmptyState.vue'

const { t: $t } = useI18n()

interface TreeNode {
  id: string | number
  label: string
  icon?: string
  iconColor?: string
  extra?: string
  children?: TreeNode[]
  disabled?: boolean
  [key: string]: unknown
}

interface Props {
  modelValue: boolean
  data: TreeNode[]
  selected?: string | number | (string | number)[]
  multiple?: boolean
  searchable?: boolean
  showToolbar?: boolean
  showRefresh?: boolean
  showFooter?: boolean
  clearable?: boolean
  dialogWidth?: string
  title?: string
  emptyDescription?: string
  defaultExpandAll?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  multiple: false,
  searchable: true,
  showToolbar: true,
  showRefresh: true,
  showFooter: true,
  clearable: true,
  dialogWidth: '500px',
  defaultExpandAll: false
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'update:selected', value: string | number | (string | number)[]): void
  (e: 'refresh'): void
  (e: 'confirm'): void
}>()

const model = ref(false)
const searchQuery = ref('')
const expandedKeys = ref<(string | number)[]>([])
const expandAll = ref(false)
const localSelected = ref(props.selected)

const internalTicked = computed({
  get: () => props.multiple && Array.isArray(props.selected) ? props.selected : [],
  set: (val) => emit('update:selected', val)
})

const selectedCount = computed(() => {
  if (props.multiple && Array.isArray(props.selected)) return props.selected.length
  return props.selected ? 1 : 0
})

const filteredTreeData = computed(() => {
  if (!searchQuery.value) return props.data
  const query = searchQuery.value.toLowerCase()
  function filterNode(node: TreeNode): TreeNode | null {
    const labelMatch = node.label.toLowerCase().includes(query)
    const children = node.children?.map(child => filterNode(child)).filter(child => child !== null)
    if (labelMatch || (children && children.length > 0)) return { ...node, ...(children !== undefined ? { children } : {}) }
    return null
  }
  return props.data.map(node => filterNode(node)).filter(node => node !== null)
})

function toggleExpandAll(): void {
  expandAll.value = !expandAll.value
  if (expandAll.value) expandedKeys.value = getAllNodeIds(props.data)
  else expandedKeys.value = []
}

function getAllNodeIds(nodes: TreeNode[]): (string | number)[] {
  const ids: (string | number)[] = []
  function traverse(nodeList: TreeNode[]): void {
    for (const node of nodeList) {
      ids.push(node.id)
      if (node.children) traverse(node.children)
    }
  }
  traverse(nodes)
  return ids
}

function handleClear(): void {
  emit('update:selected', props.multiple ? [] : '')
  searchQuery.value = ''
}

watch(() => props.modelValue, (val) => { model.value = val })
watch(model, (val) => { emit('update:modelValue', val) })
watch(() => props.defaultExpandAll, (val) => { if (val) { expandedKeys.value = getAllNodeIds(props.data); expandAll.value = true } })
watch(() => props.data, (val) => { if (props.defaultExpandAll && val.length > 0) { expandedKeys.value = getAllNodeIds(val); expandAll.value = true } })
</script>

<style scoped>
.tree-dialog {
  max-height: 70vh;
  display: flex;
  flex-direction: column;
}

.tree-dialog__tree-container {
  flex: 1;
  overflow: auto;
  padding: 16px;
}

.tree-dialog__tree {
  width: 100%;
}

.tree-node-content {
  display: flex;
  align-items: center;
  min-height: 28px;
}

.tree-node-label {
  flex: 1;
}

.tree-node-extra {
  margin-left: 8px;
  font-size: 12px;
  color: #909399;
}
</style>
