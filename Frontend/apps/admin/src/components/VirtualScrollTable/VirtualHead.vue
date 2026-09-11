<template>
  <thead class="virtual-head">
    <tr>
      <th
        v-for="column in columns"
        :key="column.name"
        :class="{
          'virtual-head__cell': true,
          'virtual-head__cell--sortable': column.sortable,
          [`virtual-head__cell--align-${column.align || 'left'}`]: true
        }"
        @click="column.sortable && $emit('sortChange', column.name)"
      >
        <div class="virtual-head__cell-content">
          <span>{{ column.label }}</span>
          <q-icon
            v-if="column.sortable"
            name="sort"
            size="12px"
            class="virtual-head__sort-icon"
          />
        </div>
      </th>
    </tr>
  </thead>
</template>

<script setup lang="ts">
interface Column {
  name: string
  label: string
  field: string
  sortable?: boolean
  align?: 'left' | 'center' | 'right'
}

defineProps<{
  columns: Column[]
}>()

defineEmits<{
  (e: 'sortChange', fieldName: string): void
}>()
</script>

<style scoped>
.virtual-head {
  background: #f5f5f5;
}

.virtual-head__cell {
  padding: 12px 16px;
  font-weight: 600;
  border-bottom: 1px solid rgba(0, 0, 0, 0.12);
  user-select: none;
}

.virtual-head__cell--sortable {
  cursor: pointer;
}

.virtual-head__cell--sortable:hover {
  background: #e0e0e0;
}

.virtual-head__cell--align-left {
  text-align: left;
}

.virtual-head__cell--align-center {
  text-align: center;
}

.virtual-head__cell--align-right {
  text-align: right;
}

.virtual-head__cell-content {
  display: flex;
  align-items: center;
  gap: 4px;
}

.virtual-head__sort-icon {
  opacity: 0.5;
}
</style>
